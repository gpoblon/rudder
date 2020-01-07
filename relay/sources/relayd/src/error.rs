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

use crate::data::node::NodeId;
use chrono;
use diesel;
use serde_json;
use std::{io, num, path::PathBuf};
use thiserror::Error;
use toml;

#[derive(Debug, Error)]
pub enum Error {
    #[error("invalid run log: {0}")]
    InvalidRunLog(String),
    #[error("invalid run info: {0}")]
    InvalidRunInfo(String),
    #[error("file name should be valid unicode")]
    InvalidFileName,
    #[error("received path {0:?} is not a file")]
    InvalidFile(PathBuf),
    #[error("inconsistent run log")]
    InconsistentRunlog,
    #[error("empty run log")]
    EmptyRunlog,
    #[error("missing id in certificate")]
    MissingIdInCertificate,
    #[error("certificate for unknown node: {0}")]
    CertificateForUnknownNode(NodeId),
    #[error("missing certificate for node: {0}")]
    MissingCertificateForNode(NodeId),
    #[error("database error: {0}")]
    Database(#[from] diesel::result::Error),
    #[error("database connection error: {0}")]
    DatabaseConnection(#[from] diesel::ConnectionError),
    #[error("database pool error: {0}")]
    Pool(#[from] diesel::r2d2::PoolError),
    #[error("I/O error: {0}")]
    Io(#[from] io::Error),
    #[error("configuration parsing error: {0}")]
    ConfigurationParsing(#[from] toml::de::Error),
    #[error("date parsing error: {0}")]
    DateParsing(#[from] chrono::ParseError),
    #[error("json parsing error: {0}")]
    JsonParsing(#[from] serde_json::Error),
    #[error("integer parsing error: {0}")]
    IntegerParsing(#[from] num::ParseIntError),
    #[error("UTF-8 decoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("UTF-8 decoding error: {0}")]
    StrUtf8(#[from] std::str::Utf8Error),
    #[error("SSL error: {0}")]
    Ssl(#[from] openssl::error::ErrorStack),
    #[error("invalid condition: {condition:}, should match {condition_regex:}")]
    InvalidCondition {
        condition: String,
        condition_regex: &'static str,
    },
    #[error("invalid condition: {condition:}, should have less then {max_length:} chars")]
    MaxLengthCondition {
        condition: String,
        max_length: usize,
    },
    #[error("boolean parsing error: {0}")]
    ParseBoolean(#[from] std::str::ParseBoolError),
    #[error("log format error: {0}")]
    LogFormat(#[from] tracing_subscriber::reload::Error),
    #[error("global logger setting error: {0}")]
    GlobalLogger(#[from] tracing::dispatcher::SetGlobalDefaultError),
    #[error("logger setting error: {0}")]
    SetLogLogger(#[from] log::SetLoggerError),
    #[error("missing target nodes")]
    MissingTargetNodes,
    #[error("invalid hash type provided {invalid:} (available hash types: {valid:})")]
    InvalidHashType {
        invalid: String,
        valid: &'static str,
    },
    #[error("invalid log filter: {0}")]
    InvalidLogFilter(#[from] tracing_subscriber::filter::ParseError),
    #[error("invalid header")]
    InvalidHeader,
    #[error("HTTP error: {0}")]
    HttpClient(#[from] reqwest::Error),
    #[error("Invalid duration: {0}")]
    InvalidDuration(#[from] humantime::DurationError),
}
