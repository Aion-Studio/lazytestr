use notify::{Event, RecursiveMode, Result as NotifyResult, Watcher};
use std::error::Error;
use std::path::Path;
use std::sync::mpsc::{channel, Receiver};

pub struct FileWatcher {
    watcher: Box<dyn Watcher>,
    rx: Receiver<NotifyResult<Event>>,
}

impl FileWatcher {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let (tx, rx) = channel();
        let watcher = notify::recommended_watcher(move |res| {
            tx.send(res).unwrap();
        })?;

        Ok(FileWatcher {
            watcher: Box::new(watcher),
            rx,
        })
    }

    pub fn watch(&mut self, path: &Path) -> Result<(), notify::Error> {
        self.watcher.watch(path, RecursiveMode::Recursive)
    }

    pub fn check_events(&self) -> Option<Event> {
        match self.rx.try_recv() {
            Ok(Ok(event)) => Some(event),
            _ => None,
        }
    }

    pub fn should_run_tests(&self, event: &Event) -> bool {
        // You can customize this logic based on your needs
        matches!(event.kind, notify::EventKind::Modify(_))
    }
}

pub fn setup_file_watcher() -> Result<FileWatcher, Box<dyn Error>> {
    let mut watcher = FileWatcher::new()?;
    watcher.watch(Path::new("."))?;
    Ok(watcher)
}
