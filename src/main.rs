#[macro_use]
extern crate clap;
extern crate chrono;
extern crate crc32fast;

#[cfg(feature = "mmap")]
extern crate memmap;

use std::io::Write;
use std::path::Path;

use clap::Command;
use clap::Arg;
use clap::ArgAction;

use cksfv::Config;
use cksfv::Output;
use cksfv::cksfv;
use cksfv::newsfv;

fn main() -> ! {
    // read CLI arguments
    let mut command = Command::new("cksfv.rs")
        .version(crate_version!())
        .author(crate_authors!("\n"))
        .override_usage("cksfv [-bciq] [-C dir] [-f file] [-g path] [file ...]")
        .arg(
            Arg::new("b")
                .short('b')
                .help("Print only the basename when creating an sfv")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("c")
                .short('c')
                .help("Use stdout for printing progress and final resolution")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("C")
                .short('C')
                .value_name("dir")
                .help("Change to directory for processing")
                
                .conflicts_with("g"),
        )
        .arg(
            Arg::new("f")
                .short('f')
                .value_name("file")
                .help("Verify the sfv file")
                .action(ArgAction::Append)
                .number_of_values(1),
        )
        .arg(
            Arg::new("g")
                .short('g')
                .value_name("path")
                .help("Go to the path name directory and verify the sfv file")
                .action(ArgAction::Append)
                .conflicts_with("C")
                .number_of_values(1),
        )
        .arg(
            Arg::new("i")
                .short('i')
                .help("Ignore case on filenames")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("L")
                .short('L')
                .help("Follow symlinks in recursive mode")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("q")
                .short('q')
                .help("Quiet, only prints errors messages")
                .action(ArgAction::SetTrue)
        )
        .arg(
            Arg::new("r")
                .short('r')
                .help("Recursively check .sfv files in subdirectories")
                .action(ArgAction::SetTrue)
                .conflicts_with("f")
                .conflicts_with("g"),
        )
        .arg(
            Arg::new("s")
                .short('s')
                .help("Replace backslashes with slashes on filenames")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("v")
                .short('v')
                .help("Verbose, by default this option is on")
                .action(ArgAction::SetTrue),
        )
        .arg(
            Arg::new("file")
                .index(1)
                .value_name("file")
                .action(ArgAction::Append),
        );

    let matches = command
        .clone()
        .get_matches();

    // build config
    let mut config = Config::default();
    config.set_quiet(matches.get_flag("q"));

    // check files recursively
    if matches.get_flag("r") {
        // get the base directory
        let cwd = std::env::current_dir().unwrap();

        // get the files to check if any
        let files = matches.get_many::<&str>("file")
            .map(|values| values.map(Path::new));

        // assign the right output stream
        if matches.get_flag("q") {
            config.set_stderr(Output::devnull());
            config.set_stdout(Output::stderr());
        } else if !matches.get_flag("c") {
            config.set_stdout(Output::stderr());
        }


        // recursively traverse the directory
        let mut retcode = 0;
        let it = walkdir::WalkDir::new(&cwd)
                .follow_links(matches.get_flag("L"))
                .sort_by(|a, b| a.depth().cmp(&b.depth()) );
        for result in it {
            if let Ok(entry) = result {
                if entry.path().extension().map(|x| x == "sfv").unwrap_or(false) {
                    let workdir = entry.path().parent().unwrap();
                    let sfv = entry.path().strip_prefix(workdir).unwrap();
                    write!(config.stderr_mut(), "Entering directory: {}\n", workdir.display()).unwrap();
                    std::env::set_current_dir(workdir).unwrap();
                    retcode *= 1 - cksfv(sfv, None, config.clone(), files.clone()).unwrap() as i32;
                }
            }
        }

        std::env::set_current_dir(&cwd).unwrap();
        std::process::exit(retcode);
    }

    // check files using the given SFV listing
    if matches.contains_id("g") || matches.contains_id("f") {
        // get the path to the SFV listing and the working directory
        let sfv = matches.get_many::<String>("g").or(matches.get_many::<String>("f")).unwrap().last().map(Path::new).unwrap();
        let workdir = if matches.contains_id("g") {
            sfv.parent()
        } else {
            matches.get_one::<String>("C").map(Path::new)
        };

        // get the files to check if any
        let files = matches.get_many::<String>("file")
            .map(|values| values.map(Path::new));

        // assign the right output stream
        if matches.get_flag("q") {
            config.set_stderr(Output::devnull());
            config.set_stdout(Output::stderr());
        } else if !matches.get_flag("c") {
            config.set_stdout(Output::stderr());
        }

        // run the operation
        let result = cksfv(sfv, workdir, config, files).unwrap();
        std::process::exit(!result as i32);
    }

    // generate a new sfv file if given files as input
    if let Some(files) = matches.get_many::<String>("file") {
        config.set_print_basename(matches.get_flag("b"));
        let result = newsfv(files.map(Path::new), config).unwrap();
        std::process::exit(!result as i32);
    }

    // otherwise is no operation given exit with EINVAL
    match command.print_help() {
        Ok(_) => std::process::exit(22),
        Err(e) => std::process::exit(e.raw_os_error().unwrap_or(1))
    }
}