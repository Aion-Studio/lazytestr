use crate::app::App;
use ansi_parser::{AnsiParser, AnsiSequence, Output};
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use log::debug;
use std::io;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame, Terminal,
};

pub fn setup_terminal() -> io::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

pub fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()
}

pub fn draw(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)].as_ref())
        .split(f.size());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
        .split(chunks[0]);

    draw_test_files(f, app, left_chunks[0]);
    draw_tests(f, app, left_chunks[1]);
    draw_test_output(f, app, chunks[1]);
}

fn draw_test_files(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &App, area: Rect) {
    let items: Vec<ListItem> = app
        .test_info
        .iter()
        .enumerate()
        .map(|(i, info)| {
            let style = if i == app.selected_index && app.active_pane == 0 {
                Style::default().fg(Color::Black).bg(Color::LightBlue)
            } else {
                Style::default().fg(Color::White)
            };
            ListItem::new(Spans::from(vec![Span::styled(
                info.path.to_string_lossy(),
                style,
            )]))
        })
        .collect();

    let list = List::new(items)
        .block(Block::default().title("Test Files").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_widget(list, area);
}

fn draw_tests(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &App, area: Rect) {
    let items: Vec<ListItem> = if let Some(info) = app.test_info.get(app.selected_index) {
        info.tests
            .iter()
            .enumerate()
            .map(|(i, test)| {
                let style = if i == app.selected_test && app.active_pane == 1 {
                    Style::default().fg(Color::Black).bg(Color::LightBlue)
                } else {
                    Style::default().fg(Color::White)
                };
                ListItem::new(Spans::from(vec![Span::styled(test, style)]))
            })
            .collect()
    } else {
        vec![ListItem::new("No tests found")]
    };

    let list = List::new(items)
        .block(Block::default().title("Tests").borders(Borders::ALL))
        .highlight_style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_symbol("> ");

    f.render_widget(list, area);
}

fn draw_test_output(f: &mut Frame<CrosstermBackend<io::Stdout>>, app: &App, area: Rect) {
    let visible_height = area.height as usize - 2; // Subtract 2 for the border
    let total_lines = app.test_output.lines().count();

    let start_line = app
        .output_scroll
        .min(total_lines.saturating_sub(visible_height));

    debug!(
        "Drawing output: scroll={}, start_line={}, total_lines={}, visible_height={}",
        app.output_scroll, start_line, total_lines, visible_height
    );

    let output_lines: Vec<Spans> = app
        .test_output
        .lines()
        .skip(start_line)
        .take(visible_height)
        .enumerate()
        .map(|(i, line)| {
            debug!("Line {}: {}", start_line + i, line);
            let mut spans = Vec::new();
            let mut current_style = Style::default();

            for output in line.ansi_parse() {
                match output {
                    Output::TextBlock(text) => {
                        spans.push(Span::styled(text.to_string(), current_style));
                    }
                    Output::Escape(sequence) => {
                        if let AnsiSequence::SetGraphicsMode(modes) = sequence {
                            for mode in modes {
                                match mode {
                                    0 => current_style = Style::default(),
                                    1 => current_style = current_style.add_modifier(Modifier::BOLD),
                                    30..=37 => {
                                        let color = match mode - 30 {
                                            0 => Color::Black,
                                            1 => Color::Red,
                                            2 => Color::Green,
                                            3 => Color::Yellow,
                                            4 => Color::Blue,
                                            5 => Color::Magenta,
                                            6 => Color::Cyan,
                                            7 => Color::White,
                                            _ => unreachable!(),
                                        };
                                        current_style = current_style.fg(color);
                                    }
                                    40..=47 => {
                                        let color = match mode - 40 {
                                            0 => Color::Black,
                                            1 => Color::Red,
                                            2 => Color::Green,
                                            3 => Color::Yellow,
                                            4 => Color::Blue,
                                            5 => Color::Magenta,
                                            6 => Color::Cyan,
                                            7 => Color::White,
                                            _ => unreachable!(),
                                        };
                                        current_style = current_style.bg(color);
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            Spans::from(spans)
        })
        .collect();

    let block = Block::default()
        .title(format!(
            "Test Output (Scroll: {}/{})",
            app.output_scroll, total_lines
        ))
        .borders(Borders::ALL)
        .border_style(Style::default().fg(if app.active_pane == 2 {
            Color::Rgb(255, 165, 0)
        } else {
            Color::White
        }));

    let output_paragraph = Paragraph::new(output_lines)
        .block(block)
        .wrap(Wrap { trim: false });

    f.render_widget(output_paragraph, area);
}
