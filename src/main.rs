// ANCHOR: imports
use std::collections::HashMap;
use std::io;
use std::io::Write;
use std::process::{Command, Stdio};

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
    fn fzf_search(&mut self) {
        let input = self.servers.join("\n");

        let mut child = Command::new("fzf")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()
            .expect("Failed to spawn fzf process");

        // Write the server list to fzf's stdin
        if let Some(mut stdin) = child.stdin.take() {
            if let Err(e) = stdin.write_all(input.as_bytes()) {
                eprintln!("Failed to write to fzf stdin: {}", e);
                return;
            }
        }

        let output = child.wait_with_output().expect("Failed to read fzf output");

        if output.status.success() {
            if let Ok(selected) = String::from_utf8(output.stdout) {
                let selected = selected.trim();
                if let Some(connection_ids) = self.resources.get(selected) {
                    if let Some(connection_id) = connection_ids.first() {
                        self.open_terminal_session(connection_id);
                    } else {
                        eprintln!("No connection ID found for the selected server.");
                    }
                } else {
                    eprintln!("Selected server not found in resources.");
                }
            }
        } else {
            eprintln!("No selection made or fzf process failed.");
        }
    }
    fn draw(&self, frame: &mut Frame) {
        let items: Vec<ListItem> = self
            .servers
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
            .collect();

        let block = Block::default().title("SSH Clients").borders(Borders::ALL);
        let list = List::new(items).block(block);

        frame.render_widget(list, frame.area());

        // Display navigation instructions
        let instructions = Line::from(
            "Navigation: ↑/↓ or j/k to move | Enter to select | q to quit"
                .bold()
                .cyan(),
        );

        let area = frame.size();
        let instruction_area = ratatui::layout::Rect {
            x: area.x,
            y: area.y + area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        };

        frame.render_widget(instructions, instruction_area);
    }

    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(KeyEvent {
                code: KeyCode::Down | KeyCode::Char('j'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                if self.selected_index + 1 < self.servers.len() {
                    self.selected_index += 1;
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Up | KeyCode::Char('k'),
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
                let selected_server = self.servers[self.selected_index].clone();
                if let Some(connection_ids) = self.resources.get(&selected_server) {
                    if let Some(connection_id) = connection_ids.first() {
                        self.open_terminal_session(connection_id);
                    }
                } else {
                    eprintln!("No connection ID found for the selected server.");
                }
            }
            Event::Key(KeyEvent {
                code: KeyCode::Char('/'),
                kind: KeyEventKind::Press,
                ..
            }) => {
                self.fzf_search();
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
                    "typeFilter": "ssh"
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

            for (info, connection_id) in info_response.infos.into_iter().zip(connection_ids) {
                if let Some(server_name) = info.name.first() {
                    self.servers.push(server_name.clone());
                    self.resources
                        .entry(server_name.clone())
                        .or_default()
                        .push(connection_id); // Store the UUID directly
                }
            }

            self.servers.sort();
            self.servers.dedup();
        }

        Ok(())
    }

    fn open_terminal_session(&self, connection_uuid: &str) {
        if let Some(token) = &self.session_token {
            let client = Client::new();
            let payload = json!({
                "connection": connection_uuid,
                "directory": "/"
            });

            execute!(io::stdout(), Clear(ClearType::All)).unwrap();
            println!("Connecting to {}...", connection_uuid);

            match client
                .post(format!("{}/connection/terminal", API_URL))
                .bearer_auth(token)
                .json(&payload)
                .send()
            {
                Ok(resp) => {
                    if resp.status().is_success() {
                        println!(
                            "Terminal session opened successfully for: {}",
                            connection_uuid
                        );
                    } else {
                        let error_text =
                            resp.text().unwrap_or_else(|_| "Unknown error".to_string());
                        eprintln!("Error opening terminal session:\n{}", error_text);
                    }
                }
                Err(err) => {
                    eprintln!("Request failed: {}", err);
                }
            }

            println!("Press any key to return...");
            let _ = event::read();
        }
    }
}
