use std::path::{Path, PathBuf};
use std::process;
use std::sync::mpsc::channel;
use std::time::Duration;

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use walkdir::WalkDir;

use crate::dropbox::Dropbox;
use crate::matcher::Matcher;

pub struct Scanner {
    dropbox: Dropbox,
    matcher: Matcher,
}

#[derive(Default)]
struct ScannerStats {
    known_ignores: u32,
    new_ignores: u32,
}

impl Scanner {
    pub fn new(matcher: Matcher, dropbox: Dropbox) -> Result<Self, String> {
        Ok(Scanner { dropbox, matcher })
    }

    pub fn scan(&self, directory_in: PathBuf, watch: bool, dry_run: bool) {
        let directory = directory_in.canonicalize().unwrap();
        println!("{:8}{:?}", if watch { "WATCH" } else { "SCAN" }, directory);

        let mut scanner_stats: ScannerStats = Default::default();

        // even if we are watching, always perform a scan before
        let walker = WalkDir::new(&directory).into_iter();
        for _entry in
            walker.filter_entry(|e| Self::handle_entry(self, e.path(), dry_run, &mut scanner_stats))
        {
        }

        if watch {
            let (sender, receiver) = channel();

            let mut watcher: RecommendedWatcher =
                Watcher::new(sender, Duration::from_secs(2)).unwrap();

            let watch_result = watcher.watch(&directory, RecursiveMode::Recursive);
            if watch_result.is_err() {
                eprintln!(
                    "Failed watching {:?}: {:?}",
                    directory,
                    watch_result.err().unwrap()
                );
                process::exit(1);
            }

            loop {
                match receiver.recv() {
                    Ok(event) => {
                        // Chmod: creating multiple dirs at once (e.g. `mkdir -p`) first is `Create`, others `Chmod`
                        match event {
                            DebouncedEvent::Create(p)
                            | DebouncedEvent::Chmod(p)
                            | DebouncedEvent::Rename(_, p) => {
                                Self::handle_entry(self, p.as_path(), dry_run, &mut scanner_stats);
                            }
                            _ => {}
                        }
                    }
                    Err(error) => eprintln!("Watch error {:?}", error),
                }
            }
        }

        println!(
            "Finished with {} known and {} new ignores.",
            scanner_stats.known_ignores, scanner_stats.new_ignores
        );
    }

    fn handle_entry(&self, path: &Path, dry_run: bool, scanner_stats: &mut ScannerStats) -> bool {
        if let Some(file_name) = path.file_name() {
            if file_name
                .to_str()
                .map(|s| s.starts_with('.'))
                .unwrap_or(false)
            {
                // println!("SKIPDOT {:?}", path);
                return false;
            }
        }

        let matches = self.matcher.matches(path.to_str().unwrap().to_string());
        if matches {
            if self.dropbox.is_ignored(path) {
                // println!("KNOWN   {:?}", path);
                scanner_stats.known_ignores += 1;
                return false;
            }

            if dry_run {
                println!("IGNORE  {:?}", path);
            } else if self.dropbox.ignore(path) {
                println!("IGNORED {:?}", path);
            } else {
                eprintln!("Failed ignoring {:?}", path);
            }

            scanner_stats.new_ignores += 1;
            return false;
        }

        true
    }
}
