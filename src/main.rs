// ANCHOR: imports
use std::collections::HashMap;
use std::io;

use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    execute,
    terminal::{Clear, ClearType},
};
use ratatui::{
    style::{Color, Modifier, Stylize},
    text::Line,
    widgets::{Block, Borders, List, ListItem, Widget},
    DefaultTerminal, Frame,
};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
// ANCHOR_END: imports

// API URL
const API_URL: &str = "http://localhost:21721";

// ANCHOR: structs
#[derive(Debug, Default)]
pub struct App {
    servers: Vec<String>,
    resources: HashMap<String, Vec<String>>,
    selected_index: usize,
    viewing_resources: bool,
    current_server: String,
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
    name: Vec<String>,
    #[serde(rename = "rawData")]
    raw_data: Option<RawData>,
}

#[derive(Deserialize)]
struct RawData {
    #[serde(rename = "containerName")]
    container_name: Option<String>,
}
// ANCHOR_END: structs

fn main() -> io::Result<()> {
    dotenvy::dotenv().ok();
    let mut terminal = ratatui::init();
    let app_result = App::default().run(&mut terminal);
    ratatui::restore();
    app_result
}

// ANCHOR: impl App
impl App {
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        if let Ok(token) = self.handshake() {
            self.session_token = Some(token);
            self.fetch_connections()
                .unwrap_or_else(|err| eprintln!("Error fetching connections: {}", err));
        } else {
            eprintln!("Error during handshake.");
        }

        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let items: Vec<ListItem> = if self.viewing_resources {
            self.resources
                .get(&self.current_server)
                .unwrap_or(&vec![])
                .iter()
                .map(|resource| ListItem::new(Line::from(format!("  {}", resource))))
                .collect()
        } else {
            self.servers
                .iter()
                .enumerate()
                .map(|(i, server)| {
                    let content = if i == self.selected_index {
                        Line::from(server.clone().bold().yellow())
                    } else {
                        Line::from(server.clone())
                    };
                    ListItem::new(content)
                })
                .collect()
        };

        let title = if self.viewing_resources {
            format!("Resources of {}", self.current_server)
        } else {
            "SSH Clients".to_string()
        };

        let block = Block::default().title(title).borders(Borders::ALL);
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
                if !self.viewing_resources && self.selected_index < self.servers.len() - 1 {
                    self.selected_index += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if !self.viewing_resources && self.selected_index > 0 {
                    self.selected_index -= 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Enter,
                kind: KeyEventKind::Press,
                ..
            }) => {
                if !self.viewing_resources {
                    self.current_server = self.servers[self.selected_index].clone();
                    self.viewing_resources = true;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('b'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.viewing_resources {
                    self.viewing_resources = false;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('q'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.exit = true;
            }
            _ => {}
        };
        Ok(())
    }

    fn handshake(&self) -> Result<String, reqwest::Error> {
        let api_key =
            std::env::var("XPIPE_API_KEY").expect("XPIPE_API_KEY environment variable not set");
        let client = Client::new();

        let response: HandshakeResponse = client
            .post(format!("{}/handshake", API_URL))
            .json(&json!({
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
                .json(&json!({
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
                .json(&json!({
                    "connections": connection_ids
                }))
                .send()?
                .json()?;

            for info in info_response.infos {
                if let Some(server_name) = info.name.first() {
                    self.servers.push(server_name.clone());
                    if let Some(raw_data) = info.raw_data {
                        if let Some(container_name) = raw_data.container_name {
                            self.resources
                                .entry(server_name.clone())
                                .or_default()
                                .push(container_name);
                        }
                    }
                }
            }

            self.servers.sort();
            self.servers.dedup();
        }

        Ok(())
    }
}
