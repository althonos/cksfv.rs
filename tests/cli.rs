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
    ///
    /// FAILS because of clap-rs/clap#1610
    fn cksfv_priority_over_newsfv() {
        assert_cli::Assert::main_binary()
            .with_args(&["-g", &data("1.sfv"), &data("2.txt")])
            .succeeds()
            .and()
            .stderr().contains(format!("Verifying: {}", data("2.sfv")).as_str())
            .unwrap()
    }
}

/// Tests to check stdout / stderr redirection
mod io {

    use super::data;

    #[test]
    /// Check that `-r` flag outputs everything to stderr
    fn test_recursive() {
        assert_cli::Assert::main_binary()
            .with_args(&["-r"])
            .succeeds()
            .and()
            .stdout().is("")
            .and()
            .stderr().satisfies(|x|
                x.ends_with(&textwrap::dedent(&format!(
                    r#"
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 12.sfv )---------------------------------------------------------
                    1.txt                                             OK
                    2.txt                                             OK
                    --------------------------------------------------------------------------------
                    Everything OK
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 2.sfv )----------------------------------------------------------
                    2.txt                                             OK
                    --------------------------------------------------------------------------------
                    Everything OK
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 1.sfv )----------------------------------------------------------
                    1.txt                                             OK
                    --------------------------------------------------------------------------------
                    Everything OK
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 0.sfv )----------------------------------------------------------
                    --------------------------------------------------------------------------------
                    Everything OK
                    "#,
                    projdir = env!("CARGO_MANIFEST_DIR"))
                )),
                "wrong output\n"
            )
            .unwrap()
    }

    #[test]
    fn test_recursive_with_c() {
        assert_cli::Assert::main_binary()
            .with_args(&["-r", "-c"])
            .succeeds()
            .and()
            .stdout().is(textwrap::dedent(
                r#"
                1.txt                                             OK
                2.txt                                             OK
                Everything OK
                2.txt                                             OK
                Everything OK
                1.txt                                             OK
                Everything OK
                Everything OK
                "#
            ).as_str())
            .and()
            .stderr().satisfies(|x|
                x.ends_with(&textwrap::dedent(&format!(
                    r#"
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 12.sfv )---------------------------------------------------------
                    --------------------------------------------------------------------------------
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 2.sfv )----------------------------------------------------------
                    --------------------------------------------------------------------------------
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 1.sfv )----------------------------------------------------------
                    --------------------------------------------------------------------------------
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 0.sfv )----------------------------------------------------------
                    --------------------------------------------------------------------------------
                    "#,
                    projdir = env!("CARGO_MANIFEST_DIR"))
                )),
                "wrong output\n"
            )
            .unwrap()
    }

    #[test]
    fn test_recursive_with_q() {
        assert_cli::Assert::main_binary()
            .with_args(&["-r", "-q"])
            .succeeds()
            .and()
            .stdout().is("")
            .and()
            .stderr().doesnt_contain("Everything OK")
            .unwrap()
    }
}
