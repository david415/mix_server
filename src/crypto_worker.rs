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

use std::thread;
use packet::Packet;
use crossbeam_channel::Receiver;


pub fn start_crypto_worker(crypto_worker_rx: Receiver<Packet>) {
    thread::spawn(move || {
        crypto_worker(crypto_worker_rx)
    });
}

fn crypto_worker(crypto_worker_rx: Receiver<Packet>) {
    loop {
        let packet = match crypto_worker_rx.recv() {
            Ok(x) => x,
            Err(e) => {
                warn!("crypto worker aborting because read channel error: {}", e);
                return
            },
        };
        // packet.raw
        // sphinx_packet_unwrap
        // pub fn sphinx_packet_unwrap(private_key: &PrivateKey, packet: &mut [u8; PACKET_SIZE]) -> (Option<Vec<u8>>, Option<[u8; HASH_SIZE]>, Option<Vec<RoutingCommand>>, Option<SphinxUnwrapError>) {

    }
}
