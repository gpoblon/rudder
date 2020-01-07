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

use crate::{configuration::Secret, data::node::NodeId, error::Error};
use serde::{
    de::{Deserializer, Error as SerdeError, Unexpected, Visitor},
    Deserialize,
};
use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt,
    fs::read_to_string,
    net::SocketAddr,
    path::{Path, PathBuf},
    str::FromStr,
    time::Duration,
};
use toml;
use tracing::debug;

pub type BaseDirectory = PathBuf;
pub type WatchedDirectory = PathBuf;
pub type NodesListFile = PathBuf;
pub type NodesCertsFile = PathBuf;

// For compatibility with int fields containing an integer
fn compat_humantime<'de, D>(d: D) -> Result<Duration, D::Error>
where
    D: Deserializer<'de>,
{
    struct V;

    impl<'de2> Visitor<'de2> for V {
        type Value = Duration;

        fn expecting(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
            fmt.write_str("a duration")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Duration, E>
        where
            E: SerdeError,
        {
            u64::try_from(v)
                .map(|s| Duration::from_secs(s))
                .map_err(|_| E::invalid_value(Unexpected::Signed(v), &self))
        }

        fn visit_str<E>(self, v: &str) -> Result<Duration, E>
        where
            E: SerdeError,
        {
            humantime::parse_duration(v).map_err(|_| E::invalid_value(Unexpected::Str(v), &self))
        }
    }

    d.deserialize_str(V)
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
// Default can be implemented in serde using the Default trait
pub struct Configuration {
    pub general: GeneralConfig,
    pub processing: ProcessingConfig,
    pub output: OutputConfig,
    pub remote_run: RemoteRun,
    pub shared_files: SharedFiles,
    pub shared_folder: SharedFolder,
}

impl Configuration {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
        let res = read_to_string(path.as_ref().join("main.conf"))?.parse::<Self>();
        if let Ok(ref cfg) = res {
            debug!("Parsed main configuration:\n{:#?}", &cfg);
        }
        res
    }
}

impl FromStr for Configuration {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(toml::from_str(s)?)
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct GeneralConfig {
    pub nodes_list_file: NodesListFile,
    pub nodes_certs_file: NodesCertsFile,
    pub node_id: NodeId,
    pub listen: SocketAddr,
    /// None means using the number of available CPUs
    pub core_threads: Option<usize>,
    pub blocking_threads: usize,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct CatchupConfig {
    #[serde(deserialize_with = "compat_humantime")]
    #[serde(default = "default_catchup_frequency")]
    pub frequency: Duration,
    #[serde(default = "default_catchup_limit")]
    pub limit: u64,
}

fn default_catchup_frequency() -> Duration {
    Duration::from_secs(10)
}

fn default_catchup_limit() -> u64 {
    50
}

#[derive(Deserialize, Debug, PartialEq, Eq, Copy, Clone)]
pub struct CleanupConfig {
    #[serde(deserialize_with = "compat_humantime")]
    #[serde(default = "default_cleanup_frequency")]
    pub frequency: Duration,
    #[serde(deserialize_with = "compat_humantime")]
    #[serde(default = "default_cleanup_retention")]
    pub retention: Duration,
}

/// 1 hour
fn default_cleanup_frequency() -> Duration {
    Duration::from_secs(3600)
}

/// 1 week
fn default_cleanup_retention() -> Duration {
    Duration::from_secs(3600 * 24 * 7)
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ProcessingConfig {
    pub inventory: InventoryConfig,
    pub reporting: ReportingConfig,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct InventoryConfig {
    pub directory: BaseDirectory,
    pub output: InventoryOutputSelect,
    pub catchup: CatchupConfig,
    pub cleanup: CleanupConfig,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum InventoryOutputSelect {
    Upstream,
    Disabled,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct ReportingConfig {
    pub directory: BaseDirectory,
    pub output: ReportingOutputSelect,
    pub catchup: CatchupConfig,
    pub cleanup: CleanupConfig,
    pub skip_event_types: HashSet<String>,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ReportingOutputSelect {
    Database,
    Upstream,
    Disabled,
}

pub trait OutputSelect {
    fn is_enabled(&self) -> bool;
}

impl OutputSelect for ReportingOutputSelect {
    fn is_enabled(&self) -> bool {
        *self != ReportingOutputSelect::Disabled
    }
}

impl OutputSelect for InventoryOutputSelect {
    fn is_enabled(&self) -> bool {
        *self != InventoryOutputSelect::Disabled
    }
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct RemoteRun {
    pub command: PathBuf,
    pub use_sudo: bool,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SharedFiles {
    pub path: PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct SharedFolder {
    pub path: PathBuf,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct OutputConfig {
    pub database: DatabaseConfig,
    pub upstream: UpstreamConfig,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct DatabaseConfig {
    /// URL without the password
    pub url: String,
    pub password: Secret,
    pub max_pool_size: u32,
}

#[derive(Deserialize, Debug, PartialEq, Eq, Clone)]
pub struct UpstreamConfig {
    // TODO better URL type
    pub url: String,
    pub user: String,
    pub password: Secret,
    pub verify_certificates: bool,
    // TODO timeout?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_fails_with_configuration() {
        let empty = "";
        let config = empty.parse::<Configuration>();
        assert!(config.is_err());
    }

    #[test]
    fn it_parses_main_configuration() {
        let config = Configuration::new("tests/files/config/");

        let reference = Configuration {
            general: GeneralConfig {
                nodes_list_file: PathBuf::from("tests/files/nodeslist.json"),
                nodes_certs_file: PathBuf::from("tests/files/keys/nodescerts.pem"),
                node_id: "root".to_string(),
                listen: "127.0.0.1:3030".parse().unwrap(),
                core_threads: None,
                blocking_threads: 100,
            },
            processing: ProcessingConfig {
                inventory: InventoryConfig {
                    directory: PathBuf::from("target/tmp/inventories/"),
                    output: InventoryOutputSelect::Upstream,
                    catchup: CatchupConfig {
                        frequency: Duration::from_secs(10),
                        limit: 50,
                    },
                    cleanup: CleanupConfig {
                        frequency: Duration::from_secs(10),
                        retention: Duration::from_secs(10),
                    },
                },
                reporting: ReportingConfig {
                    directory: PathBuf::from("target/tmp/reporting/"),
                    output: ReportingOutputSelect::Database,
                    catchup: CatchupConfig {
                        frequency: Duration::from_secs(10),
                        limit: 50,
                    },
                    cleanup: CleanupConfig {
                        frequency: Duration::from_secs(30),
                        retention: Duration::from_secs(30 * 60 + 20),
                    },
                    skip_event_types: HashSet::new(),
                },
            },
            output: OutputConfig {
                upstream: UpstreamConfig {
                    url: "https://127.0.0.1:8080".to_string(),
                    user: "rudder".to_string(),
                    password: Secret::new("password".to_string()),
                    verify_certificates: false,
                },
                database: DatabaseConfig {
                    url: "postgres://rudderreports@127.0.0.1/rudder".to_string(),
                    password: Secret::new("PASSWORD".to_string()),
                    max_pool_size: 5,
                },
            },
            remote_run: RemoteRun {
                command: PathBuf::from("tests/api_remote_run/fake_agent.sh"),
                use_sudo: false,
            },
            shared_files: SharedFiles {
                path: PathBuf::from("tests/api_shared_files"),
            },
            shared_folder: SharedFolder {
                path: PathBuf::from("tests/api_shared_folder"),
            },
        };
        assert_eq!(config.unwrap(), reference);
    }
}
