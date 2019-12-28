use std::io::BufRead;
use std::io::BufReader;
use std::io::Cursor;
use std::io::Error as IoError;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

/// Get the path to a resource in the `data` folder as a string.
fn data(name: &str) -> String {
    Path::new(file!())
        .parent()
        .unwrap()
        .join("data")
        .join(name)
        .into_os_string()
        .into_string()
        .expect("invalid Unicode data")
}

#[test]
/// Check that running with both `-g` and `-C` flags fails.
fn test_1() {

    let config = cksfv::Config::default()
        .with_stdout(cksfv::Output::Other(Box::new(Cursor::new(Vec::new()))));

    let input = PathBuf::from(data("1.txt"));
    if let Err(e) = cksfv::newsfv(vec![input.as_path()], config) {
        panic!("cksfv failed with: {:?}", e);
    }

    // assert_eq!(
    //     BufReader::new(std::io::Cursor::new(&buffer))
    //         .lines()
    //         .collect::<Result<Vec<String>, _>>()
    //         .unwrap()
    //         .last()
    //         .unwrap(),
    //     std::fs::File::open(data("1.sfv")).map(BufReader::new).unwrap()
    //         .lines()
    //         .collect::<Result<Vec<String>, _>>()
    //         .unwrap()
    //         .last()
    //         .unwrap(),
    // );
}
