// packet.rs - Packet struct.
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

use std::time::{Duration, SystemTime, UNIX_EPOCH};


#[derive(Default)]
pub struct Packet {
    pub id: u64,
    pub raw: Vec<u8>,
    pub receive_time: u64,
}

impl Packet {
    pub fn new(raw: Vec<u8>) -> Self {
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(x) => x,
            Err(_) => {
                panic!("clock went back in time");
            },
        };
        let in_ms = now.as_secs() * 1000 +
            now.subsec_nanos() as u64 / 1_000_000;
        Packet{
            id: 0, // XXX - FIX ME
            raw: raw,
            receive_time: in_ms,
        }
    }
}
