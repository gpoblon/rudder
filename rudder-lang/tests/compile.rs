mod utils;
use utils::*;
use colored::Colorize;
use std::{
    fs,
    io::Error,
    path::PathBuf,
};
use test_case::test_case;

// List of temporary test files: an array of tuples `(filename, is_ok, content)`
// Format is not 100% correct since there are several superfluous whitespaces but these are trimmmed by the parser
static TESTS: &'static [(&str, &str)] = &[
    (
        // a proper enum definition, should pass
        "s_purest.rl",
        r#"@format=0
        "#
    ),
    (
        // a wrong enum definition (enm keyword)
        "f_enm.rl",
        r#"@format=0
        enm error {
            ok,
            err
        }"#
    ),
    (
        // a wrong enum definition (enm keyword)
        "s_enum.rl",
        r#"@format=0
        enum success {
            ok,
            err
        }"#
    ),
];


// add any new entry to the tests by adding a test_case line on the top of the following
// comment any line to skip the corresponding test
#[test_case(2 ; "s_enum")]
#[test_case(1 ; "f_enm")]
#[test_case(0 ; "s_purest")]

fn generated(file_index: usize) -> Result<(), Error> {
    let filename = TESTS[file_index].0;
    let content = TESTS[file_index].1;
    fs::create_dir_all("tests/tmp")?;
    let path = PathBuf::from(&format!("tests/tmp/{}", filename));
    prepare_temporary_file(&path, content)?;
    let result = test_file(&path);
    match should_compile(filename) {
        Some(is_ok) => assert_eq!(result.is_ok(), is_ok),
        None => println!("{}: could not test  {:?}", "Warning (test)".bright_yellow().bold(), path.to_str().unwrap().bright_yellow()),
    }
    fs::remove_file(path).expect("Could not delete temporary file");
    Ok(())
}