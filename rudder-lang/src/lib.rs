// SPDX-License-Identifier: GPL-3.0-or-later
// SPDX-FileCopyrightText: 2019-2020 Normation SAS

// Found no other way to import log and its macros
#[macro_use]
extern crate log;

extern crate serde_json;

#[macro_use]
pub mod error;
pub mod compile;
pub mod generator;
pub mod io;
mod ir;
pub mod migrate;
pub mod opt;
pub use generator::Format;
pub mod cfstrings;
pub mod logger;
mod parser;
pub mod rudderlang_lib;
pub mod technique;

use serde::Deserialize;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Deserialize)]
pub enum Action {
    ReadTechnique,
    GenerateTechnique,
    Migrate,
    Compile,
}
impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Action::ReadTechnique => "read technique",
                Action::GenerateTechnique => "generate technique",
                Action::Migrate => "migrate",
                Action::Compile => "compile",
            }
        )
    }
}
impl fmt::Debug for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Action::ReadTechnique => "Technique reading",
                Action::GenerateTechnique => "Technique generation",
                Action::Migrate => "Migration",
                Action::Compile => "Compilation",
            }
        )
    }
}
