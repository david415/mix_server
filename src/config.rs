// config.rs - Mix server configuration.
// Copyright (C) 2018  David Anthony Stainton.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::fs::File;
use std::io::prelude::*;
use toml;

use super::errors::ConfigError;


#[derive(Debug, Deserialize, Serialize)]
pub struct Logging {
    pub disable: bool,
    pub log_file: String,
    pub level: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Server {
    pub identifier: String,
    pub addresses: Vec<String>,
    pub data_dir: String,
    pub is_provider: bool,
    pub num_listeners: u16,
    pub num_sphinx_workers: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Nonvoting {
    address: String,
    public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Peer {
    addresses: Vec<String>,
    identity_public_key: String,
    link_public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Voting {
    peers: Vec<Peer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pki {
    nonvoting: Option<Nonvoting>,
    voting: Option<Voting>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub logging: Logging,
    pub server: Server,
    pub pki: Pki,
}

impl Config {
    pub fn load(file: String) -> Result<Config, ConfigError> {
        let mut file = File::open(file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }
}
