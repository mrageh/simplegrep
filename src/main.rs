extern crate grep;
extern crate termcolor;
extern crate walkdir;

use std::env;
use std::error::Error;
use std::ffi::OsString;
use std::process;

use grep::cli;
use grep::printer::{SummaryBuilder, SummaryKind};
use grep::regex::RegexMatcher;
use grep::searcher::{BinaryDetection, SearcherBuilder};
use termcolor::ColorChoice;
use walkdir::WalkDir;

fn main() {
    if let Err(err) = try_main() {
        eprintln!("{}", err);
        process::exit(1);
    }
}

fn try_main() -> Result<(), Box<Error>> {
    let mut args: Vec<OsString> = env::args_os().collect();

    if args.len() < 2 {
        return Err("Usage: simplegrep <pattern> [<path> ...]".into());
    }

    if args.len() == 2 {
        args.push(OsString::from("./"));
    }

    println!("Command line args are: {:?}", &args);

    println!("Search term {:?}", cli::pattern_from_os(&args[1])?);
    println!("Search directory, default is current root{:?}", &args[2..]);

    search(cli::pattern_from_os(&args[1])?, &args[2..])
}

fn search(pattern: &str, paths: &[OsString]) -> Result<(), Box<Error>> {
    let regex_pattern_exact_match_word = format!(r#"(^|\W){}($|\W)"#, &pattern);
    let matcher = RegexMatcher::new_line_matcher(&regex_pattern_exact_match_word)?;

    let mut searcher = SearcherBuilder::new()
        .binary_detection(BinaryDetection::quit(b'\x00'))
        .line_number(false)
        .build();

    let mut printer = SummaryBuilder::new()
        .kind(SummaryKind::Count)
        //.max_matches(Some(10)) may possible need to limit matches, to return smaller list
        .build(cli::stdout(ColorChoice::Never));

    for path in paths {
        for result in WalkDir::new(path) {
            let dent = match result {
                Ok(dent) => dent,
                Err(err) => {
                    eprintln!("{}", err);
                    continue;
                }
            };

            if !dent.file_type().is_file() {
                continue;
            }

            let result = searcher.search_path(
                &matcher,
                dent.path(),
                printer.sink_with_path(&matcher, dent.path()),
            );

            if let Err(err) = result {
                eprintln!("{}: {}", dent.path().display(), err);
            }
        }
    }
    Ok(())
}
