// ANCHOR: imports
use dotenvy::dotenv;
use std::env;
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
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
// ANCHOR_END: imports

// API URL and API Key Constants
const API_URL: &str = "http://localhost:21721";

// ANCHOR: structs
#[derive(Debug, Default)]
pub struct App {
    ssh_clients: Vec<String>,
    selected_index: usize,
    exit: bool,
    session_token: Option<String>,
}

#[derive(Deserialize)]
struct HandshakeResponse {
    sessionToken: String,
}

#[derive(Deserialize)]
struct ConnectionQueryResponse {
    found: Vec<String>,
}

#[derive(Deserialize)]
struct ConnectionInfoResponse {
    infos: Vec<ConnectionInfo>,
}

#[derive(Deserialize)]
struct ConnectionInfo {
    connection: String,
    name: Vec<String>,
}
// ANCHOR_END: structs

fn main() -> io::Result<()> {
    dotenv().ok();
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

// ANCHOR: impl App
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Perform handshake to get the session token
        match self.handshake() {
            Ok(token) => {
                self.session_token = Some(token);
                if let Err(err) = self.fetch_connections() {
                    eprintln!("Error fetching connections: {}", err);
                }
            }
            Err(err) => {
                eprintln!("Error during handshake: {}", err);
            }
        }

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
        execute!(io::stdout(), Clear(ClearType::All)).unwrap();
        if let Some(selected) = self.ssh_clients.get(self.selected_index) {
            println!("\nOpening terminal session for: {}", selected);
            println!("Press any key to return...");
            let _ = event::read();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn handshake(&self) -> Result<String, reqwest::Error> {
        let api_key =
            env::var("XPIPE_API_KEY").expect("XPIPE_API_KEY environment variable not set");
        let client = Client::new();

        let response: HandshakeResponse = client
            .post(format!("{}/handshake", API_URL))
            .json(&serde_json::json!({
                "auth": {
                    "type": "ApiKey",
                    "key": api_key
                },
                "client": {
                    "type": "Api",
                    "name": "xpcli"
                }
            }))
            .send()?
            .json()?;

        Ok(response.sessionToken)
    }

    fn fetch_connections(&mut self) -> Result<(), reqwest::Error> {
        if let Some(token) = &self.session_token {
            let client = Client::new();
            let response: ConnectionQueryResponse = client
                .post(format!("{}/connection/query", API_URL))
                .bearer_auth(token)
                .json(&serde_json::json!({
                    "categoryFilter": "*",
                    "connectionFilter": "*",
                    "typeFilter": "*"
                }))
                .send()?
                .json()?;

            let connection_ids = response.found;

            let info_response: ConnectionInfoResponse = client
                .post(format!("{}/connection/info", API_URL))
                .bearer_auth(token)
                .json(&serde_json::json!({
                    "connections": connection_ids
                }))
                .send()?
                .json()?;

            self.ssh_clients = info_response
                .infos
                .iter()
                .flat_map(|info| info.name.clone())
                .collect();
        }

        Ok(())
    }
}
// ANCHOR_END: impl App
