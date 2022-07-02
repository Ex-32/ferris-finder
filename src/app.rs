use std::{
    io::{self, Stdout},
    sync, vec,
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
        KeyModifiers
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
// use crate::fuzzy;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Focus {
    Table,
    Search
}

impl Focus {
    fn toggle(&mut self) {
        match self {
            Focus::Table => *self = Focus::Search,
            Focus::Search => *self = Focus::Table,
        }
    }
}

pub struct App {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    event_receiver: sync::mpsc::Receiver<Event>,
    data: Vec<ucd::CharEntry>,

    table_state: TableState,
    table_data: Vec<Vec<String>>,

    search: String,
    focus: Focus
}

impl App {
    pub fn new(
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
            terminal,
            event_receiver,
            data,
            table_state,
            table_data,
            search: String::new(),
            focus: Focus::Table
        })
    }

    pub fn draw(&mut self) -> io::Result<CompletedFrame>{
        self.terminal.draw(|f|{
            let size = f.size();
            let rects = Layout::default()
                .direction(layout::Direction::Vertical)
                // .margin(1)
                .constraints(
                    [
                        layout::Constraint::Percentage(90),
                        layout::Constraint::Max(10),
                    ].as_ref()
                )
                .split(size);

            let selected_style = Style::default().add_modifier(Modifier::REVERSED);
            let header_cells = ["Character", "Codepoint", "Name"]
                .iter()
                .map(|h| Cell::from(*h).style(Style::default().fg(Color::White).add_modifier(Modifier::BOLD)));
            let header = Row::new(header_cells)
                .style(Style::default())
                .height(1)
                .bottom_margin(1);
            let rows = self.table_data.iter().map(|item| {
                let cells = item.iter().map(|c| Cell::from(c.as_ref()));
                Row::new(cells).height(1)
            });
            let t = Table::new(rows)
                .header(header)
                .block(Block::default().borders(Borders::ALL).title("Table"))
                .highlight_style(selected_style)
                .highlight_symbol("|> ")
                .widths(&[
                    Constraint::Percentage(10),
                    Constraint::Percentage(10),
                    Constraint::Percentage(40),
                    Constraint::Percentage(40),
                ]);


            let search_text: String;
            if self.focus == Focus::Search {
                search_text = format!("{}█", self.search);
            } else {
                search_text = self.search.clone();
            }

            let search = widgets::Paragraph::new(search_text)
                .block(
                    widgets::Block::default()
                        .borders(widgets::Borders::ALL)
                        .title("search: ")
                );


            f.render_stateful_widget(t, rects[0], &mut self.table_state);
            f.render_widget(search, rects[1]);
        })
    }

    pub fn update(&mut self) -> io::Result<()> {
        let old_search = self.search.clone();
        loop {
            let event = match self.event_receiver.try_recv() {
                Ok(event) => event,
                Err(e) => match e {
                    sync::mpsc::TryRecvError::Empty => break,
                    sync::mpsc::TryRecvError::Disconnected => {
                        panic!("Error: event handler thread disconnected prematurely")
                    }
                }
            };
            match event {
                Event::Key(key) => {
                    match self.focus {
                        Focus::Search => {
                            match key.code {
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
                                _ => ()
                            }
                        },
                        Focus::Table => {
                            match key.code {
                                KeyCode::Up => self.table_previous(),
                                KeyCode::Down => self.table_next(),
                                _ => ()
                            }
                        }
                    };
                    match key.code {
                        KeyCode::Tab => {
                            self.focus.toggle();
                        },
                        _ => ()
                    };
                }
                _ => ()
                // Event::Mouse(mouse_event) => match mouse_event {

                // }
            }
        }
        if old_search.ne(&self.search) {
            self.table_data = App::table_items_from_data(&self.data);
        }

        Ok(())
    }

    fn table_items_from_data(data: &Vec<ucd::CharEntry>) -> Vec<Vec<String>> {
        let new = data.iter()
            .map(|x| {
                vec![
                    char::from_u32(x.codepoint)
                        .unwrap_or(char::REPLACEMENT_CHARACTER)
                        .to_string(),
                    format!("U+{:X}", x.codepoint),
                    x.name.clone(),
                    x.unicode_1_name.clone()
                ]
            })
        .collect();
        new
    }

    fn table_next(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i >= self.table_data.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.table_state.select(Some(i));
    }

    fn table_previous(&mut self) {
        let i = match self.table_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.table_data.len() - 1
                } else {
                    i - 1
                }
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
