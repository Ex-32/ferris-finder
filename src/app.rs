use std::{
    io::{self, Stdout},
    sync::{
        self,
        atomic::{AtomicBool,Ordering},
        Arc
    }
};
use crossterm::{
    terminal::{
        EnterAlternateScreen,
        LeaveAlternateScreen,
        enable_raw_mode,
        disable_raw_mode,
    },
    event::{
        Event,
        EnableMouseCapture,
        DisableMouseCapture,
        KeyCode,
        KeyModifiers,
        MouseEventKind
    },
};
use tui::{
    layout::{self, Layout, Constraint},
    widgets::{self, *},
    backend::CrosstermBackend,
    Terminal,
    terminal::CompletedFrame, style::{Style, Color, Modifier},
};
use crate::ucd;
use crate::fuzzy;

pub struct App {
    running_flag: Arc<AtomicBool>,
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_receiver: sync::mpsc::Receiver<Event>,
    data: Vec<ucd::CharEntry>,
    table_state: TableState,
    table_data: Vec<Vec<String>>,
    search: String,
    pub exit_buffer: Option<String>,
}

impl App {
    pub fn new(
        running_flag: Arc<AtomicBool>,
        mut stdout: Stdout,
        event_receiver: sync::mpsc::Receiver<Event>,
        data: Vec<ucd::CharEntry>
    ) -> io::Result<Self> {
        enable_raw_mode()?;
        crossterm::execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = match Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(err) => return Err(err),
        };
        let table_data = App::table_items_from_data(&data);
        let mut table_state = TableState::default();
        table_state.select(Some(0));

        Ok(App {
            running_flag,
            terminal,
            event_receiver,
            data,
            table_state,
            table_data,
            search: String::new(),
            exit_buffer: None,
        })
    }

    // ANCHOR draw UI function
    pub fn draw(&mut self) -> io::Result<CompletedFrame> {
        self.terminal.draw(|f|{
            let size = f.size();
            let rects = Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(
                    [
                        layout::Constraint::Percentage(90),
                        layout::Constraint::Min(3),
                    ].as_ref()
                )
                .split(size);
            let selected_style = Style::default()
                .add_modifier(Modifier::REVERSED);
            let header_cells = ["Char", "Code", "Name"]
                .iter()
                .map(|x| {
                    Cell::from(*x)
                        .style(Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD))
                }
            );
            let header = Row::new(header_cells)
                .style(Style::default())
                .height(1);
            let rows = self.table_data.iter().map(|item| {
                let cells = item.iter().map(|c| Cell::from(c.as_ref()));
                Row::new(cells).height(1)
            });
            let t = Table::new(rows)
                .header(header)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title("ferris-finder")
                )
                .highlight_style(selected_style)
                .highlight_symbol("|> ")
                .widths(&[
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                ]);

            let search = widgets::Paragraph::new(format!("{}█", self.search))
                .block(
                    widgets::Block::default()
                        .borders(widgets::Borders::ALL)
                        .title("search: ")
                );

            f.render_stateful_widget(t, rects[0], &mut self.table_state);
            f.render_widget(search, rects[1]);
        })
    }

    // ANCHOR update state function
    pub fn update(&mut self) -> io::Result<()> {
        let old_search = self.search.clone();
        loop {
            let event = match self.event_receiver.try_recv() {
                Ok(event) => event,
                Err(e) => match e {
                    sync::mpsc::TryRecvError::Empty => break,
                    sync::mpsc::TryRecvError::Disconnected => {
                        panic!("Error: event handler thread disconnected\
                        prematurely")
                    }
                }
            };

            match event {
                Event::Mouse(mouse) => {
                    match mouse.kind {
                        MouseEventKind::ScrollUp => self.table_up(1),
                        MouseEventKind::ScrollDown => self.table_down(1),
                        _ => ()
                    }
                },
                Event::Key(key) => match key.code {
                    KeyCode::Up => self.table_up(1),
                    KeyCode::Down => self.table_down(1),

                    KeyCode::PageUp => self.table_up(10),
                    KeyCode::PageDown => self.table_down(10),

                    KeyCode::Home => self.table_state.select(Some(0)),
                    KeyCode::End => self.table_state.select(
                        Some(self.table_data.len() - 1)
                    ),

                    KeyCode::Char(ch) => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.search.extend(ch.to_uppercase());
                        } else {
                            self.search.push(ch);
                        }
                        continue;
                    },
                    KeyCode::Backspace => {
                        self.search.pop();
                        continue;
                    },
                    KeyCode::Enter => {
                        let i = self.table_state.selected();
                        if i.is_some() {
                            let i = i.unwrap();
                            self.exit_buffer = match self.table_data.get(i) {
                                Some(entry) => match entry.get(0) {
                                    Some(str) => Some(str.clone()),
                                    None => None,
                                },
                                None => None,
                            };
                            self.running_flag.store(false, Ordering::Relaxed);
                        }
                    }
                    _ => ()
                },
                _ => ()
            }
        }
        if old_search.ne(&self.search) {
            self.table_data = App::table_items_from_data(&fuzzy::prune(
                &self.data,
                &self.search
            ));
            self.table_state.select(Some(0));
        }
        Ok(())
    }

    // ANCHOR helper functions

    fn table_items_from_data(data: &Vec<ucd::CharEntry>) -> Vec<Vec<String>> {
        let new = data.iter()
            .map(|x| {
                vec![
                    char::from_u32(x.codepoint)
                        .unwrap_or(char::REPLACEMENT_CHARACTER)
                        .to_string(),
                    ucd::CharEntry::fmt_codepoint(x.codepoint),
                    x.name.clone(),
                    x.unicode_1_name.clone()
                ]
            })
        .collect();
        new
    }

    fn table_down(&mut self, count: usize) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i + count > self.table_data.len() - 1 {
                    (i + count) - self.table_data.len()
                }
                else { i + count }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn table_up(&mut self, count: usize) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if count > i {
                    self.table_data.len() - (count - i)
                }
                else { i - count }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

impl Drop for App {
    fn drop(&mut self) -> () {
        let _ = disable_raw_mode();
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
