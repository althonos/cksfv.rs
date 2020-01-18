extern crate assert_cli;
#[macro_use]
extern crate textwrap_macros;

use std::path::Path;

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
    #[allow(non_snake_case)]
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
    /// TODO
    fn cksfv_priority_over_newsfv() {
        assert_cli::Assert::main_binary()
            .with_args(&["-g", &data("1.sfv"), &data("2.txt")])
            .succeeds()
            .and()
            .stderr().contains(format!("Verifying: {}", data("1.sfv")).as_str())
            .unwrap()
    }
}

/// Tests to check stdout / stderr redirection
mod io {

    use super::data;

    #[test]
    fn test_g() {
        assert_cli::Assert::main_binary()
            .with_args(&["-g", &data("12.sfv")])
            .succeeds()
            .and()
            .stdout().is("")
            .and()
            .stderr().contains(
                dedent!(
                    r#"
                    --( Verifying: tests/data/12.sfv )----------------------------------------------
                    1.txt                                             OK
                    2.txt                                             OK
                    --------------------------------------------------------------------------------
                    Everything OK
                    "#
                )
            )
            .unwrap()
    }

    #[test]
    fn test_g_with_c() {
        assert_cli::Assert::main_binary()
            .with_args(&["-c", "-g", &data("12.sfv")])
            .succeeds()
            .and()
            .stdout().is(
                dedent!(
                    r#"
                    1.txt                                             OK
                    2.txt                                             OK
                    Everything OK
                    "#
                )
            )
            .and()
            .stderr().satisfies(|x|
                x.ends_with(dedent!(
                    r#"
                    --( Verifying: tests/data/12.sfv )----------------------------------------------
                    --------------------------------------------------------------------------------
                    "#
                )),
                "wrong output\n"
            )
            .unwrap()
    }

    #[test]
    /// Check that `-r` flag outputs everything to stderr
    fn test_recursive() {
        assert_cli::Assert::main_binary()
            .with_args(&["-r"])
            .succeeds()
            .and()
            .stdout().is("")
            .and()
            .stderr()
                .contains(
                    textwrap::dedent(&format!(
                        r#"
                        Entering directory: {projdir}/tests/data
                        --( Verifying: 12.sfv )---------------------------------------------------------
                        1.txt                                             OK
                        2.txt                                             OK
                        --------------------------------------------------------------------------------
                        Everything OK
                        "#,
                        projdir = env!("CARGO_MANIFEST_DIR"))
                    ).trim()
                )
            .stderr()
                .contains(
                    textwrap::dedent(&format!(
                        r#"
                        Entering directory: {projdir}/tests/data
                        --( Verifying: 2.sfv )----------------------------------------------------------
                        2.txt                                             OK
                        --------------------------------------------------------------------------------
                        Everything OK
                        "#,
                        projdir = env!("CARGO_MANIFEST_DIR"))
                    ).trim()
                )
            .stderr()
                .contains(
                    textwrap::dedent(&format!(
                        r#"
                        Entering directory: {projdir}/tests/data
                        --( Verifying: 1.sfv )----------------------------------------------------------
                        1.txt                                             OK
                        --------------------------------------------------------------------------------
                        Everything OK
                        "#,
                        projdir = env!("CARGO_MANIFEST_DIR"))
                    ).trim()
                )
            .stderr()
                .contains(
                    textwrap::dedent(&format!(
                        r#"
                        Entering directory: {projdir}/tests/data
                        --( Verifying: 0.sfv )----------------------------------------------------------
                        --------------------------------------------------------------------------------
                        Everything OK
                        "#,
                        projdir = env!("CARGO_MANIFEST_DIR")
                    )).trim()
                )
            .unwrap()
    }

    #[test]
    fn test_recursive_with_c() {
        assert_cli::Assert::main_binary()
            .with_args(&["-r", "-c"])
            .succeeds()
            .and()
            .stdout()
                .contains(dedent!(
                    r#"
                    1.txt                                             OK
                    2.txt                                             OK
                    Everything OK
                    "#
                ).trim())
            .stdout()
                .contains(dedent!(
                    r#"
                    1.txt                                             OK
                    Everything OK
                    "#
                ).trim())
            .stdout()
                .contains(dedent!(
                    r#"
                    2.txt                                             OK
                    Everything OK
                    "#
                ).trim())
            .stdout()
                .contains(dedent!(
                    r#"
                    Everything OK
                    "#
                ).trim())
            .and()
            .stderr()
                .contains(textwrap::dedent(&format!(
                    r#"
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 12.sfv )---------------------------------------------------------
                    --------------------------------------------------------------------------------
                    "#,
                    projdir = env!("CARGO_MANIFEST_DIR")
                )).trim())
            .stderr()
                .contains(textwrap::dedent(&format!(
                    r#"
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 2.sfv )----------------------------------------------------------
                    --------------------------------------------------------------------------------
                    "#,
                    projdir = env!("CARGO_MANIFEST_DIR")
                )).trim())
            .stderr()
                .contains(textwrap::dedent(&format!(
                    r#"
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 1.sfv )----------------------------------------------------------
                    --------------------------------------------------------------------------------
                    "#,
                    projdir = env!("CARGO_MANIFEST_DIR")
                )).trim())
            .stderr()
                .contains(textwrap::dedent(&format!(
                    r#"
                    Entering directory: {projdir}/tests/data
                    --( Verifying: 0.sfv )----------------------------------------------------------
                    --------------------------------------------------------------------------------
                    "#,
                    projdir = env!("CARGO_MANIFEST_DIR")
                )).trim())
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
