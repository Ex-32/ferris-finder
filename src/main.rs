
use std::{
    io,
    fs,
    sync,
    thread,
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering}
    }
};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};

mod ucd;
mod app;
mod fuzzy;

fn main() -> Result<(), io::Error> {

    let running = Arc::new(AtomicBool::new(true));
    let (event_sender, event_receiver) = sync::mpsc::channel::<Event>();

    let running_copy = running.clone();
    thread::spawn(move || {
        let mut ctrlc_pressed = false;
        let sender = event_sender;
        let running = running_copy;
        loop {
            let event = match event::read() {
                Ok(event) => event,
                Err(err) => {
                    panic!("Fatal error reading key events: {}", err);
                }
            };

            if let Event::Key(key) = event {
                if let KeyCode::Char(ch) = key.code {
                    if ch == 'c' &&
                    key.modifiers.contains(KeyModifiers::CONTROL) {
                        if ctrlc_pressed { exit(2); }
                        running.store(false, Ordering::Relaxed);
                        ctrlc_pressed = true;
                        continue;
                    }
                }
            }

            if sender.send(event).is_err() { return; }
        }
    });

    let filename = "data/UnicodeData.csv";

    let data = match fs::read_to_string(filename) {
        Ok(data) => data.trim()
            .split("\n")
            .filter_map(|x| ucd::CharEntry::from_ucd_line(x))
            .collect::<Vec<ucd::CharEntry>>(),
        Err(err) => {
            println!("Error reading unicode data file '{}': {}", filename, err);
            exit(1);
        }
    };

    let mut app = match app::App::new(
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
    }

    drop(app);
    Ok(())
}
