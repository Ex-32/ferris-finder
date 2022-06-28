
use std::{
    io,
    fs,
    thread,
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering}
    },
};
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen}
};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use signal_hook::{
    iterator::Signals,
    consts::{SIGTERM, SIGQUIT, SIGINT},
};

mod ucd;

fn main() -> Result<(), io::Error> {

    let running = Arc::new(AtomicBool::new(true));

    let signal_running =  running.clone();
    let mut signals = Signals::new([SIGTERM, SIGQUIT, SIGINT])?;
    thread::spawn(move || {
        let running = signal_running;
        let ctrlc_pressed  = AtomicBool::new(false);
        for sig in signals.forever() {
            match sig {
                SIGTERM => running.swap(false, Ordering::Relaxed),
                SIGQUIT => running.swap(false, Ordering::Relaxed),
                SIGINT  => {
                    if ctrlc_pressed.load(Ordering::SeqCst) {
                        exit(1);
                    }
                    ctrlc_pressed.store(true, Ordering::SeqCst);
                    running.swap(false, Ordering::Relaxed)
                },
                _ => unreachable!()
            };
        }
    });

    let filename = "data/UnicodeData.csv";

    let data = match fs::read_to_string(filename) {
        Ok(data) => data
            .trim()
            .split("\n")
            .filter_map(|x| ucd::CharEntry::from_ucd_line(x))
            .collect::<Vec<ucd::CharEntry>>(),
        Err(err) => {
            println!("Error reading unicode data file: {}", err);
            exit(1);
        }
    };

    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    terminal.draw(|f| {
        let size = f.size();
        let block = tui::widgets::Block::default()
            .title("Block")
            .borders(tui::widgets::Borders::ALL);
        f.render_widget(block, size);
    })?;

    while running.load(Ordering::Relaxed) {
        thread::sleep(std::time::Duration::from_millis(10));
    }

    crossterm::execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}
