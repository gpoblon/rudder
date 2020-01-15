mod utils;
use utils::*;
use colored::Colorize;
use std::{
    fs,
    io::Error,
    path::PathBuf,
};
use test_case::test_case;

#[test_case( r#"@format=0
enm error {
    ok,
    err
}
"#, "f_enm" ; "f_enm")]

#[test_case( r#"@format=0
enum error {
    ok,
    err
}
"#, "s_enum" ; "s_enum")]

#[test_case( r#"@format=0
"#, "s_format" ; "s_format")]

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
}