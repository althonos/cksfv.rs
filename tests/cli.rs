extern crate assert_cli;

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

/// Tests to mimick the original behaviour of `cksfv`.
mod behaviour {

    use super::data;

    #[test]
    /// Check that running with both `-g` and `-C` flags fails.
    fn failure_g_and_C_flags() {
        assert_cli::Assert::main_binary()
            .with_args(&["-C", "/tmp", "-g", "list.sfv"])
            .fails()
            .unwrap();
    }

    #[test]
    /// Check that running with multiple `-f` values checks the last one.
    fn multiple_f_last() {
        assert_cli::Assert::main_binary()
            .with_args(&["-C", &data(""), "-f", &data("1.sfv"), "-f", &data("2.sfv")])
            .succeeds()
            .and()
            .stderr().contains(format!("Verifying: {}", data("2.sfv")).as_str())
            .unwrap()
    }

    #[test]
    /// Check that running with multiple `-g` values checks the last one.
    fn multiple_g_last() {
        assert_cli::Assert::main_binary()
            .with_args(&["-g", &data("1.sfv"), "-g", &data("2.sfv")])
            .succeeds()
            .and()
            .stderr().contains(format!("Verifying: {}", data("2.sfv")).as_str())
            .unwrap()
    }

    #[test]
    #[ignore]
    /// Check that when given both files to check and files to create a new
    /// SFV for the program only checks the existing SFV.
    fn cksfv_priority_over_newsfv() {
        assert_cli::Assert::main_binary()
            .with_args(&["-g", &data("1.sfv"), &data("2.txt")])
            .succeeds()
            .and()
            .stderr().contains(format!("Verifying: {}", data("2.sfv")).as_str())
            .unwrap()
    }
}
