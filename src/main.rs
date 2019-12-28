#[macro_use]
extern crate clap;
extern crate chrono;
extern crate crc32fast;

#[cfg(feature = "mmap")]
extern crate memmap;

use std::collections::HashMap;
use std::fs::File;
use std::fmt::Debug;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::io::Error as IoError;
use std::iter::IntoIterator;
use std::path::Path;

use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;
use chrono::Timelike;
use clap::App;
use clap::Arg;
use crc32fast::Hasher;

use cksfv::Config;
use cksfv::Output;
use cksfv::cksfv;
use cksfv::newsfv;

fn main() -> ! {
    // read CLI arguments
    let matches = App::new("cksfv.rs")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .usage("cksfv [-bciq] [-C dir] [-f file] [-g path] [file ...]")
        .arg(
            Arg::with_name("b")
                .short("b")
                .help("Print only the basename when creating an sfv")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("c")
                .short("c")
                .help("Use stdout for printing progress and final resolution")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("C")
                .short("C")
                .value_name("dir")
                .help("Change to directory for processing")
                .takes_value(true)
                .conflicts_with("g"),
        )
        .arg(
            Arg::with_name("f")
                .short("f")
                .value_name("file")
                .help("Verify the sfv file")
                .takes_value(true)
                .multiple(true)
        )
        .arg(
            Arg::with_name("g")
                .short("g")
                .value_name("path")
                .help("Go to the path name directory and verify the sfv file")
                .takes_value(true)
                .multiple(true)
                .conflicts_with("C"),
        )
        .arg(
            Arg::with_name("i")
                .short("i")
                .help("Ignore case on filenames")
                .takes_value(false),
        )
        .arg(
            Arg::with_name("L")
                .short("L")
                .help("Follow symlinks in recursive mode"),
        )
        .arg(
            Arg::with_name("q")
                .short("q")
                .help("Quiet, only prints errors messages"),
        )
        .arg(
            Arg::with_name("r")
                .short("r")
                .help("Recursively check .sfv files in subdirectories")
                .conflicts_with("f")
                .conflicts_with("g"),
        )
        .arg(
            Arg::with_name("s")
                .short("s")
                .help("Replace backslashes with slashes on filenames"),
        )
        .arg(
            Arg::with_name("v")
                .short("v")
                .help("Verbose, by default this option is on"),
        )
        .arg(
            Arg::with_name("file")
                .index(1)
                .value_name("file")
                .multiple(true)
        )
        .get_matches();

    // build config
    let mut config = Config::default();

    // check files using the given SFV listing
    if matches.is_present("g") || matches.is_present("f") {
        // get the path to the SFV listing and the working directory

        let sfv = matches.values_of("g").or(matches.values_of("f")).unwrap().last().map(Path::new).unwrap();
        let workdir = if matches.is_present("g") {
            sfv.parent()
        } else {
            matches.value_of("C").map(Path::new)
        };

        // assign the right output stream
        if matches.is_present("q") {
            config.set_stderr(Output::devnull());
            config.set_stdout(Output::stderr());
        } else if !matches.is_present("c") {
            config.set_stdout(Output::stderr());
        }

        // run the operation
        let result = cksfv(sfv, workdir, config).unwrap();
        std::process::exit(!result as i32);
    }

    // generate a new sfv file if given files as input
    if let Some(files) = matches.values_of("file") {
        config.set_print_basename(matches.is_present("b"));
        let result = newsfv(files.map(Path::new), config).unwrap();
        std::process::exit(!result as i32);
    }

    // otherwise is no operation given exit with EINVAL
    println!("{}", matches.usage());
    std::process::exit(22);
}
