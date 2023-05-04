use color_eyre::eyre::{self, WrapErr, Result};
use crossterm::event::{self, poll, Event, KeyCode, KeyModifiers};
use std::{
    fs,
    io::{self, Write, Read},
    path::{Path, PathBuf},
    process::exit,
    sync,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

mod app;
mod fuzzy;
mod ucd;

fn main() -> Result<()> {
    color_eyre::install()?;

    // create PathBuf pointing to a file to download/cache the UCD data in the
    // user's cache directory
    let filename: PathBuf = [
        dirs::cache_dir()
            .ok_or(eyre::eyre!("unable to locate cache directory"))?,
        "ferris-finder.ucd.cache".into()
    ].iter().collect();

    let running = Arc::new(AtomicBool::new(true));
    let (event_sender, event_receiver) = sync::mpsc::channel::<Event>();

    let running_copy = running.clone();
    thread::spawn(move || {
        let mut ctrlc_pressed = false;
        let sender = event_sender;
        let running = running_copy;
        loop {
            if let Ok(available) = poll(Duration::new(0, 0)) {
                if !available {
                    thread::sleep(Duration::from_millis(10));
                }
            }

            let event = match event::read() {
                Ok(event) => event,
                Err(err) => {
                    panic!("failed to read key events: {}", err);
                }
            };

            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char(ch) => {
                        if ch == 'c' && key.modifiers.contains(KeyModifiers::CONTROL) {
                            if ctrlc_pressed {
                                exit(2);
                            }
                            running.store(false, Ordering::Relaxed);
                            ctrlc_pressed = true;
                            continue;
                        }
                    }
                    KeyCode::Esc => {
                        running.store(false, Ordering::Relaxed);
                        continue;
                    }
                    _ => (),
                }
            }

            if sender.send(event).is_err() {
                return;
            }
        }
    });

    let data = get_unicode_data(&filename)
        .wrap_err("unable to retrive unicode data")?
        .trim()
        .split('\n')
        .filter_map(ucd::CharEntry::from_ucd_line)
        .collect::<Vec<_>>();

    let mut app = match app::App::new(running.clone(), io::stdout(), event_receiver, data) {
        Ok(app) => app,
        Err(err) => {
            println!("Error initializing display: {}", err);
            exit(1);
        }
    };

    while running.load(Ordering::Relaxed) {
        app.update()?;
        app.draw()?;
        thread::sleep(Duration::from_millis(30));
    }

    let exit_buffer = app.exit_buffer.clone();
    drop(app);

    if exit_buffer.is_some() {
        println!("{}", exit_buffer.unwrap());
    }

    Ok(())
}

fn get_unicode_data(filename: &Path) -> Result<String> {

    match fs::File::open(filename) {
        Ok(mut file) => {
            let mut ucd = String::new();
            let _ = file.read_to_string(&mut ucd)
                .wrap_err("unable to read UCD data from cache")?;
            Ok(ucd)
        },
        Err(_) => {
            eprintln!(
                "failed to open UCD cache, attempting to download to:\n{}",
                filename.to_string_lossy()
            );
            let mut new_file = fs::File::create(filename)
                .wrap_err("unable to create new file for UCD cache")?;

            let ucd = reqwest::blocking::get(
                "http://www.unicode.org/Public/UCD/latest/ucd/\
                UnicodeData.txt"
            ).wrap_err("error in http GET for UCD from www.unicode.org")?
                .text()
                .wrap_err("error decoding UCD message body")?;

            match new_file.write_all(ucd.as_bytes()) {
                Ok(()) => (),
                Err(e) => 
                    eprintln!("failed to write UCD data to cache file: {}", e),
            }

            Ok(ucd)
        }
    }
}
