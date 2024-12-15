// ANCHOR: imports
use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{Clear, ClearType},
};
use ratatui::{
    layout::Rect,
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Widget},
    DefaultTerminal, Frame,
};
// ANCHOR_END: imports

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

// ANCHOR: app
#[derive(Debug, Default)]
pub struct App {
    ssh_clients: Vec<String>,
    selected_index: usize,
    exit: bool,
}
// ANCHOR_END: app

// ANCHOR: impl App
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        self.ssh_clients = vec![
            "xpipe-client-1".to_string(),
            "xpipe-client-2".to_string(),
            "xpipe-client-3".to_string(),
        ];

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let items: Vec<ListItem> = self
            .ssh_clients
            .iter()
            .enumerate()
            .map(|(i, client)| {
                let content = if i == self.selected_index {
                    Line::from(client.clone().bold().yellow())
                } else {
                    Line::from(client.clone())
                };
                ListItem::new(content)
            })
            .collect();

        let block = Block::default().title("SSH Clients").borders(Borders::ALL);

        let list = List::new(items).block(block);

        frame.render_widget(list, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Down,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.selected_index < self.ssh_clients.len() - 1 {
                    self.selected_index += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.open_session();
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.exit();
            }
            _ => {}
        };
        Ok(())
    }

    fn open_session(&self) {
        // Clear the screen before printing
        execute!(io::stdout(), Clear(ClearType::All)).unwrap();
        println!(
            "\nOpening terminal session for: {}",
            self.ssh_clients[self.selected_index]
        );
        println!("Press any key to return...");

        let _ = event::read(); // Wait for any key press before returning
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
// ANCHOR_END: impl App
