// server.rs - The mix server.
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

extern crate threadpool;

use threadpool::ThreadPool;
use std::thread::spawn;
use std::sync::mpsc::channel;
use std::net::{TcpListener, TcpStream};

#[derive(Copy, Clone)]
pub struct TcpStreamHandler {
    handler: fn(TcpStream)
}

impl TcpStreamHandler {
    pub fn new(handler: fn(TcpStream)) -> TcpStreamHandler {
        TcpStreamHandler{
            handler: handler,
        }
    }

    fn handle_stream(&self, stream: TcpStream) {
        (self.handler)(stream);
    }
}


pub struct TcpStreamFount {
    thread_pool: Option<ThreadPool>,
    listen_addr: String,
    tcp_listener: Option<TcpListener>,
    stream_handler: TcpStreamHandler,
}

impl TcpStreamFount {
    pub fn new(listen_addr: String, pool_size: usize, stream_handler: TcpStreamHandler) -> TcpStreamFount {
        let f = TcpStreamFount{
            thread_pool: Some(ThreadPool::new(pool_size)),
            listen_addr: listen_addr,
            tcp_listener: None,
            stream_handler: stream_handler,
        };
        f
    }

    pub fn run(&mut self) {
        let listener = TcpListener::bind(self.listen_addr.clone()).unwrap();
        let handler = self.stream_handler;
        for maybe_stream in listener.incoming() {
            match maybe_stream {
                Ok(stream) => {
                    self.thread_pool.take().unwrap().execute(move || {
                        handler.handle_stream(stream);
                    });
                }
                Err(_) => {
                    return;
                }
            }
        }
    }

    pub fn halt(&mut self) {
        drop(self.thread_pool.take());
    }
}
