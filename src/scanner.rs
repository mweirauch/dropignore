use std::path::PathBuf;

use walkdir::{DirEntry, WalkDir};

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
        Ok(Scanner {
            dropbox: dropbox,
            matcher: matcher,
        })
    }

    pub fn scan(&self, directory_in: PathBuf, dry_run: bool) {
        let directory = directory_in.canonicalize().unwrap();
        println!("SCAN    {:?}", directory);

        let mut scanner_stats: ScannerStats = Default::default();

        let walker = WalkDir::new(directory).into_iter();
        for _entry in
            walker.filter_entry(|e| Self::handle_entry(self, e, dry_run, &mut scanner_stats))
        {
        }

        println!(
            "Finished with {} known and {} new ignores.",
            scanner_stats.known_ignores, scanner_stats.new_ignores
        );
    }

    fn handle_entry(
        &self,
        entry: &DirEntry,
        dry_run: bool,
        scanner_stats: &mut ScannerStats,
    ) -> bool {
        let path = entry.path();

        if entry
            .file_name()
            .to_str()
            .map(|s| s.starts_with("."))
            .unwrap_or(false)
        {
            // println!("SKIPDOT {:?}", path);
            return false;
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
            } else {
                match self.dropbox.ignore(path) {
                    true => println!("IGNORED {:?}", path),
                    false => eprintln!("Failed ignoring {:?}", path),
                }
            }

            scanner_stats.new_ignores += 1;
            return false;
        }

        true
    }
}
