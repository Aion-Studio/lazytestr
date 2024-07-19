use crossterm::event::KeyCode;
use log::debug;
use std::error::Error;
use std::path::PathBuf;

pub struct TestInfo {
    pub path: PathBuf,
    pub tests: Vec<String>,
}

pub struct App {
    pub test_info: Vec<TestInfo>,
    pub selected_index: usize,
    pub active_pane: usize,
    pub selected_test: usize,
    pub test_output: String,
    pub watch_mode: bool,
    pub output_scroll: usize,
    pub output_height: usize,
    pub total_output_lines: usize,
    pub should_quit: bool,
}

impl App {
    pub fn new() -> Self {
        App {
            test_info: Vec::new(),
            selected_index: 0,
            active_pane: 0,
            selected_test: 0,
            test_output: String::new(),
            watch_mode: false,
            output_scroll: 0,
            should_quit: false,
            output_height: 0,
            total_output_lines: 0,
        }
    }

    pub fn handle_input(&mut self, key: KeyCode) -> Result<bool, Box<dyn Error>> {
        match key {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('w') => self.toggle_watch_mode(),
            KeyCode::Char('r') => return Ok(true), // Signal to rescan tests
            KeyCode::Enter => return Ok(true),     // Signal to run test
            //
            KeyCode::Char('h') => self.move_left(),
            KeyCode::Char('l') => self.move_right(),
            KeyCode::Char('j') | KeyCode::Char('k') => self.navigate_list(key),
            _ => {}
        }
        Ok(false)
    }

    pub fn update_output_height(&mut self, height: usize) {
        self.output_height = height;
        self.adjust_scroll();
    }

    pub fn scroll_to_bottom(&mut self) {
        self.output_scroll = self.total_output_lines.saturating_sub(self.output_height);
        self.adjust_scroll();
    }

    pub fn scroll_up(&mut self) {
        if self.output_scroll > 0 {
            self.output_scroll -= 1;
        }
    }

    pub fn scroll_down(&mut self) {
        self.output_scroll += 1;
        self.adjust_scroll();
    }

    fn adjust_scroll(&mut self) {
        let max_scroll = self.total_output_lines.saturating_sub(self.output_height);
        self.output_scroll = self.output_scroll.min(max_scroll);
    }

    fn move_left(&mut self) {
        self.active_pane = if self.active_pane == 0 {
            2
        } else {
            self.active_pane - 1
        };
    }

    fn move_right(&mut self) {
        self.active_pane = (self.active_pane + 1) % 3;
    }
    pub fn clear_test_output(&mut self) {
        self.test_output.clear();
        self.output_scroll = 0;
    }

    pub fn toggle_watch_mode(&mut self) {
        self.watch_mode = !self.watch_mode;
    }

    pub fn navigate_list(&mut self, key: KeyCode) {
        match self.active_pane {
            0 => {
                if key == KeyCode::Char('j') {
                    self.selected_index =
                        (self.selected_index + 1).min(self.test_info.len().saturating_sub(1));
                } else if key == KeyCode::Char('k') {
                    self.selected_index = self.selected_index.saturating_sub(1);
                }
            }

            1 => {
                if let Some(info) = self.test_info.get(self.selected_index) {
                    if key == KeyCode::Char('j') {
                        self.selected_test =
                            (self.selected_test + 1).min(info.tests.len().saturating_sub(1));
                    } else if key == KeyCode::Char('k') {
                        self.selected_test = self.selected_test.saturating_sub(1);
                    }
                }
            }

            2 => {
                let total_lines = self.test_output.lines().count();
                let page_size = self.output_height.saturating_sub(2); // Subtract 2 for borders
                match key {
                    KeyCode::Char('j') => self.scroll_down(),
                    KeyCode::Char('k') => self.scroll_up(),
                    KeyCode::Char('d') => {
                        self.output_scroll =
                            (self.output_scroll + page_size).min(total_lines.saturating_sub(1));
                        debug!("Page down. New scroll position: {}", self.output_scroll);
                    }
                    KeyCode::Char('u') => {
                        self.output_scroll = self.output_scroll.saturating_sub(page_size);
                        debug!("Page up. New scroll position: {}", self.output_scroll);
                    }
                    _ => {}
                }
            }

            _ => {}
        }
    }

    pub fn add_test_output(&mut self, new_output: &str) {
        self.test_output.push_str(new_output);
        // Limit the total number of lines to prevent excessive memory usage
        let max_lines = 1000;
        let lines: Vec<&str> = self.test_output.lines().collect();
        if lines.len() > max_lines {
            self.test_output = lines[lines.len() - max_lines..].join("\n");
        }

        self.total_output_lines = self.test_output.lines().count();
        self.scroll_to_bottom();
    }

    pub fn update_scroll(&mut self) {
        let total_lines = self.test_output.lines().count();
        if total_lines > self.output_height {
            self.output_scroll = self.output_scroll.min(total_lines - 1);
        } else {
            self.output_scroll = 0;
        }
    }
}
