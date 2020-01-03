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
use getset::Getters;
use getset::MutGetters;
use getset::Setters;
use clap::App;
use clap::Arg;
use crc32fast::Hasher;

/// Use a 64k buffer size for better performance.
const DEFAULT_BUFFER_SIZE: usize = 65536;

/// The final value of a CRC32 checksum round.
pub type Crc32 = u32;

// ---------------------------------------------------------------------------

/// Given a path to a file, attempt to compute its CRC32 hash.
fn compute_crc32(file: &Path) -> Result<Crc32, IoError> {
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
fn compute_crc32_inner(mut file: File) -> Result<Crc32, IoError> {
    let mut hasher = Hasher::new();
    let mmap = unsafe { memmap::MmapOptions::new().map(&file)? };
    hasher.update(&mmap[..]);
    Ok(hasher.finalize())
}

/// Compute a CRC32 from a file content without using `mmap`.
#[cfg(not(feature = "mmap"))]
fn compute_crc32_inner(mut file: File) -> Result<Crc32, IoError> {
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

pub trait WriteDebug: Debug + Write {}

impl<F: Debug + Write> WriteDebug for F {}

#[derive(Debug)]
pub enum Output {
    Devnull,
    Stdout(std::io::Stdout),
    Stderr(std::io::Stderr),
}

impl Output {
    pub fn devnull() -> Self {
        Self::Devnull
    }

    pub fn stdout() -> Self {
        Output::Stdout(std::io::stdout())
    }

    pub fn stderr() -> Self {
        Output::Stderr(std::io::stderr())
    }
}

impl Clone for Output {
    fn clone(&self) -> Self {
        use self::Output::*;
        match self {
            Devnull => Self::devnull(),
            Stdout(_) => Self::stdout(),
            Stderr(_) => Self::stderr(),
        }
    }
}

impl Default for Output {
    fn default() -> Self {
        Self::stderr()
    }
}

impl Write for Output {
    fn write(&mut self, buf: &[u8]) -> Result<usize, IoError> {
        use self::Output::*;
        match self {
            Devnull => Ok(buf.len()),
            Stdout(out) => out.write(buf),
            Stderr(err) => err.write(buf),
        }
    }

    fn flush(&mut self) -> Result<(), IoError> {
        use self::Output::*;
        match self {
            Devnull => Ok(()),
            Stdout(out) => out.flush(),
            Stderr(err) => err.flush(),
        }
    }
}

// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Getters, MutGetters, Setters)]
pub struct Config {
    #[get = "pub"]
    #[get_mut = "pub"]
    #[set = "pub"]
    stdout: Output,
    #[get = "pub"]
    #[get_mut = "pub"]
    #[set = "pub"]
    stderr: Output,
    #[get = "pub"]
    #[get_mut = "pub"]
    #[set = "pub"]
    quiet: bool,
    #[get = "pub"]
    #[get_mut = "pub"]
    #[set = "pub"]
    print_basename: bool,
    #[get = "pub"]
    #[get_mut = "pub"]
    #[set = "pub"]
    ignore_case: bool,
    #[get = "pub"]
    #[get_mut = "pub"]
    #[set = "pub"]
    force_slashes: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self::new()
    }
}

impl Config {
    /// Create a new `Cksfv` instance with default arguments.
    pub fn new() -> Self {
        Config {
            stdout: Output::stdout(),
            stderr: Output::stderr(),
            quiet: false,
            print_basename: false,
            ignore_case: false,
            force_slashes: false,
        }
    }

    pub fn with_stdout(mut self, stdout: Output) -> Self {
        self.stdout = stdout;
        self
    }

    pub fn with_stderr(mut self, stderr: Output) -> Self {
        self.stderr = stderr;
        self
    }

    pub fn with_print_basenamet(mut self, print_basename: bool) -> Self {
        self.print_basename = print_basename;
        self
    }

    /// Consume the configuration instance and get the `stdout` field.
    pub fn extract_stdout(self) -> Output {
        self.stdout
    }
}

// ---------------------------------------------------------------------------

/// Generate a new SFV listing from a list of files.
///
/// This function always writes the result to `config.stdout`, which defaults
/// to `std::io::Stdout` if no configuration is provided.
pub fn newsfv<'a, F, C>(files: F, config: C) -> Result<bool, IoError>
where
    F: IntoIterator<Item = &'a Path>,
    C: Into<Option<Config>>
{
    // get a default config if none provided.
    let mut cfg: Config = config.into().unwrap_or_default();

    // collect the files
    let files: Vec<&Path> = files.into_iter().collect();

    // generate the headers from the files that where found
    let now: DateTime<Local> = Local::now();
    write!(
        cfg.stdout,
        "; Generated by cksfv.rs v{} on {:04}-{:02}-{:02} at {:02}:{:02}.{:02}\n",
        crate_version!(),
        now.year(),
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    )?;
    write!(cfg.stdout, "; Project web site: {}\n", env!("CARGO_PKG_REPOSITORY"))?;
    write!(cfg.stdout, ";\n")?;
    for file in files.iter().filter(|p| p.is_file()) {
        if let Ok(metadata) = std::fs::metadata(file) {
            let mtime: DateTime<Local> = From::from(metadata.modified().unwrap());
            write!(
                cfg.stdout,
                "; {:>12}  {:02}:{:02}.{:02} {:04}-{:02}-{:02} {}\n",
                metadata.len(),
                mtime.hour(),
                mtime.minute(),
                mtime.second(),
                mtime.year(),
                mtime.month(),
                mtime.day(),
                file.display()
            )?;
        }
    }

    // compute CRC32 of each file and generate the SFV listing
    let mut success = true;
    for file in &files {
        match compute_crc32(file) {
            Ok(crc32) if cfg.print_basename => {
                let name = file.file_name().unwrap();
                write!(cfg.stdout, "{} {:X}\n",  AsRef::<Path>::as_ref(&name).display(), crc32)?
            }
            Ok(crc32) => write!(cfg.stdout, "{} {:X}\n", file.display(), crc32)?,
            Err(err) => {
                success = false;
                write!(cfg.stderr, "cksfv: {}: {}\n", file.display(), err)?
            }
        }
    }

    // return `true` if all CRC32 where successfully computed
    Ok(success)
}

/// Check a SFV listing at the given location, optionally using `workdir`.
///
/// This function always writes some progress messages to `config.stderr`, and
/// outputs a message line for each file it checks to `config.stdout`.
pub fn cksfv<'a, F, C>(sfv: &Path, workdir: Option<&Path>, config: C, files: Option<F>) -> Result<bool, IoError>
where
    F: IntoIterator<Item = &'a Path>,
    C: Into<Option<Config>>
{
    // get a default config if none provided.
    let mut cfg: Config = config.into().unwrap_or_default();

    // print the terminal "UI"
    let workdir = workdir.unwrap_or_else(|| Path::new("."));
    write!(
        cfg.stderr,
        "--( Verifying: {} ){}\n",
        sfv.display(),
        "-".repeat(63 - sfv.display().to_string().len())
    )?;

    // open the SFV listing
    let listing = match File::open(sfv) {
        Ok(file) => BufReader::new(file),
        Err(err) => {
            write!(cfg.stderr, "cksfv: {}: {}\n", sfv.display(), err);
            return Ok(false);
        }
    };

    let mut success = true;
    let mut lines = listing.lines();
    if let Some(files) = files {
        // only check the files given as arguments
        unimplemented!("TODO: checking with file arguments");
    } else {
        // check every line of the listing
        while let Some(Ok(line)) = lines.next() {
            if !line.starts_with(';') {
                // extract filename and CRC from listing
                let i = line.trim_end().rfind(' ').unwrap();
                let filename = Path::new(&line[..i]);
                let crc32_old = u32::from_str_radix(&line[i+1..], 16).unwrap();
                // check the current CRC32 and compare against recorded one
                match compute_crc32(&workdir.join(filename)) {
                    Ok(crc32_new) if crc32_new != crc32_old => {
                        success = false;
                        if cfg.quiet {
                            write!(cfg.stdout, "{:<50}different CRC\n", filename.display())?;
                        } else {
                            write!(cfg.stdout, "cksfv: {}: Has a different CRC\n", filename.display())?;
                        }
                    }
                    Err(err) if cfg.quiet => {
                        write!(cfg.stdout, "cksfv: {}: {}\n", filename.display(), err)?;
                    }
                    Err(err) => {
                        write!(cfg.stdout, "{:<50}{:<30}\n", filename.display(), err)?;
                        success = false
                    }
                    Ok(_) if !cfg.quiet => {
                        write!(cfg.stdout, "{:<50}OK\n", filename.display())?;
                    }
                    Ok(_) => (),
                }
            }
        }
    }

    // add result message
    write!(cfg.stderr, "{}\n", "-".repeat(80))?;
    if !cfg.quiet {
        if success {
            write!(cfg.stdout, "Everything OK\n")?;
        } else {
            write!(cfg.stdout, "Errors Occured\n")?;
        }
    }
    Ok(success)
}
