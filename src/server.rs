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

extern crate crossbeam_channel;
extern crate clear_on_drop;
extern crate ecdh_wrapper;
extern crate mix_link;


use std::path::Path;
use log4rs::encode::pattern::PatternEncoder;
use log::LevelFilter;
use crossbeam_channel::unbounded;
use clear_on_drop::ClearOnDrop;

use ecdh_wrapper::PrivateKey;
use self::mix_link::messages::{SessionConfig, PeerAuthenticator};

use super::config::Config;
use super::tcp_listener::TcpStreamFount;
use super::wire_worker::WireWorker;


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
    incoming_conn_founts: Vec<TcpStreamFount>,
    wire_workers: Vec<WireWorker>,
    peer_auth: PeerAuthenticator, // XXX
}

impl Server {
    pub fn new(cfg: Config, peer_auth: PeerAuthenticator) -> Server {
        let s = Server {
            cfg: cfg,
            incoming_conn_founts: vec![],
            wire_workers: vec![],
            peer_auth: peer_auth,
        };
        init_logger(s.cfg.logging.log_file.as_str());
        s
    }

    pub fn run(&mut self) {
        info!("mix_server is still in pre-alpha. DO NOT DEPEND ON IT FOR STRONG SECURITY OR ANONYMITY.");

        let data_dir_path = Path::new(&self.cfg.server.data_dir);
        let link_priv_path = data_dir_path.join("link.private.pem");
        let priv_file = link_priv_path.to_str().unwrap();
        let link_pub_path = data_dir_path.join("link.public.pem");
        let pub_file = link_pub_path.to_str().unwrap();
        let link_priv_key = match PrivateKey::from_pem_files(priv_file.to_string(), pub_file.to_string()) {
            Ok(x) => x,
            Err(e) => {
                error!("mix_server failed to load link keys: {}", e);
                return;
            },
        };
        //let clear_link_priv_key = ClearOnDrop::new(&mut link_priv_key);

        let (tcp_fount_tx, tcp_fount_rx) = unbounded();
        let (crypto_worker_tx, crypto_worker_rx) = unbounded();

        for address in self.cfg.server.addresses.clone() {
            let mut fount = TcpStreamFount::new(address, tcp_fount_tx.clone());
            fount.run();
            self.incoming_conn_founts.push(fount);
        }

        for _i in 1..self.cfg.server.num_wire_workers {
            let mut wire_worker = WireWorker::new(self.peer_auth.clone(), link_priv_key, tcp_fount_rx.clone(), crypto_worker_tx.clone());
            wire_worker.run();
            self.wire_workers.push(wire_worker);
        }
    }
}
