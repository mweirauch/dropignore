mod configuration;
mod dropbox;
mod matcher;
mod scanner;

use std::env;
use std::io;
use std::process;

use clap::{crate_description, crate_name, crate_version, App, AppSettings, Arg, SubCommand};
use fern::colors::{Color, ColoredLevelConfig};
use log::{error, LevelFilter};

use crate::configuration::Configuration;
use crate::dropbox::Dropbox;
use crate::matcher::Matcher;
use crate::scanner::Scanner;

fn main() {
    let dry_run_arg = Arg::with_name("dry-run")
        .help("Only show which actions would be performed. (default: false)")
        .short("n")
        .long("dry-run");

    let app = App::new(crate_name!())
        .version(crate_version!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandRequiredElseHelp)
        .setting(AppSettings::VersionlessSubcommands)
        .arg(
            clap::Arg::with_name("verbose")
                .help("Increases logging verbosity (up to '-vv')")
                .short("v")
                .long("verbose")
                .multiple(true)
                .global(true),
        )
        .subcommand(
            SubCommand::with_name("scan")
                .about("Scans the given directory recursively for ignore candidates")
                .arg(&dry_run_arg)
                .arg(
                    Arg::with_name("directory")
                        .help("The directory to scan (default: current working directory)"),
                ),
        )
        .subcommand(
            SubCommand::with_name("watch")
                .about("Watches the given directory recursively for ignore candidates")
                .arg(&dry_run_arg)
                .arg(
                    Arg::with_name("directory")
                        .help("The directory to watch (default: current working directory)"),
                ),
        );

    let matches = app.get_matches();

    init_logging(matches.occurrences_of("verbose"));

    let configuration = Configuration::load(crate_name!()).unwrap();

    match matches.subcommand() {
        ("scan", Some(subcommand_matches)) | ("watch", Some(subcommand_matches)) => {
            let directory = match subcommand_matches.value_of("directory") {
                Some(d) => Ok(std::path::PathBuf::from(d)),
                _ => env::current_dir(),
            };

            if directory.is_err() {
                error!(
                    "Couldn't determine directory to scan: {:?}",
                    directory.err().unwrap()
                );
                process::exit(1);
            }

            let matcher = Matcher::new(&configuration.matcher_config).unwrap();
            let dropbox = Dropbox::new().unwrap();
            let scanner = Scanner::new(matcher, dropbox).unwrap();

            let dry_run = subcommand_matches.is_present("dry-run");
            let watch = matches.subcommand_name().map(|n| n == "watch").unwrap();

            scanner.scan(directory.unwrap(), watch, dry_run);
        }
        _ => unreachable!(),
    }
}

fn init_logging(verbosity: u64) {
    let colors = ColoredLevelConfig::new()
        .trace(Color::Magenta)
        .debug(Color::Blue)
        .info(Color::Green)
        .warn(Color::Yellow)
        .error(Color::Red);

    let lc = fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{:5}][{:-13}] {}",
                colors.color(record.level()),
                record
                    .target()
                    .split("::")
                    .last()
                    .unwrap_or_else(|| record.target()),
                message,
            ))
        })
        .level(LevelFilter::Info)
        .chain(io::stdout());

    let level_filter = match verbosity {
        0 => LevelFilter::Info,
        1 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    lc.level_for(crate_name!(), level_filter)
        .apply()
        .expect("Failed to set up logging.");
}
