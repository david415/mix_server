// tcp_listener.rs - Mix server tcp listener.
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

use std::thread;
use std::thread::JoinHandle;
use std::sync::{Mutex, Arc};
use std::net::{TcpListener, TcpStream};

use crossbeam_channel::Sender;


pub struct TcpStreamFount {
    listen_addr: String,
    stream_chan: Sender<TcpStream>,
    job_handle: Option<JoinHandle<()>>,
}

impl TcpStreamFount {
    pub fn new(listen_addr: String, chan: Sender<TcpStream>) -> TcpStreamFount {
        TcpStreamFount{
            listen_addr: listen_addr,
            stream_chan: chan,
            job_handle: None,
        }
    }

    pub fn run(&mut self) {
        let listener = TcpListener::bind(self.listen_addr.clone()).unwrap();
        let ch = self.stream_chan.clone();
        self.job_handle = Some(thread::spawn(move || {
            for maybe_stream in listener.incoming() {
                match maybe_stream {
                    Ok(stream) => {
                        if let Err(e) = ch.send(stream) {
                            warn!("send failure: {}", e);
                        }
                    }
                    Err(_) => {
                        return;
                    }
                }
            }
        }));
    }

    pub fn halt(&mut self) {
        drop(self.job_handle.take());
    }
}
