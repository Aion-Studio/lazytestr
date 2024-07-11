use colored::*;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};

use ignore::WalkBuilder;
use notify::{
    Config as NotifyConfig, RecommendedWatcher, RecursiveMode, Result as NotifyResult, Watcher,
};
use regex::Regex;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::channel;
use std::thread;
use std::{error::Error, path::PathBuf};
use std::{fs, sync::mpsc::Receiver};
use std::{path::Path, sync::mpsc::Sender};

use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

struct TestInfo {
    path: PathBuf,
    tests: Vec<String>,
}

struct App {
    test_info: Vec<TestInfo>,
    selected_index: usize,
    active_pane: usize,
    selected_test: usize,
    debug_logs: Vec<String>,
    test_output: String,
    watch_mode: bool,
    debug_scroll: usize,

    test_output_rx: Receiver<String>,
    test_output_tx: Sender<String>,
    gitignore: ignore::gitignore::Gitignore,
}

impl App {
    fn new() -> Result<Self, Box<dyn Error>> {
        let (test_output_tx, test_output_rx) = channel();
        let mut app = App {
            test_info: Vec::new(),
            selected_index: 0,
            debug_scroll: 0,
            active_pane: 0,
            selected_test: 0,
            test_output_rx,
            test_output_tx,
            debug_logs: Vec::new(),
            test_output: String::new(),
            watch_mode: false,
            gitignore: ignore::gitignore::Gitignore::new(".").0,
        };
        app.scan_for_tests(".")?;
        Ok(app)
    }

    fn scroll_debug(&mut self, amount: i32) {
        if amount < 0 {
            self.debug_scroll = self.debug_scroll.saturating_sub(amount.abs() as usize);
        } else {
            self.debug_scroll = self.debug_scroll.saturating_add(amount as usize);
        }
    }

    fn scan_for_tests<P: AsRef<Path>>(&mut self, dir: P) -> Result<(), Box<dyn Error>> {
        let test_regex =
            Regex::new(r"(?m)#\[(cfg\(test\)|test|(tokio::)?test)\][\s\n]*(async\s+)?fn\s+(\w+)")?;
        let walker = WalkBuilder::new(dir)
            .hidden(false) // Show hidden files
            .git_ignore(true) // Use .gitignore
            .build();

        for entry in walker {
            match entry {
                Ok(entry) => {
                    let path = entry.path();
                    self.log(&format!("Scanning: {}", path.display()));

                    if path.is_file() && path.extension().map_or(false, |ext| ext == "rs") {
                        if !self.gitignore.matched(path, path.is_dir()).is_ignore() {
                            let content = fs::read_to_string(path)?;
                            let tests: Vec<String> = test_regex
                                .captures_iter(&content)
                                .filter_map(|cap| cap.get(4).map(|m| m.as_str().to_string()))
                                .collect();

                            if !tests.is_empty() {
                                self.log(&format!(
                                    "Found {} tests in {}",
                                    tests.len(),
                                    path.display()
                                ));
                                self.test_info.push(TestInfo {
                                    path: path.to_path_buf(),
                                    tests,
                                });
                            } else {
                                self.log(&format!("No tests found in {}", path.display()));
                            }
                        } else {
                            self.log(&format!("Ignoring file: {}", path.display()));
                        }
                    }
                }
                Err(err) => self.log(&format!("Error accessing entry: {}", err)),
            }
        }

        Ok(())
    }

    fn run_test(&mut self) -> Result<(), Box<dyn Error>> {
        if self.active_pane == 1 {
            if let Some(info) = self.test_info.get(self.selected_index) {
                if let Some(test_name) = info.tests.get(self.selected_test) {
                    self.test_output.clear();
                    let tx = self.test_output_tx.clone();
                    let test_name = test_name.clone();

                    thread::spawn(move || {
                        let mut cmd = Command::new("cargo")
                            .args(&["test", &test_name, "--", "--nocapture"])
                            .env("RUSTFLAGS", "-Awarnings")
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to start command");

                        let stdout = cmd.stdout.take().unwrap();
                        let stderr = cmd.stderr.take().unwrap();

                        let stdout_reader = BufReader::new(stdout);
                        let stderr_reader = BufReader::new(stderr);

                        tx.send(format_test_output(&format!(
                            "Running test: {}\n",
                            test_name
                        )))
                        .unwrap();

                        for line in stdout_reader.lines().chain(stderr_reader.lines()) {
                            if let Ok(line) = line {
                                tx.send(format_test_output(&format!("{}\n", line))).unwrap();
                            }
                        }

                        let status = cmd.wait().expect("Failed to wait on child");
                        tx.send(format_test_output(&format!(
                            "Test finished with status: {:?}\n",
                            status
                        )))
                        .unwrap();
                    });
                }
            }
        }
        Ok(())
    }

    fn toggle_watch_mode(&mut self) {
        self.watch_mode = !self.watch_mode;
    }

