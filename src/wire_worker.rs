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
use std::sync::mpsc;

use self::mix_link::sync::Session;
use self::mix_link::messages::{SessionConfig, PeerAuthenticator};
use self::mix_link::commands::Command;
use ecdh_wrapper::PrivateKey;


pub struct WireHandshakeWorker {
    session: Option<Arc<Mutex<Session>>>,
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
            session: None,
            chan: consumer_chan,
        }
    }

    pub fn on_stream(&mut self, stream: TcpStream) {
        let mut session = Session::new(self.session_config.take().unwrap(), false).unwrap();
        session.initialize(stream).unwrap();
        session = session.into_transport_mode().unwrap();
        session.finalize_handshake().unwrap();
        self.session = Some(Arc::new(Mutex::new(session)));
        let s = self.session.take().unwrap().clone();
        self.chan.send(self.session.take().unwrap());
    }
}


pub struct WireWorker {
    reader_chan: mpsc::SyncSender<Command>,
    writer_chan: mpsc::Receiver<Command>,
}

impl WireWorker {
    pub fn new(reader_chan: mpsc::SyncSender<Command>, writer_chan: mpsc::Receiver<Command>) -> WireWorker {
        WireWorker{
            writer_chan: writer_chan,
            reader_chan: reader_chan,
        }
    }

    pub fn reader(&self, session: Arc<Mutex<Session>>) {
        loop {
            let cmd = session.lock().unwrap().recv_command().unwrap();
            self.reader_chan.send(cmd);
        }
    }

    pub fn writer(&self, session: Arc<Mutex<Session>>) {
        loop {
            let cmd = self.writer_chan.recv().unwrap();
            session.lock().unwrap().send_command(&cmd).unwrap();
        }
    }
}
