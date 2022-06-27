
use std::{
    io,
    thread,
    process::exit,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering}
    },
};
use crossterm::{
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
    event::{EnableMouseCapture, DisableMouseCapture},
};
use tui::{
    backend::CrosstermBackend,
    Terminal,
};
use signal_hook::{
    iterator::Signals,
    consts::{SIGTERM, SIGQUIT, SIGINT},
};

fn main() -> Result<(), io::Error> {

    let running = Arc::new(AtomicBool::new(true));

    let signal_running =  running.clone();
    let mut signals = Signals::new([SIGTERM, SIGQUIT, SIGINT])?;
    thread::spawn(move || {
        let running = signal_running;
        let ctrlc  = AtomicBool::new(false);
        for sig in signals.forever() {
            match sig {
                SIGTERM => running.swap(false, Ordering::Relaxed),
                SIGQUIT => running.swap(false, Ordering::Relaxed),
                SIGINT => {
                    if ctrlc.load(Ordering::SeqCst) {
                        exit(1);
                    }
                    ctrlc.store(true, Ordering::SeqCst);
                    running.swap(false, Ordering::Relaxed)
                },
                _ => unreachable!()
            };
        }
    });


    let mut stdout = io::stdout();
    crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
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

    crossterm::execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
