# `cksfv.rs` [![Star me](https://img.shields.io/github/stars/althonos/cksfv.rs.svg?style=social&label=Star&maxAge=3600)](https://github.com/althonos/cksfv.rs/stargazers)

*A 10x faster [cksfv](https://zakalwe.fi/~shd/foss/cksfv/) reimplementation
using Rust and the [crc32fast](https://crates.io/crates/crc32fast) crate.*

[![TravisCI](https://img.shields.io/travis/althonos/cksfv.rs/master.svg?maxAge=600&style=flat-square)](https://travis-ci.org/althonos/cksfv.rs/branches)
[![Codecov](https://img.shields.io/codecov/c/gh/althonos/cksfv.rs/master.svg?style=flat-square&maxAge=600)](https://codecov.io/gh/althonos/cksfv.rs)
[![License](https://img.shields.io/badge/license-MIT-blue.svg?style=flat-square&maxAge=2678400)](https://choosealicense.com/licenses/mit/)
[![Source](https://img.shields.io/badge/source-GitHub-303030.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/cksfv.rs)
[![Crate](https://img.shields.io/crates/v/cksfv.svg?maxAge=600&style=flat-square)](https://crates.io/crates/cksfv)
[![Documentation](https://img.shields.io/badge/docs.rs-latest-4d76ae.svg?maxAge=2678400&style=flat-square)](https://docs.rs/cksfv)
[![Changelog](https://img.shields.io/badge/keep%20a-changelog-8A0707.svg?maxAge=2678400&style=flat-square)](https://github.com/althonos/cksfv.rs/blob/master/CHANGELOG.md)


## 🗺️ Overview

[`cksfv`](https://zakalwe.fi/~shd/foss/cksfv/) is a FOSS tool developed by
Bryan Call and maintained by [Heikki Orsila](https://github.com/heikkiorsila)
to validate integrity of files using the CRC32 checksum. It can be used to
check a list of files against a `.sfv` signature list, and to generate new
lists.

This repository contains code for an implementation written from scratch in Rust,
that provides the same CLI as the original program but much better performance
thanks to a CRC32 implementation that takes advantage of modern CPUs.


## 🛠️ Features

Features from the original binary:

- [x] Exact same Command-Line Interface for scripting compatibility
- [x] SFV listing generation compatible with the original `cksfv`
- [ ] SFV listing validation with `-f`, `-g` or `-r`
- [ ] Specific options:
  - [ ] `-b` flag to only print base filenames
  - [ ] `-c` flag to print everything to STDOUT
  - [ ] `-C` flag to change directory when processing files
  - [ ] `-i` flag to ignore case on filenames
  - [ ] `-L` flag to follow symlinks
  - [ ] `-q` flag to only print error messages
  - [ ] `-s` flag to replace backslashes

Additional features:

- [x] Support for `mmap` syscall to avoid reading the file directly
- [ ] Multithreading for several files


## ⏱️ Benchmarks

Benchmark where conducted using a 1.6GiB file using the `cksfv` binary
distributed with ArchLinux, or this program compiled with either stable or
nightly Rust, running [`hyperfine`](https://github.com/sharkdp/hyperfine):

```
Benchmark #1: cksfv test.mkv
  Time (mean ± σ):      4.376 s ±  0.120 s    [User: 4.117 s, System: 0.251 s]
  Range (min … max):    4.237 s …  4.555 s    10 runs

Benchmark #2: cargo +stable run --release -- test.mkv
  Time (mean ± σ):     387.9 ms ±   9.3 ms    [User: 158.3 ms, System: 224.9 ms]
  Range (min … max):   380.0 ms … 402.3 ms    10 runs

Benchmark #3: cargo +nightly run --release -- test.mkv
  Time (mean ± σ):     387.9 ms ±  11.7 ms    [User: 160.9 ms, System: 226.6 ms]
  Range (min … max):   373.8 ms … 414.9 ms    10 runs

Benchmark #4: cargo +nightly run --all-features --release test.mkv
  Time (mean ± σ):     347.6 ms ±  12.3 ms    [User: 208.8 ms, System: 136.1 ms]
  Range (min … max):   330.7 ms … 368.3 ms    10 runs
 d

Summary
  'cargo +nightly run --all-features --release test.mkv' ran
    1.12 ± 0.05 times faster than 'cargo +stable run --release -- test.mkv'
    1.12 ± 0.05 times faster than 'cargo +nightly run --release -- test.mkv'
   12.59 ± 0.56 times faster than 'cksfv test.mkv'
```

## 📜 License

This tool is provided under the open-source
[MIT license](https://choosealicense.com/licenses/mit/).
