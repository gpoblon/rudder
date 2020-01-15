// To test only the technique compiler, run this: `cargo test --test compile`

/// this file will be the integration tests base for techniques compilation to cfengine file
/// calling files from the *techniques* folder
/// takes an rl file and a cf file, parses the first, compiles, and compares expected output with the second
/// naming convention: 
/// input is rl -> state_checkdef.rl
/// output is .rl.cf -> state_checkdef.rl.cf
/// with success: s_ & failure: f_
/// example of files that should succeed: s_errors.rl s_errors.rl.cf
// TODO: cf files added (solely in case of success) for later use: comparing it to generated output (.rl.cf)

mod utils;
use utils::*;

use std::{
    fs,
    io::Error,
    path::{Path, PathBuf},
    ffi::OsStr,
};
use colored::Colorize;

// This static array is an alternative (yet exactly the same as the toml file test).
// Trace of some iterative work, need to pick between toml file test and directly in a test file
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
        "f_enum.rl",
        r#"@format=0
        enum error {
            ok,
            err
        }"#
    ),
];

#[test]
// Tests statically string-defined "files"
fn raw_filestrings() -> Result<(), Error> {
    fs::create_dir_all("tests/tmp")?;
    let tests: Vec<(&str, &str)> = TESTS.iter().cloned().collect();
    for (filename, content) in tests {
        let path = PathBuf::from(&format!("tests/tmp/{}", filename));
        prepare_temporary_file(&path, content)?;
        let result = test_file(&path);
        match should_compile(&path) {
            Some(is_success) => assert_eq!(result.is_ok(), is_success),
            None => println!("{}: could not test  {:?}", "Warning (test)".bright_yellow().bold(), path.to_str().unwrap().bright_yellow()),
        }
        fs::remove_file(path).expect("Could not delete temporary file");
    }
    Ok(())
}

#[test]
// Tests statically string-defined "files"
fn literal_filestrings() -> Result<(), Error> {
    fs::create_dir_all("tests/tmp")?;
    match fs::read_to_string(Path::new("tests/virtual_files.toml")) {
        Ok(content) => {
            match content.parse::<toml::Value>() {
                Ok(toml_file) => test_toml(toml_file)?,
                Err(e) => return Err(e.into()),
            }
        },
        Err(e) => return Err(e),
    }
    // let tests: Vec<(&str, bool, &str)> = TESTS.iter().cloned().collect();
    Ok(())
}

fn test_toml(toml_file: toml::Value) -> Result<(), Error> {
    if let Some(files) = toml_file.as_table() {
        for (filename, content) in files {
            if let Some(content) = content.as_str() {
                let path = PathBuf::from(&format!("tests/tmp/{}", filename));
                prepare_temporary_file(&path, content)?;
                let result = test_file(&path);
                match should_compile(&path) {
                    Some(is_success) => assert_eq!(result.is_ok(), is_success),
                    None => println!("{}: could not test  {:?}", "Warning (test)".bright_yellow().bold(), path.to_str().unwrap().bright_yellow()),
                }
                fs::remove_file(path).expect("Could not delete temporary file");
            }
        }
    }
    Ok(())
}

#[test]
/// Tests every file from the */compile* folder
fn real_files() -> Result<(), Error> {
    fs::create_dir_all("tests/tmp")?;
    let file_paths = fs::read_dir("tests/compile")?;
    for file_entry in file_paths {
        let path: &Path= &file_entry?.path();
        if path.extension() == Some(OsStr::new("rs")) {
            let result = test_file(path);
            match should_compile(path) {
                Some(is_success) => assert_eq!(result.is_ok(), is_success),
                None => println!("{}: could not test  {:?}", "Warning (test)".bright_yellow().bold(), path.to_str().unwrap().bright_yellow()),
            }
        }
    }
    Ok(())
}