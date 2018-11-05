// server.rs - Mix server.
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

use std::path::Path;
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;

use super::config::Config;


fn init_logger(log_dir: &str) {
    use log4rs::config::{Appender, Root};
    use log4rs::config::Config as Log4rsConfig;
    use log4rs::append::file::FileAppender;
    let log_path = Path::new(log_dir).join("mixnet_server.log");
    let file_appender = FileAppender::builder()
        .encoder(Box::new(PatternEncoder::new("{d} - {m}{n}")))
        .build(log_path)
        .unwrap();
    let config = Log4rsConfig::builder()
        .appender(Appender::builder().build("mixnet_server", Box::new(file_appender)))
        .build(Root::builder().appender("mixnet_server").build(LevelFilter::Info)) // XXX
        .unwrap();
    let _handle = log4rs::init_config(config).unwrap();
}

pub struct Server {
    cfg: Config,
}

impl Server {
    pub fn new(cfg: Config) -> Server {
        let s = Server {
            cfg: cfg,
        };
        init_logger(s.cfg.logging.log_file.as_str());
        s
    }

    pub fn run(&mut self) {
        info!("mixnet_server is still in pre-alpha. DO NOT DEPEND ON IT FOR STRONG SECURITY OR ANONYMITY.");
    }
}
