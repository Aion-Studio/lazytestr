mod app;
mod config;
mod file_watcher;
mod test_runner;
mod ui;
mod utils;

use app::App;
use config::setup_environment;
use file_watcher::setup_file_watcher;
use log::debug;
use std::error::Error;
use std::sync::mpsc::channel;
use test_runner::TestRunner;
use ui::{draw, restore_terminal, setup_terminal};

fn main() -> Result<(), Box<dyn Error>> {
    setup_environment()?;

    let mut terminal = setup_terminal()?;
    let mut app = App::new();
    let (output_tx, output_rx) = channel();
    let test_runner = TestRunner::new(output_tx);
    let file_watcher = setup_file_watcher()?;

    // Initial scan for tests
    test_runner.scan_for_tests(&mut app, ".")?;

    loop {
        // Handle test output
        while let Ok(output) = output_rx.try_recv() {
            app.add_test_output(&output);
            let height = terminal.size()?.height as usize;
            app.update_scroll(height);
        }

        // Draw UI
        terminal.draw(|f| draw(f, &mut app))?;

        // Handle input
        let should_run_action = app.handle_input()?;

        if app.should_quit {
            break;
        }

        if should_run_action {
            if app.active_pane == 1 {
                debug!("Running selected test");
                app.clear_test_output();
                test_runner.run_test(&app)?;
            } else if app.active_pane == 0 {
                debug!("Rescanning for tests");
                test_runner.scan_for_tests(&mut app, ".")?;
                debug!("Rescan complete. Found {} test files", app.test_info.len());
            }
        }

        // Handle file watcher events
        if app.watch_mode {
            if let Some(event) = file_watcher.check_events() {
                if file_watcher.should_run_tests(&event) {
                    debug!("File change detected, running tests");
                    test_runner.run_test(&app)?;
                }
            }
        }
    }

    restore_terminal(&mut terminal)?;
    Ok(())
}
