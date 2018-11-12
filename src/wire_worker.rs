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
extern crate crossbeam;
extern crate crossbeam_utils;
extern crate crossbeam_thread;
extern crate crossbeam_channel;

use std::sync::{Mutex, Arc, Barrier};
use std::net::TcpStream;

use crossbeam_utils::thread;
use crossbeam::thread::{ScopedJoinHandle, ScopedThreadBuilder};
use crossbeam_channel::{Receiver, Sender, unbounded};

use self::mix_link::sync::Session;
use self::mix_link::errors::HandshakeError;
use self::mix_link::messages::{SessionConfig, PeerAuthenticator};
use self::mix_link::commands::Command;
use ecdh_wrapper::PrivateKey;

use packet::Packet;
use super::tcp_listener::TcpStreamFount;

#[derive(Clone)]
pub struct WireWorker {
    link_private_key: PrivateKey,
    tcp_fount_rx: Receiver<TcpStream>,
    reader_tx: Sender<Session>,
    reader_rx: Receiver<Session>,
    writer_tx: Sender<Session>,
    writer_rx: Receiver<Session>,
    crypto_worker_tx: Sender<Packet>,
    barrier: Arc<Barrier>,
    peer_auth: PeerAuthenticator
}

impl WireWorker {
    pub fn new(peer_auth: PeerAuthenticator, link_private_key: PrivateKey, tcp_fount_rx: Receiver<TcpStream>, crypto_worker_tx: Sender<Packet>) -> WireWorker {
        let (reader_tx, reader_rx) = unbounded();
        let (writer_tx, writer_rx) = unbounded();

        WireWorker{
            link_private_key: link_private_key,
            tcp_fount_rx: tcp_fount_rx,
            crypto_worker_tx: crypto_worker_tx,
            barrier: Arc::new(Barrier::new(3)),
            peer_auth: peer_auth,
            reader_tx: reader_tx,
            reader_rx: reader_rx,
            writer_tx: writer_tx,
            writer_rx: writer_rx,
        }
    }

    pub fn create_session(&self, session_config: SessionConfig, stream: TcpStream) -> Result<Session, HandshakeError> {
        let mut session = Session::new(session_config, false)?;
        session.initialize(stream)?;
        session = session.into_transport_mode()?;
        session.finalize_handshake()?;
        Ok(session)
    }

    fn get_authenticator(&mut self) -> PeerAuthenticator {
        self.peer_auth.clone()
    }

    pub fn stream_dispatcher(&mut self) {
        loop {
            if let Ok(stream) = self.tcp_fount_rx.recv() {
                let authenticator = self.get_authenticator();
                let session_config = SessionConfig{
                    authenticator: authenticator,
                    authentication_key: self.link_private_key,
                    peer_public_key: None,
                    additional_data: vec![],
                };
                let session = match self.create_session(session_config, stream) {
                    Ok(x) => x,
                    Err(e) => {
                        error!("session creation failure: {}", e);
                        self.halt();
                        return
                    }
                };
                let session_writer = session.clone();
                if let Err(e) = self.reader_tx.send(session) {
                    warn!("shutting down wire worker because of a failure to dispatch session to reader thread: {}", e);
                    self.halt();
                    return
                }
                if let Err(e) = self.writer_tx.send(session_writer) {
                    warn!("shutting down wire worker because of a failure to dispatch session to writer thread: {}", e);
                    self.halt();
                    return
                }
                let barrier = self.barrier.clone();
                barrier.wait();
            } else {
                warn!("fount chan recv failure, halting wire worker.");
                self.halt();
                return
            }
        }
    }

    pub fn reader(&mut self) {}

    pub fn writer(&mut self) {}

    pub fn run(&mut self) {
        if let Err(_) = thread::scope(|scope| {
            let mut worker = self.clone();
            let mut reader = self.clone();
            let mut writer = self.clone();
            let dispatcher_handle = Some(scope.spawn(move |_| {
                worker.stream_dispatcher();
            }));
            let reader_handle = Some(scope.spawn(move |_| {
                reader.reader();
            }));
            let writer_handle = Some(scope.spawn(move |_| {
                writer.writer();
            }));

        }) {
            warn!("wire worker failed to spawn thread(s)");
        }
    }

    pub fn halt(&mut self) {
        // XXX fix me
        //drop(self.writer_handle.take());
        //drop(self.reader_handle.take());
    }
}
