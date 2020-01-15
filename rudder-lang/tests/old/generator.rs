use std::{
    fs,
    io::Error,
    io::prelude::*,
    path::Path,
};

fn generate_tests_file(files: Vec<(String, String)>) -> Result<(), Error> {
    let header = r#"mod utils;
use utils::*;
use colored::Colorize;
use std::{
    fs,
    io::Error,
    path::PathBuf,
};
use test_case::test_case;"#;

    let mut content = header.to_owned();
    for (filename, test_content) in files {
        let new_test = format!("#[test_case( r#\"{}\"#, \"{}\" ; \"{}\")]", test_content, filename, filename);
        content = [content, new_test].join("\n\n");
    }

    let func_call = r#"

fn generated(content: &str, filename: &str) -> Result<(), Error> {
    fs::create_dir_all("tests/tmp")?;
    let path = PathBuf::from(&format!("tests/tmp/{}", filename));
    prepare_temporary_file(&path, content)?;
    let result = test_file(&path);
    match should_compile(&path) {
        Some(is_success) => assert_eq!(result.is_ok(), is_success),
        None => println!("{}: could not test  {:?}", "Warning (test)".bright_yellow().bold(), path.to_str().unwrap().bright_yellow()),
    }
    fs::remove_file(path).expect("Could not delete temporary file");
    Ok(())
}"#;
    content.push_str(&func_call);
    let mut file = fs::File::create("tests/generated.rs")?;
    file.write_all(content.as_bytes())?;
    Ok(())
}


/// This function generates `.rl` files to test,
/// then a rust file that calls out a test for every single of those `.rl` files
/// Dirty turnaround to split tests
fn main() -> Result<(), Error> {
    // files are generated files to test
    let files = literal_filestrings()?;
    generate_tests_file(files)?;
    Ok(())
}

fn literal_filestrings() -> Result<Vec<(String, String)>, Error> {
    fs::create_dir_all("tests/tmp")?;
    match fs::read_to_string(Path::new("tests/virtual_files.toml")) {
        Ok(content) => {
            match content.parse::<toml::Value>() {
                Ok(toml_file) => return Ok(populate_toml(toml_file)?),
                Err(e) => return Err(e.into()),
            }
        },
        Err(e) => return Err(e),
    }
}

fn populate_toml(toml_file: toml::Value) -> Result<Vec<(String, String)>, Error> {
    let mut populated_files = Vec::new();
    if let Some(files) = toml_file.as_table() {
        for (filename, content) in files {
            if let Some(content) = content.as_str() {
                populated_files.push((filename.to_owned(), content.to_owned()));
            }
        }
    }
    Ok(populated_files)
}