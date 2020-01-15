// ======================== COMPILE UTILITARY FUNCTIONS ========================

use std::{
    fs,
    io::{Error, Write},
    path::Path,
};
use colored::Colorize;

/// Tool function extracting expected compilation result
// pub fn should_compile(path: &Path) -> Option<bool> {
//     if let Some(filename) = path.file_name() {
//         return match &filename.to_str().unwrap()[ .. 2] {
//             "s_" => Some(true),
//             "f_" => Some(false),
//             _ => None,
//         }    
//     }
//     None
// }

/// Tool function extracting expected compilation result
pub fn should_compile(filename: &str) -> Option<bool> {
    return match &filename[ .. 2] {
        "s_" => Some(true),
        "f_" => Some(false),
        _ => None,
    }    
}

/// Tool function dedicated to create a temporary file, writing content in it.
/// Allows to test simulated files based on strings
pub fn prepare_temporary_file(path: &Path, content: &str) -> Result<(), Error> {
    let mut file = fs::File::create(path)?;
    file.write_all(content.as_bytes())?;
    Ok(())
}

/// Test technique compilation from base crate on a given file
pub fn test_file(path: &Path) -> Result<(), String> {
    let filename = path.to_str().unwrap();
    match rudderc::compile::compile_file(path, path, false) {
        Err(rudderc::error::Error::User(e)) => {
            println!("{}: compilation of {} failed: {}", "Error (test)".bright_red().bold(), filename.bright_yellow(), e);
            return Err(e)
        },
        Ok(_) => {
            println!("{}: compilation of {}", "Success (test)".bright_green().bold(), filename.bright_yellow());
            return Ok(())
        },
        _ => panic!("What kind of error is this ?"),
    };
}

// ======================= TRANSLATE UTILITARY FUNCTIONS =======================
