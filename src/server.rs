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

use std::net::TcpStream;
use std::sync::mpsc;

use super::tcp_listener::TcpStreamFount;


pub struct Config {
    listen_addr: String,
    listener_pool_size: usize,
}

pub struct Server {
    config: Config,
    stream_fount: Option<TcpStreamFount>,
}

impl Server {

    pub fn new(cfg: Config) -> Server {
        Server {
            config: cfg,
            stream_fount: None,
        }
    }

    pub fn run(&mut self, handler: fn(TcpStream)) {
        let (snd, rcv) = mpsc::sync_channel(0);
        self.stream_fount = Some(TcpStreamFount::new(self.config.listen_addr.clone(), snd));
        self.stream_fount.take().unwrap().run();
    }
}
