use crate::fuzzy;
use crate::ucd;
use crossterm::{
    event::{self, Event, KeyCode, KeyModifiers, MouseEventKind},
    terminal,
};
use rayon::prelude::*;
use std::{
    io,
    sync::{
        self,
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};
use tui::{
    backend::CrosstermBackend,
    layout::{self, Constraint},
    style::{Color, Modifier, Style},
    text,
    widgets::{self, BorderType, Borders},
    Terminal,
};

pub struct App {
    running_flag: Arc<AtomicBool>,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
    event_receiver: sync::mpsc::Receiver<Event>,
    data: Box<[ucd::CharEntry]>,
    table_state: widgets::TableState,
    table_data: Box<[Box<[Box<str>]>]>,
    search: String,
    pub exit_buffer: Option<Box<str>>,
}

impl App {
    pub fn new(
        running_flag: Arc<AtomicBool>,
        mut stdout: io::Stdout,
        event_receiver: sync::mpsc::Receiver<Event>,
        data: Box<[ucd::CharEntry]>,
    ) -> io::Result<Self> {
        terminal::enable_raw_mode()?;
        crossterm::execute!(
            stdout,
            terminal::EnterAlternateScreen,
            event::EnableMouseCapture
        )?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = match tui::Terminal::new(backend) {
            Ok(terminal) => terminal,
            Err(err) => return Err(err),
        };
        let table_data = App::table_items_from_data(&data.par_iter().collect::<Vec<_>>());
        let mut table_state = widgets::TableState::default();
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
    pub fn draw(&mut self) -> io::Result<tui::terminal::CompletedFrame> {
        self.terminal.draw(|f| {
            let size = f.size();
            let rects = layout::Layout::default()
                .direction(layout::Direction::Vertical)
                .constraints(
                    [
                        Constraint::Min(1),
                        Constraint::Min(3),
                        Constraint::Percentage(100),
                    ]
                    .as_ref(),
                )
                .split(size);
            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let header_cells = ["Char", "Code", "Name"].iter().map(|x| {
                widgets::Cell::from(*x).style(
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                )
            });
            let header = widgets::Row::new(header_cells)
                .style(Style::default())
                .height(1);
            let rows = self.table_data.iter().map(|item| {
                let cells = item.iter().map(|c| widgets::Cell::from(c.as_ref()));
                widgets::Row::new(cells).height(1)
            });
            let table = widgets::Table::new(rows)
                .header(header)
                .block(
                    widgets::Block::default()
                        .borders(Borders::ALL)
                        .border_type(BorderType::Rounded), // .title("ferris-finder")
                )
                .highlight_style(selected_style)
                .highlight_symbol("|> ")
                .widths(&[
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                ]);

            let search = widgets::Paragraph::new(format!("{}█", self.search)).block(
                widgets::Block::default()
                    .borders(Borders::ALL)
                    .border_type(BorderType::Rounded)
                    .title(" Search "),
            );

            let title = widgets::Block::default().title(text::Span::styled(
                "Ferris-Finder",
                Style::default().add_modifier(Modifier::BOLD),
            ));

            f.render_widget(title, rects[0]);
            f.render_widget(search, rects[1]);
            f.render_stateful_widget(table, rects[2], &mut self.table_state);
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
                        panic!("event handler thread disconnected prematurely")
                    }
                },
            };

            match event {
                Event::Mouse(mouse) => match mouse.kind {
                    MouseEventKind::ScrollUp => self.table_up(1),
                    MouseEventKind::ScrollDown => self.table_down(1),
                    _ => (),
                },
                Event::Key(key) => match key.code {
                    KeyCode::Esc => self.running_flag.store(false, Ordering::Relaxed),

                    KeyCode::Up => self.table_up(1),
                    KeyCode::Down => self.table_down(1),

                    KeyCode::PageUp => self.table_up(10),
                    KeyCode::PageDown => self.table_down(10),

                    KeyCode::Home => self.table_state.select(Some(0)),
                    KeyCode::End => self.table_state.select(Some(self.table_data.len() - 1)),

                    KeyCode::Char(ch) => {
                        if key.modifiers.contains(KeyModifiers::SHIFT) {
                            self.search.extend(ch.to_uppercase());
                        } else {
                            self.search.push(ch);
                        }
                        continue;
                    }
                    KeyCode::Backspace => {
                        self.search.pop();
                        continue;
                    }
                    KeyCode::Enter => {
                        let i = self.table_state.selected();
                        if let Some(i) = i {
                            self.exit_buffer = match self.table_data.get(i) {
                                Some(entry) => entry.get(0).cloned(),
                                None => None,
                            };
                            self.running_flag.store(false, Ordering::Relaxed);
                        }
                    }
                    _ => (),
                },
                _ => (),
            }
        }
        if old_search.ne(&self.search) {
            self.table_data = App::table_items_from_data(&fuzzy::prune(&self.data, &self.search));
            self.table_state.select(Some(0));
        }
        Ok(())
    }

    // ANCHOR helper functions

    fn table_items_from_data(data: &[&ucd::CharEntry]) -> Box<[Box<[Box<str>]>]> {
        data.par_iter()
            .map(|x| {
                vec![
                    char::from_u32(x.codepoint)
                        .unwrap_or(char::REPLACEMENT_CHARACTER)
                        .to_string()
                        .into_boxed_str(),
                    ucd::CharEntry::fmt_codepoint(x.codepoint),
                    x.name.clone(),
                    x.unicode_1_name.clone(),
                ]
                .into_boxed_slice()
            })
            .collect::<Vec<_>>()
            .into_boxed_slice()
    }

    fn table_down(&mut self, count: usize) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i + count > self.table_data.len() - 1 {
                    (i + count) - self.table_data.len()
                } else {
                    i + count
                }
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
                } else {
                    i - count
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }
}

impl Drop for App {
    fn drop(&mut self) {
        let _ = terminal::disable_raw_mode();
        let _ = crossterm::execute!(
            self.terminal.backend_mut(),
            terminal::LeaveAlternateScreen,
            event::DisableMouseCapture
        );
        let _ = self.terminal.show_cursor();
    }
}
