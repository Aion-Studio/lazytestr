use crate::app::{App, TestInfo};
use crate::utils::scan_for_tests;
use std::error::Error;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::sync::mpsc::Sender;
use std::thread;

pub struct TestRunner {
    output_tx: Sender<String>,
    use_nextest: bool,
}

impl TestRunner {
    pub fn new(output_tx: Sender<String>) -> Self {
        let use_nextest = Self::check_nextest_installed();
        TestRunner {
            output_tx,
            use_nextest,
        }
    }

    fn check_nextest_installed() -> bool {
        Command::new("cargo")
            .args(&["nextest", "--version"])
            .output()
            .map(|output| output.status.success())
            .unwrap_or(false)
    }
    pub fn run_test(&self, app: &App) -> Result<(), Box<dyn Error>> {
        if let Some(info) = app.test_info.get(app.selected_index) {
            if let Some(test_name) = info.tests.get(app.selected_test) {
                let test_name = test_name.clone();
                let tx = self.output_tx.clone();
                let path = info.path.clone();

                let use_nextest = self.use_nextest;
                thread::spawn(move || {
                    tx.send(format!("Running test: {}\n", test_name)).unwrap();

                    let mut cmd = if use_nextest {
                        Command::new("cargo")
                            .current_dir(path.parent().unwrap())
                            .args(&["nextest", "run", &test_name, "--no-capture"])
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to start nextest command")
                    } else {
                        Command::new("cargo")
                            .current_dir(path.parent().unwrap())
                            .args(&["test", &test_name, "--", "--nocapture"])
                            .stdout(Stdio::piped())
                            .stderr(Stdio::piped())
                            .spawn()
                            .expect("Failed to start test command")
                    };

                    let stdout = cmd.stdout.take().unwrap();
                    let stderr = cmd.stderr.take().unwrap();

                    let tx_clone = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stdout);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                tx_clone.send(line + "\n").unwrap();
                            }
                        }
                    });

                    let tx_clone = tx.clone();
                    thread::spawn(move || {
                        let reader = BufReader::new(stderr);
                        for line in reader.lines() {
                            if let Ok(line) = line {
                                tx_clone.send(line + "\n").unwrap();
                            }
                        }
                    });

                    let status = cmd.wait().expect("Failed to wait on child");
                    tx.send(format!("Test finished with status: {:?}\n", status))
                        .unwrap();
                });
            }
        }
        Ok(())
    }

    pub fn scan_for_tests(&self, app: &mut App, dir: &str) -> Result<(), Box<dyn Error>> {
        self.output_tx
            .send("Rescanning for tests...\n".to_string())?;

        app.test_info.clear();
        app.selected_index = 0;
        app.selected_test = 0;

        let test_info = scan_for_tests(dir)?;
        for (path, tests) in test_info {
            app.test_info.push(TestInfo { path, tests });
        }

        self.output_tx.send(format!(
            "Rescan complete. Found {} test files.\n",
            app.test_info.len()
        ))?;

        Ok(())
    }
}
