
// wire_worker.rs - Wire protocol worker.
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

extern crate mix_link;
extern crate ecdh_wrapper;

use std::sync::{Mutex, Arc};
use std::net::TcpStream;
use std::thread;
use std::thread::JoinHandle;
use std::sync::mpsc;

use self::mix_link::sync::Session;
use self::mix_link::messages::{SessionConfig, PeerAuthenticator};
use self::mix_link::commands::Command;
use ecdh_wrapper::PrivateKey;


pub struct WireHandshakeWorker {
    session_config: Option<SessionConfig>,
    chan: mpsc::SyncSender<Arc<Mutex<Session>>>,
}

impl WireHandshakeWorker {
    pub fn new(auth: PeerAuthenticator, server_keypair: PrivateKey, consumer_chan: mpsc::SyncSender<Arc<Mutex<Session>>>) -> WireHandshakeWorker {
        let session_config = SessionConfig {
            authenticator: auth,
            authentication_key: server_keypair,
            peer_public_key: None,
            additional_data: vec![],
        };
        WireHandshakeWorker{
            session_config: Some(session_config),
            chan: consumer_chan,
        }
    }

    fn on_stream(&mut self, stream: TcpStream) {
        let cfg = self.session_config.clone().unwrap();
        let mut session = Session::new(cfg.clone(), false).unwrap();
        session.initialize(stream).unwrap();
        session = session.into_transport_mode().unwrap();
        session.finalize_handshake().unwrap();
        self.chan.send(Arc::new(Mutex::new(session)));
    }
}


pub struct WireWorker {
    reader_chan: Arc<Mutex<mpsc::SyncSender<Command>>>,
    reader_handle: Option<JoinHandle<()>>,
    writer_chan: Arc<Mutex<mpsc::Receiver<Command>>>,
    writer_handle: Option<JoinHandle<()>>,
}

impl WireWorker {
    pub fn new(reader_chan: mpsc::SyncSender<Command>, writer_chan: mpsc::Receiver<Command>) -> WireWorker {
        WireWorker{
            writer_chan: Arc::new(Mutex::new(writer_chan)),
            writer_handle: None,
            reader_chan: Arc::new(Mutex::new(reader_chan)),
            reader_handle: None,
        }
    }

    pub fn reader(&mut self, session: Arc<Mutex<Session>>) {
        let ch = self.reader_chan.clone();
        self.reader_handle = Some(thread::spawn(move || {
            loop {
                let cmd = session.lock().unwrap().recv_command().unwrap();
                ch.lock().unwrap().send(cmd);
            }
        }));
    }

    pub fn writer(&mut self, session: Arc<Mutex<Session>>) {
        let ch = self.writer_chan.clone();
        self.writer_handle = Some(thread::spawn(move || {
            loop {
                let cmd = ch.lock().unwrap().recv().unwrap();
                session.lock().unwrap().send_command(&cmd).unwrap();
            }
        }));
    }
}
