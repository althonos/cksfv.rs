extern crate assert_cli;

#[test]
/// Check that running with both `-g` and `-C` flags fails.
fn failure_g_and_C_flags() {
    assert_cli::Assert::main_binary()
        .with_args(&["-C", "/tmp", "-g", "list.sfv"])
        .fails()
        .unwrap();
}
