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


pub struct WireWorker {
    session_config: Option<SessionConfig>,
    reader_chan: Arc<Mutex<mpsc::SyncSender<Command>>>,
    reader_handle: Option<JoinHandle<()>>,
    writer_chan: Arc<Mutex<mpsc::Receiver<Command>>>,
    writer_handle: Option<JoinHandle<()>>,
}

impl WireWorker {
    pub fn new(auth: PeerAuthenticator, server_keypair: PrivateKey, reader_chan: mpsc::SyncSender<Command>, writer_chan: mpsc::Receiver<Command>) -> WireWorker {
        let session_config = SessionConfig {
            authenticator: auth,
            authentication_key: server_keypair,
            peer_public_key: None,
            additional_data: vec![],
        };
        WireWorker{
            session_config: Some(session_config),
            writer_chan: Arc::new(Mutex::new(writer_chan)),
            writer_handle: None,
            reader_chan: Arc::new(Mutex::new(reader_chan)),
            reader_handle: None,
        }
    }

    pub fn on_stream(mut self, stream: TcpStream) {
        let cfg = self.session_config.clone().unwrap();
        let mut session = Session::new(cfg.clone(), false).unwrap();
        session.initialize(stream).unwrap();
        session = session.into_transport_mode().unwrap();
        session.finalize_handshake().unwrap();
        let writer_session = Arc::new(session);
        let reader_session = writer_session.clone();
        let reader_ch = self.reader_chan.clone();
        self.reader_handle = Some(thread::spawn(move || {
            loop {
                let cmd = reader_session.recv_command().unwrap();
                reader_ch.lock().unwrap().send(cmd);
            }
        }));
        let writer_ch = self.writer_chan.clone();
        self.writer_handle = Some(thread::spawn(move || {
            loop {
                let cmd = writer_ch.lock().unwrap().recv().unwrap();
                writer_session.send_command(&cmd).unwrap();
            }
        }));
    }

    pub fn halt(&mut self) {
        drop(self.writer_handle.take());
        drop(self.reader_handle.take());
    }
}
