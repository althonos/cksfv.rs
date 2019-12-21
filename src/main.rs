#[macro_use]
extern crate clap;
extern crate chrono;
extern crate crc32fast;

#[cfg(feature = "mmap")]
extern crate memmap;

use std::fs::File;
use std::io::Read;
use std::io::BufRead;
use std::io::BufReader;
use std::iter::IntoIterator;
use std::path::Path;

use chrono::DateTime;
use chrono::Datelike;
use chrono::Local;
use chrono::Timelike;
use clap::App;
use clap::Arg;
use crc32fast::Hasher;

/// Use a 64k buffer size for better performance.
const DEFAULT_BUFFER_SIZE: usize = 65536;

// ---------------------------------------------------------------------------

/// Given a path to a file, attempt to compute its CRC32 hash.
fn compute_crc32(file: &Path) -> Result<u32, std::io::Error> {
    // check the file is not a directory (File::open is fine opening
    // a directory and will just read it as an empty file, but we want
    // a hard error)
    if file.is_dir() {
        return Err(std::io::Error::from_raw_os_error(21));
    }

    // open the file and compute the hash
    File::open(&file).and_then(compute_crc32_inner)
}

/// Compute a CRC32 from a file content using `mmap`.
#[cfg(feature = "mmap")]
fn compute_crc32_inner(mut file: File) -> Result<u32, std::io::Error> {
    let mut hasher = Hasher::new();
    let mmap = unsafe { memmap::MmapOptions::new().map(&file)? };
    hasher.update(&mmap[..]);
    Ok(hasher.finalize())
}

/// Compute a CRC32 from a file content without using `mmap`.
#[cfg(not(feature = "mmap"))]
fn compute_crc32_inner(mut file: File) -> Result<u32, std::io::Error> {
    let mut hasher = Hasher::new();
    let mut buffer = [0; DEFAULT_BUFFER_SIZE];
    loop {
        let n = file.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        hasher.update(&mut buffer[..n]);
    }
    Ok(hasher.finalize())
}

// ---------------------------------------------------------------------------

/// Generate a new SFV listing from a list of files.
fn newsfv<'a, F>(files: F, basename: bool) -> bool
where
    F: IntoIterator<Item = &'a Path>,
{
    // collect the files
    let files: Vec<&Path> = files.into_iter().collect();

    // generate the headers from the files that where found
    let now: DateTime<Local> = Local::now();
    println!(
        "; Generated by cksfv.rs v{} on {:04}-{:02}-{:02} at {:02}:{:02}.{:02}",
        crate_version!(),
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    );
    println!("; Project web site: {}", env!("CARGO_PKG_REPOSITORY"));
    println!(";");
    for file in files.iter().filter(|p| p.is_file()) {
        if let Ok(metadata) = std::fs::metadata(file) {
            let mtime: DateTime<Local> = From::from(metadata.modified().unwrap());
            println!(
                "; {:>12}  {:02}:{:02}.{:02} {:04}-{:02}-{:02} {}",
                metadata.len(),
                mtime.hour(),
                mtime.minute(),
                mtime.second(),
                mtime.year(),
                mtime.month(),
                mtime.day(),
                file.display()
            );
        }
    }

    // compute CRC32 of each file and generate the SFV listing
    let mut success = true;
    for file in &files {
        match compute_crc32(file) {
            Ok(crc32) if basename => {
                let name = file.file_name().unwrap();
                println!("{} {:X}",  AsRef::<Path>::as_ref(&name).display(), crc32)
            }
            Ok(crc32) => println!("{} {:X}", file.display(), crc32),
            Err(err) => {
                success = false;
                eprintln!("cksfv: {}: {}", file.display(), err)
            }
        }
    }

    // return `true` if all CRC32 where successfully computed
    success
}

/// Check a SFV listing at the given location, optionally using `workdir`.
fn cksfv(sfv: &Path, workdir: Option<&Path>) -> bool {
    // print the terminal "UI"
    let workdir = workdir.unwrap_or_else(|| Path::new("."));
    println!(
        "--( Verifying: {} ){}",
        sfv.display(),
        "-".repeat(63 - sfv.display().to_string().len())
    );

    // open the SFV listing
    let listing = match File::open(sfv) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            eprintln!("cksfv: {}: {}", sfv.display(), err);
            return false;
        }
    };

    // check every line of the listing
    let mut success = true;
    let mut lines = listing.lines();
    while let Some(Ok(line)) = lines.next() {
        if !line.starts_with(';') {
            // extract filename and CRC from listing
            let i = line.trim_end().rfind(' ').unwrap();
            let filename = Path::new(&line[..i]);
            let crc32_old = u32::from_str_radix(&line[i+1..], 16).unwrap();
            // check the current CRC32 and compare against recorded one
            match compute_crc32(&workdir.join(filename)) {
                Ok(crc32_new) if crc32_new != crc32_old => {
                    eprintln!("{:<50}different CRC", filename.display());
                    success = false;
                }
                Err(err) => {
                    eprintln!("{:<50}{:<30}", filename.display(), err);
                    success = false
                }
                Ok(crc32_new) => {
                    eprintln!("{:<50}OK", filename.display());
                }
            }
        }
    }

    // add result message
    eprintln!("{}", "-".repeat(80));
    if success {
        eprintln!("Everything OK");
    } else {
        eprintln!("Errors Occured")
    }
    success
}

// ---------------------------------------------------------------------------

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
                .conflicts_with("g"),
        )
        .arg(
            Arg::with_name("g")
                .short("g")
                .value_name("path")
                .help("Go to the path name directory and verify the sfv file")
                .takes_value(true)
                .conflicts_with("C")
                .conflicts_with("f"),
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
                .conflicts_with("g")
                .conflicts_with("f"),
        )
        .get_matches();

    // generate a new sfv file if given files as input
    if let Some(files) = matches.values_of("file") {
        let result = newsfv(files.map(Path::new), matches.is_present("b"));
        std::process::exit(!result as i32);
    }

    // check files using the given SFV listing
    if let Some(sfv) = matches.value_of("f").map(Path::new) {
        let workdir = matches.value_of("C").map(Path::new);
        let result = cksfv(sfv, workdir);
        std::process::exit(!result as i32);
    } else if let Some(sfv) = matches.value_of("g").map(Path::new) {
        let workdir = sfv.parent();
        let result = cksfv(sfv, workdir);
        std::process::exit(!result as i32);
    }

    // otherwise is no operation given exit with EINVAL
    println!("{}", matches.usage());
    std::process::exit(22)
}
