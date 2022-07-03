
use std::{
    io::{self, Write},
    fs::{self, File, read_to_string},
    sync,
    thread,
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering}
    },
    time::Duration,
    path::Path
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers, poll};

mod ucd;
mod app;
mod fuzzy;

fn main() -> Result<(), io::Error> {

    let mut filename = match dirs::cache_dir() {
        Some(path) => {path},
        None => {
            eprintln!("Error: could not find cache directory,\
                using current working directory");
            match std::env::current_dir() {
                Ok(path) => path,
                Err(err) => {
                    panic!("could not find current working directory: {}", err);
                }
            }
        }
    };
    filename.push("ferris-finder.data");

    let running = Arc::new(AtomicBool::new(true));
    let (event_sender, event_receiver) = sync::mpsc::channel::<Event>();

    let running_copy = running.clone();
    thread::spawn(move || {
        let mut ctrlc_pressed = false;
        let sender = event_sender;
        let running = running_copy;
        loop {
            match poll(Duration::new(0,0)) {
                Ok(available) => {
                    if !available { thread::sleep(Duration::from_millis(10)); }
                }
                Err(_) => ()
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
                        if ch == 'c' &&
                        key.modifiers.contains(KeyModifiers::CONTROL) {
                            if ctrlc_pressed { exit(2); }
                            running.store(false, Ordering::Relaxed);
                            ctrlc_pressed = true;
                            continue;
                        }
                    },
                    KeyCode::Esc => {
                        running.store(false, Ordering::Relaxed);
                        continue;
                    },
                    _ => ()
                }
            }

            if sender.send(event).is_err() { return; }
        }
    });



    let _ = get_unicode_data_file(&filename);
    let data = match read_to_string(&filename) {
        Ok(data) => data,
        Err(err) => {
            eprintln!(
                "Error reading unicode data file '{}': {}",
                filename.to_string_lossy(),
                err
            );
            exit(1);
        }
    };

    let data = data.trim()
        .split("\n")
        .filter_map(|x| ucd::CharEntry::from_ucd_line(x))
        .collect::<Vec<ucd::CharEntry>>();

    let mut app = match app::App::new(
        running.clone(),
        io::stdout(),
        event_receiver,
        data
    ) {
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

fn get_unicode_data_file(filename: &Path) -> File {
    match fs::File::open(filename) {
        Ok(file) => file,
        Err(_) => {
            eprintln!(
                "Couldn't find data file, attempting to download to:\n{}",
                filename.to_string_lossy()
            );
            match fs::File::create(filename) {
                Ok(mut new_file) => {
                    match reqwest::blocking::get(
                    "http://www.unicode.org/Public/UCD/latest/ucd/\
                        UnicodeData.txt"
                    ) {
                        Ok(response) => match response.text() {
                            Ok(text) => match
                            new_file.write_all(text.as_bytes()) {
                                Ok(_) => (),
                                Err(err) => {
                                    eprintln!(
                                        "Error writing unicode data to file:\
                                        {}",
                                        err
                                    );
                                }
                            },
                            Err(err) => {
                                eprintln!(
                                    "Error resolving http request: {}",
                                    err
                                );
                                exit(1);
                            }
                        },
                        Err(err) => {
                            eprintln!(
                                "Error while downloading Unicode data: {}",
                                err
                            );
                            exit(1);
                        }
                    };
                    get_unicode_data_file(filename)
                },
                Err(err) => {
                    eprintln!(
                        "Error creating new file for Unicode data: {}",
                        err
                    );
                    exit(1);
                }
            }
        }
    }
}
