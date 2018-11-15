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

use ecdh_wrapper::PrivateKey;
use mix_link::sync::Session;
use mix_link::errors::HandshakeError;
use mix_link::messages::{SessionConfig, PeerAuthenticator};
use mix_link::commands::Command;

use packet::Packet;
use super::tcp_listener::TcpStreamFount;


pub trait PeerAuthenticatorFactory {
    fn build(&self) -> PeerAuthenticator;
}

#[derive(Clone)]
pub struct WireConfig {
    pub link_private_key: PrivateKey,
    pub tcp_fount_rx: Receiver<TcpStream>,
    pub crypto_worker_tx: Sender<Packet>,
}

fn create_session(session_config: SessionConfig, stream: TcpStream) -> Result<Session, HandshakeError> {
    let mut session = Session::new(session_config, false)?;
    session.initialize(stream)?;
    session = session.into_transport_mode()?;
    session.finalize_handshake()?;
    Ok(session)
}

fn session_dispatcher<T: PeerAuthenticatorFactory + Send>(reader_tx: Sender<Session>, barrier: Arc<Barrier>, cfg: WireConfig, peer_auth_factory: T) {
    loop {
        barrier.wait();
        if let Ok(stream) = cfg.tcp_fount_rx.recv() {
            let session_config = SessionConfig{
                authenticator: peer_auth_factory.build(),
                authentication_key: cfg.link_private_key,
                peer_public_key: None,
                additional_data: vec![],
            };
            let session = match create_session(session_config, stream) {
                Ok(x) => x,
                Err(e) => {
                    warn!("failed to create noise session: {}", e);
                    continue
                },
            };
            if let Err(e) = reader_tx.send(session) {
                warn!("shutting down wire worker because of a failure to dispatch session to reader thread: {}", e);
                return
            }
        } else {
            warn!("fount chan recv failure, halting wire worker.");
            return
        }
    } // end of loop {
}

fn reader(reader_rx: Receiver<Session>, barrier: Arc<Barrier>) {
    loop {
        barrier.wait();
        if let Ok(mut session) = reader_rx.recv() {
            if let Ok(cmd) = session.recv_command() {
                println!("server received command {:?}", cmd);
            } else {
                session.close();
            }
        } else {
            warn!("failed to recv session on reader_rx");
            return
        }
    }
}

pub fn start_wire_worker<T: PeerAuthenticatorFactory + Send>(cfg: WireConfig, peer_auth_factory: T) {
    let barrier = Arc::new(Barrier::new(2));
    let (reader_tx, reader_rx) = unbounded();
    let dispatcher_barrier = barrier.clone();
    let reader_barrier = barrier.clone();

    if let Err(_) = thread::scope(|scope| {
        let mut thread_handles = vec![];
        thread_handles.push(Some(scope.spawn(move |_| {
            session_dispatcher(reader_tx, dispatcher_barrier, cfg, peer_auth_factory);
        })));
        thread_handles.push(Some(scope.spawn(move |_| {
            reader(reader_rx, reader_barrier);
        })));
    }) {
        warn!("wire worker failed to spawn thread(s)");
        return
    }
}


#[cfg(test)]
mod tests {

    #[test]
    fn basic_wire_worker_test() {


        //start_wire_worker()
    }
}
