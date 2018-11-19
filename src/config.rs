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
    pub num_wire_workers: u16,
    pub num_sphinx_workers: u16,
    pub num_crypto_workers: u16,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Nonvoting {
    pub address: String,
    pub public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Peer {
    pub addresses: Vec<String>,
    pub identity_public_key: String,
    pub link_public_key: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Voting {
    pub epoch_duration: u64,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Pki {
    pub nonvoting: Option<Nonvoting>,
    pub voting: Option<Voting>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Config {
    pub logging: Logging,
    pub server: Server,
    pub pki: Pki,
}

impl Config {
    pub fn load(contents: String) -> Result<Config, ConfigError> {
        let config: Config = toml::from_str(&contents)?;
        Ok(config)
    }

    pub fn load_file(file: String) -> Result<Config, ConfigError> {
        let mut file = File::open(file)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        Ok(Config::load(contents)?)
    }

    pub fn store(&self, file_name: String) -> Result<(), ConfigError> {
        let mut file = File::create(file_name)?;
        let toml_config = toml::to_string(&self).unwrap();
        file.write_all(toml_config.as_bytes())?;
        Ok(())
    }
}
