// SPDX-License-Identifier: GPL-3.0-only

use log::LevelFilter;
use core::str::FromStr;
use std::io::Write;
use std::panic;

use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

/// Adds verbose levels: off, error, (warn,) info, debug, (trace). For example info includes error, debug includes info and error
/// The level is set through program arguments. Default is Warn
/// run the program with `-l info` (eq. `--logger info`) optional argument. Case-insensitive
/// There is also an optional json formatter that outputs plain json format. run the program with `-j` or `--json` optional argument.
pub fn set(log_level_str: &str, is_json: bool) {
    let log_level = match LevelFilter::from_str(log_level_str) {
        Ok(level) => level,
        Err(_) => LevelFilter::Warn 
    };

    // Content called when panic! is encountered to close logger brackets and print error
    set_panic_hook(is_json);

    if is_json {
        print_json_output_opener();
        // prevents any output stylization from the colored crate
        colored::control::set_override(false);
    }
    
    let mut builder = env_logger::Builder::new();
    if is_json {
        // Note: record .file() and line() allow to get the origin of the print
        builder.format(move |buf, record| {
            writeln!(buf, r#"    {{
      "status": "{}",
      "message": "{}"
    }},"#,
            record.level().to_string().to_ascii_lowercase(),
            record.args().to_string()
        )});
    }
    builder.filter(None, log_level)
    .format_timestamp(None)
    .format_level(false)
    .format_module_path(false)
    .init();
}

fn set_panic_hook(is_json_log: bool) {
    panic::set_hook(Box::new(move |e| {
        let e_message = e.payload().downcast_ref::<&str>().unwrap_or(&"no message provided");
        let location = match e.location() {
            Some(loc) => {
                format!(" in file '{}' at line {}", loc.file(), loc.line())
            },
            None => String::new()
        };
        let message = format!("The following unrecoverable error occured{}: '{}'", location, e_message);
        if is_json_log {
            println!(r#"    {{
      "Compilation result": {{
          "status": "unrecoverable error",
          "message": "{}"
      }}
    }}
  ]
}}"#, message);
        } else {
            error!("{}", message);
        }
    }));
}

fn print_json_output_opener() {
    let start = SystemTime::now();
    let time = match start.duration_since(UNIX_EPOCH) {
        Ok(since_the_epoch) => since_the_epoch.as_millis().to_string(),
        Err(_) => "could not get correct time".to_owned()
    };
    println!("{{\n  \"time\": \"{}\",\n  \"logs\": [", time);
}

pub fn print_output_closure(is_json: bool, is_success: bool, input_file: &str, output_file: &str) {
    let pwd = std::env::current_dir().unwrap_or(PathBuf::new());
    match is_json {
        true => {
            let res_str = match is_success {
                true => "SUCCESS",
                false => "FAILURE"
            };
            println!(r#"    {{
      "Compilation result": {{
        "status": "{}",
        "from": "{}",
        "to": "{}",
        "pwd": {:?}
      }}
    }}
  ]
}}"#, res_str, input_file, output_file, pwd);
        },
        false => {
            let res_str = match is_success {
                true => format!("Everything worked as expected, \"{}\" generated from \"{}\"", output_file, input_file),
                false => format!("An error occured, \"{}\" file has not been created from \"{}\"", output_file, input_file)
            };
            println!("{}", res_str)
        }
    };
}
