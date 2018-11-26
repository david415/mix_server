// errors.rs - Mix server errors.
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

//! Mix server error types.

use std::io::Error as IoError;
use std::error::Error;
use std::fmt;
use toml;

use ecdh_wrapper::errors::KeyError;


#[derive(Debug)]
pub enum ConfigError {
    IoError(std::io::Error),
    TomlError(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ConfigError::*;
        match self {
            IoError(x) => x.fmt(f),
            TomlError(x) => x.fmt(f),
        }
    }
}

impl Error for ConfigError {
    fn description(&self) -> &str {
        "I'm a ConfigError."
    }

    fn cause(&self) -> Option<&Error> {
        use self::ConfigError::*;
        match self {
            IoError(x) => x.cause(),
            TomlError(x) => x.cause(),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(error: std::io::Error) -> Self {
        ConfigError::IoError(error)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(error: toml::de::Error) -> Self {
        ConfigError::TomlError(error)
    }
}

#[derive(Debug)]
pub enum MixKeyError {
    CreateCacheFailed,
    LoadCacheFailed,
    KeyError(KeyError),
    IoError(IoError),
}

impl fmt::Display for MixKeyError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::MixKeyError::*;
        match self {
            CreateCacheFailed => write!(f, "Failed to create cache."),
            LoadCacheFailed => write!(f, "Failed to load cache."),
            KeyError(x) => x.fmt(f),
            IoError(x) => x.fmt(f),
        }
    }
}

impl Error for MixKeyError {
    fn description(&self) -> &str {
        "I'm a MixKeyError."
    }

    fn cause(&self) -> Option<&Error> {
        use self::MixKeyError::*;
        match self {
            CreateCacheFailed => None,
            LoadCacheFailed => None,
            KeyError(x) => x.cause(),
            IoError(x) => x.cause(),
        }
    }
}

impl From<KeyError> for MixKeyError {
    fn from(error: KeyError) -> Self {
        MixKeyError::KeyError(error)
    }
}

impl From<IoError> for MixKeyError {
    fn from(error: IoError) -> Self {
        MixKeyError::IoError(error)
    }
}

#[derive(Debug)]
pub enum UnwrapPacketError {
    NoKey,
    CacheFail,
    Replay,
}

impl fmt::Display for UnwrapPacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::UnwrapPacketError::*;
        match self {
            NoKey => write!(f, "no mix key found"),
            CacheFail => write!(f, "cache failure"),
            Replay => write!(f, "sphinx packet replay detected"),
        }
    }
}

impl Error for UnwrapPacketError {
    fn description(&self) -> &str {
        "I'm a unwrap packet error."
    }

    fn cause(&self) -> Option<&Error> {
        use self::UnwrapPacketError::*;
        match self {
            NoKey => None,
            CacheFail => None,
            Replay => None,
        }
    }
}

#[derive(Debug)]
pub enum PacketError {
    WrongSize,
}

impl fmt::Display for PacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::PacketError::*;
        match self {
            WrongSize => write!(f, ""),
        }
    }
}

impl Error for PacketError {
    fn description(&self) -> &str {
        "I'm a unwrap packet error."
    }

    fn cause(&self) -> Option<&Error> {
        use self::PacketError::*;
        match self {
            WrongSize => None,
        }
    }
}
