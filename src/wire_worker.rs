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

extern crate crossbeam;
extern crate crossbeam_utils;
extern crate crossbeam_thread;
extern crate crossbeam_channel;
extern crate ecdh_wrapper;
extern crate mix_link;

use std::sync::{Mutex, Arc, Barrier};
use std::net::TcpStream;
use std::thread as std_thread;

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


pub struct StaticAuthenticatorFactory {
    pub auth: PeerAuthenticator,
}

pub enum PeerAuthenticatorFactory {
    Static(StaticAuthenticatorFactory),
}

impl PeerAuthenticatorFactory {
    fn build(&self) -> PeerAuthenticator {
        match *self {
            PeerAuthenticatorFactory::Static(ref factory) => {
                factory.auth.clone()
            },
        }
    }
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

fn session_dispatcher(reader_tx: Sender<Session>, barrier: Arc<Barrier>, cfg: WireConfig, peer_auth_factory: PeerAuthenticatorFactory) {
    loop {
        barrier.wait();
        if let Ok(stream) = cfg.tcp_fount_rx.recv() {
            let session_config = SessionConfig{
                authenticator: peer_auth_factory.build(),
                authentication_key: cfg.link_private_key.clone(),
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
                debug!("server received command {:?}", cmd);
            } else {
                session.close();
            }
        } else {
            warn!("failed to recv session on reader_rx");
            return
        }
    }
}

pub fn start_wire_worker(cfg: WireConfig, peer_auth_factory: PeerAuthenticatorFactory) {
    std_thread::spawn(move || {
        start_wire_worker_runner(cfg, peer_auth_factory);
    });
}

pub fn start_wire_worker_runner(cfg: WireConfig, peer_auth_factory: PeerAuthenticatorFactory) {
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
    extern crate rand;
    extern crate ecdh_wrapper;
    extern crate mix_link;

    use std::{thread, time};
    use std::time::Duration;
    use std::collections::HashMap;
    use std::net::{TcpListener, TcpStream};
    use std::thread as std_thread;
    use self::rand::os::OsRng;
    use ecdh_wrapper::{PrivateKey, PublicKey};
    use mix_link::messages::{SessionConfig, PeerAuthenticator, ServerAuthenticatorState,
                             ClientAuthenticatorState};

    use super::super::wire_worker::{start_wire_worker};
    use super::*;

    #[test]
    fn basic_wire_worker_test() {
        let mut rng = OsRng::new().unwrap();
        let mix_priv_key = PrivateKey::generate(&mut rng).unwrap();
        let mix_pub_key = mix_priv_key.public_key().clone();
        let upstream_mix_priv_key = PrivateKey::generate(&mut rng).unwrap();
        let upstream_mix_pub_key = upstream_mix_priv_key.clone();

        let mut upstream_server_auth = ServerAuthenticatorState::default();
        upstream_server_auth.mix_map = HashMap::new();
        upstream_server_auth.mix_map.entry(mix_pub_key).or_insert(true);
        let upstream_auth = PeerAuthenticator::Server(upstream_server_auth);

        let mut mix_map: HashMap<PublicKey, bool> = HashMap::new();
        mix_map.insert(upstream_mix_priv_key.public_key(), true);
        let peer_auth = PeerAuthenticator::Server(ServerAuthenticatorState{
            mix_map: mix_map,
        });
        let static_auth_factory = StaticAuthenticatorFactory {
            auth: peer_auth.clone(),
        };
        let factory = PeerAuthenticatorFactory::Static(static_auth_factory);

        let (tcp_fount_tx, tcp_fount_rx) = unbounded();
        let (crypto_worker_tx, crypto_worker_rx) = unbounded();
        let cfg = WireConfig {
            link_private_key: mix_priv_key,
            tcp_fount_rx: tcp_fount_rx,
            crypto_worker_tx: crypto_worker_tx,
        };
        start_wire_worker(cfg, factory);

        let mix_addr = String::from("127.0.0.1:34578");
        let listener = TcpListener::bind(mix_addr.clone()).unwrap();
        let job_handle = Some(std_thread::spawn(move || {
            for maybe_stream in listener.incoming() {
                match maybe_stream {
                    Ok(stream) => {
                        tcp_fount_tx.send(stream).unwrap();
                    }
                    Err(_) => {
                        return;
                    }
                }
            }
        }));

        // client
        let client_config = SessionConfig {
            authenticator: upstream_auth,
            authentication_key: upstream_mix_priv_key,
            peer_public_key: Some(mix_pub_key),
            additional_data: vec![],
        };
        let mut session = Session::new(client_config, true).unwrap();

        let stream = TcpStream::connect(mix_addr.clone()).expect("connection failed");
        session.initialize(stream).unwrap();
        session = session.into_transport_mode().unwrap();
        session.finalize_handshake().unwrap();
        println!("client handshake completed!");

        thread::sleep(Duration::from_secs(1));
        let cmd = Command::NoOp{};
        session.send_command(&cmd).unwrap();
        session.send_command(&cmd).unwrap();
        session.send_command(&cmd).unwrap();
    }
}
