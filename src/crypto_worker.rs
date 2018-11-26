// crypto_worker.rs - Mix crypto worker.
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
extern crate sphinx_replay_cache;

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use packet::Packet;
use crossbeam_channel::{Receiver, Select};
use sphinx_replay_cache::MixKeys;


pub struct CryptoWorkerConfig {
    pub crypto_worker_rx: Receiver<Packet>,
    pub update_rx: Receiver<bool>,
    pub halt_rx: Receiver<bool>,
    pub slack_time: u64,
}

pub fn start_crypto_worker(cfg: CryptoWorkerConfig) {
    thread::spawn(move || {
        crypto_worker(cfg)
    });
}

fn crypto_worker(cfg: CryptoWorkerConfig) {
    let mut sel = Select::new();
    let oper1 = sel.recv(&cfg.crypto_worker_rx);
    let oper2 = sel.recv(&cfg.update_rx);
    let oper3 = sel.recv(&cfg.halt_rx);
    loop {
        let mut packet = Packet::default();
        let oper = sel.select();
        match oper.index() {
            i if i == oper1 => {
                packet = match oper.recv(&cfg.crypto_worker_rx) {
                    Ok(x) => x,
                    Err(e) => {
                        warn!("crypto worker failed to receive packet: {}", e);
                        return
                    },
                };

            },
            i if i == oper2 => {
                oper.recv(&cfg.update_rx);
                // XXX not yet implemented
                continue
            },
            i if i == oper3 => {
                oper.recv(&cfg.halt_rx);
                return
            },
            _ => unreachable!(),
        }

        // Drop the packet if it has been sitting in the queue waiting to
	// be decrypted for way too long.
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(x) => x,
            Err(e) => {
                warn!("crypto worker failed to read time from clock: {}", e);
                return
            },
        };
        let dwell_time = now - Duration::from_millis(packet.receive_time);
        if dwell_time > Duration::from_millis(cfg.slack_time) {
            debug!("dropping packet, dwelled too long.");
            continue
        } else {
            debug!("crypto worker packet queue delay {:?}", dwell_time);
        }

        // XXX ...
    }
}