    fn log(&mut self, message: &str) {
        self.debug_logs.push(message.to_string());
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new()?;

    let mut redraw = true;
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(
        move |res: NotifyResult<notify::Event>| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )?;

    watcher.watch(Path::new("."), RecursiveMode::Recursive)?;

    loop {
        let mut received_output = false;
        while let Ok(output) = app.test_output_rx.try_recv() {
            app.test_output.push_str(&output); // Output is already formatted
            received_output = true;
        }
        if received_output {
            redraw = true;
        }

        // Handle file system events (for watch mode)
        if app.watch_mode {
            if let Ok(event) = rx.try_recv() {
                match event.kind {
                    notify::EventKind::Modify(_) => {
                        app.run_test()?;
                        redraw = true;
                    }
                    _ => {}
                }
            }
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('w') => {
                        app.toggle_watch_mode();
                        if app.watch_mode {
                            app.run_test()?;
                        }

                        redraw = true;
                    }
                    KeyCode::Enter => {
                        app.run_test()?;
                        redraw = true;
                    }

                    KeyCode::Char('q') => break,
                    KeyCode::Char('h') => {
                        app.active_pane = (app.active_pane + 2) % 3;
                        redraw = true;
                    }
                    KeyCode::Char('l') => {
                        app.active_pane = (app.active_pane + 1) % 3;
                        redraw = true;
                    }
                    KeyCode::Char('j') => match app.active_pane {
                        0 => {
                            redraw = true;
                            app.selected_index =
                                (app.selected_index + 1).min(app.test_info.len().saturating_sub(1))
                        }
                        1 => {
                            redraw = true;
                            if let Some(info) = app.test_info.get(app.selected_index) {
                                app.selected_test =
                                    (app.selected_test + 1).min(info.tests.len().saturating_sub(1));
                            }
                        }
                        2 => {
                            redraw = true;
                            app.scroll_debug(1)
                        }
                        _ => {}
                    },
                    KeyCode::Char('k') => {
                        redraw = true;
                        match app.active_pane {
                            0 => app.selected_index = app.selected_index.saturating_sub(1),
                            1 => app.selected_test = app.selected_test.saturating_sub(1),
                            2 => app.scroll_debug(-1),
                            _ => {}
                        }
                    }
                    KeyCode::Char('y') => {
                        //todo
                    }

                    _ => {}
                }
            }
        }
        if redraw {
            terminal.draw(|f| draw_ui(f, &app))?;
            redraw = false;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn draw_ui(f: &mut tui::Frame<CrosstermBackend<std::io::Stdout>>, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    let left_items: Vec<ListItem> = app
        .test_info
        .iter()
        .enumerate()
        .map(|(i, info)| {
            let style = if i == app.selected_index && app.active_pane == 0 {
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default()
            };
            ListItem::new(Spans::from(vec![Span::styled(
                info.path.to_string_lossy(),
                style,
            )]))
        })
        .collect();

    let left_list = List::new(left_items)
        .block(Block::default().title("Test Files").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(left_list, left_chunks[0]);

    let right_items: Vec<ListItem> = if let Some(info) = app.test_info.get(app.selected_index) {
        info.tests
            .iter()
            .enumerate()
            .map(|(i, test)| {
                let style = if i == app.selected_test && app.active_pane == 1 {
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                };
                ListItem::new(Spans::from(vec![Span::styled(test, style)]))
            })
            .collect()
    } else {
        vec![ListItem::new("No tests found")]
    };

    let right_list = List::new(right_items)
        .block(Block::default().title("Tests").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD));

    f.render_widget(right_list, left_chunks[1]);

    let debug_text = app.debug_logs.join("\n");

    let bottom_text = if !app.test_output.is_empty() {
        app.test_output.as_str()
    } else {
        debug_text.as_str()
    };

    let bottom_paragraph = Paragraph::new(bottom_text)
        .block(
            Block::default()
                .title(if !app.test_output.is_empty() {
                    if app.watch_mode {
                        "Test Output (Watch Mode)"
                    } else {
                        "Test Output"
                    }
                } else {
                    "Debug Log"
                })
                .borders(Borders::ALL),
        )
        .wrap(tui::widgets::Wrap { trim: true })
        .scroll((app.debug_scroll as u16, 0));

    f.render_widget(bottom_paragraph, chunks[1]);

    // Highlight the active pane
    let highlight_block = Block::default()
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Yellow));
    match app.active_pane {
        0 => f.render_widget(highlight_block, left_chunks[0]),
        1 => f.render_widget(highlight_block, left_chunks[1]),
        2 => f.render_widget(highlight_block, chunks[1]),
        _ => {}
    }
}

fn format_test_output(output: &str) -> String {
    let lines: Vec<&str> = output.lines().collect();
    let mut formatted_output = String::new();
    let mut in_test_result = false;

    for line in lines {
        if line.starts_with("Running test:") {
            formatted_output.push_str(&format!("\n{}\n", line.bright_green()));
        } else if line.contains("test result:") {
            in_test_result = true;
            formatted_output.push_str(&format!("\n{}\n", line.yellow()));
        } else if line.starts_with("stderr:") || line.starts_with("stdout:") {
            let (prefix, content) = line.split_at(7);
            formatted_output.push_str(&format!("{} {}\n", prefix.bright_blue(), content));
        } else if line.contains("FAILED") {
            formatted_output.push_str(&format!("{}\n", line.bright_red()));
        } else if in_test_result {
            formatted_output.push_str(&format!("{}\n", line.yellow()));
        } else {
            formatted_output.push_str(&format!("{}\n", line));
        }
    }

    formatted_output
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_find_tests_in_file() {
        assert!("testing hello{".contains("testing"));
    }
}
