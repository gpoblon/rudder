// Copyright 2019 Normation SAS
//
// This file is part of Rudder.
//
// Rudder is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// In accordance with the terms of section 7 (7. Additional Terms.) of
// the GNU General Public License version 3, the copyright holders add
// the following Additional permissions:
// Notwithstanding to the terms of section 5 (5. Conveying Modified Source
// Versions) and 6 (6. Conveying Non-Source Forms.) of the GNU General
// Public License version 3, when you create a Related Module, this
// Related Module is not considered as a part of the work and may be
// distributed under the license agreement of your choice.
// A "Related Module" means a set of sources files including their
// documentation that, without modification of the Source Code, enables
// supplementary functions or services in addition to those offered by
// the Software.
//
// Rudder is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Rudder.  If not, see <http://www.gnu.org/licenses/>.

#![allow(clippy::large_enum_variant)]

#[macro_use]
extern crate log;
use log::LevelFilter;
use core::str::FromStr;
use std::io::Write;

use rudderc::compile::compile_file;
use rudderc::translate::translate_file;
use structopt::StructOpt;
use colored;
use colored::Colorize;
use std::path::PathBuf;

///!  Principle:
///!  1-  rl -> PAST::add_file() -> PAST
///!         -> AST::from_past -> AST
///!         -> generate() -> cfengine/json/...
///!
///!  2- json technique -> translate() -> rl
///!
///!  3- ncf library -> generate-lib() -> stdlib.rl + translate-config
///!

// MAIN

// Questions :
// - compatibilité avec les techniques définissant des variables globales depuis une GM qui dépend d'une autre ?
// - usage du '!' -> "macros", enum expr, audit&test ?
// - sous typage explicite mais pas chiant
// - a qui s'applique vraiment les namespace ? variables, resources, enums, fonctions ? quels sont les default intelligents ?
// - a quoi ressemblent les iterators ?
// - arguments non ordonnés pour les resources et les states ?
// - usage des alias: pour les children, pour les (in)compatibilités, pour le générateur?

// Next steps:
//
//

// TODO a state S on an object A depending on a condition on an object B is invalid if A is a descendant of B
// TODO except if S is the "absent" state

/// Rust langage compiler
#[derive(Debug, StructOpt)]
#[structopt(rename_all = "kebab-case")]
struct Opt {
    /// Output file or directory
    #[structopt(long, short)]
    output: PathBuf,
    /// Input file or directory
    #[structopt(long, short)]
    input: PathBuf,
    /// Set to use technique translation mode
    #[structopt(long, short)]
    translate: bool,
    /// Set to compile a single technique
    #[structopt(long, short)]
    compile: bool,
    /// Set to change default env logger behavior (INFO, DEBUG, ERROR)
    #[structopt(long, short)]
    logger: Option<String>,
    /// Output format to use: standard terminal or json style
    #[structopt(long)]
    json: bool,
}

// TODO use termination
fn main() {
    
    // easy option parsing
    let opt = Opt::from_args();
    
    // prevents any output stylization from the colored crate
    if opt.json {
        colored::control::set_override(false);
    }

    if let Some(log) = opt.logger {
        set_logger(&log, opt.json);
    } 

    if opt.translate {
        match translate_file(&opt.input, &opt.output) {
            Err(e) => error!("{}", e),
            Ok(_) => info!("{} {}", "File translation".bright_green(), "OK".bright_cyan()),
        }
    } else {
        debug!("Hm... I shoud end up here if I am compiling");
        match compile_file(&opt.input, &opt.output, opt.compile) {
            Err(e) => error!("{}", e),
            Ok(_) => info!("{} {}", "Compilation".bright_green(), "OK".bright_cyan()),
        }
    }
}

/// Adds verbose levels: OFF ERROR (WARN) INFO DEBUG (TRACE). For example INFO includes ERROR, DEBUG includes INFO and ERROR
/// The level is set through program arguments. Default is Warn
/// run the program with `-l INFO` (eq. `--logger INFO`) option argument
fn set_logger(log_level_str: &str, is_json: bool) {
    let log_level = match LevelFilter::from_str(log_level_str) {
        Ok(level) => level,
        Err(_) => LevelFilter::Warn 
    };
    let mut builder = env_logger::Builder::new();
    if is_json {
        // Note: record .file() and line() allow to get the origin of the print
        builder.format(|buf, record| {
            // writeln!(buf, "{{\n\t\"log level\": \"{}\",\n\t\"message\": \"{}\",\n\t\"ts\": \"{}\"\n}}",
            writeln!(buf, r#"{{
    "log level": "{}",
    "message": "{}",
    "ts": "{}"
}}"#,
            record.level().to_string().to_ascii_lowercase(),
            record.args().to_string(),
            buf.timestamp())
        });
    }
    builder.filter(None, log_level)
    .format_timestamp(None)
    .format_level(false)
    .format_module_path(false)
    .init();
}

// fn set_fmt(is_json: bool, buf: &mut Formatter, record: &Record) -> std::io::Result<()> {
//     // if is_json == false {
//         writeln!(buf, "{}: {}",
//         record.level(),
//         record.args());
//     // }
//     Ok(())
// }    

// Phase 2
// - function, measure(=fact), action
// - variable = anything
// - optimize before generation (remove unused code, simplify expressions ..)
// - inline native (cfengine, ...)
// - remediation resource (phase 3: add some reactive concept)
// - read templates and json a compile time
