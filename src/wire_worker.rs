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
extern crate crossbeam_channel;
extern crate crossbeam_thread;
extern crate ecdh_wrapper;
extern crate mix_link;

use std::sync::{Arc, Barrier};
use std::net::TcpStream;
use std::thread as std_thread;

use crossbeam_utils::thread;
use crossbeam_channel::{Receiver, Sender, unbounded};

use ecdh_wrapper::PrivateKey;
use mix_link::sync::Session;
use mix_link::errors::HandshakeError;
use mix_link::messages::{SessionConfig, PeerAuthenticator};
use mix_link::commands::Command;

use packet::Packet;

#[derive(PartialEq, Debug, Clone)]
pub struct StaticAuthenticatorBuilder {
    pub auth: PeerAuthenticator,
}

#[derive(PartialEq, Debug, Clone)]
pub enum PeerAuthenticatorBuilder {
    Static(StaticAuthenticatorBuilder),
}

impl PeerAuthenticatorBuilder {
    fn build(&self) -> PeerAuthenticator {
        match *self {
            PeerAuthenticatorBuilder::Static(ref builder) => {
                builder.auth.clone()
            },
        }
    }
}


#[derive(Clone)]
pub struct WireConfig {
    pub link_private_key: PrivateKey,
    pub tcp_fount_rx: Receiver<TcpStream>,
    pub crypto_worker_tx: Sender<Packet>,
    pub peer_auth_builder: PeerAuthenticatorBuilder,
    pub is_provider: bool,
}

fn create_session(session_config: SessionConfig, stream: TcpStream) -> Result<Session, HandshakeError> {
    let mut session = Session::new(session_config, false)?;
    session.initialize(stream)?;
    session = session.into_transport_mode()?;
    session.finalize_handshake()?;
    Ok(session)
}

fn session_dispatcher(reader_tx: Sender<Session>, barrier: Arc<Barrier>, cfg: WireConfig) {
    loop {
        barrier.wait();
        if let Ok(stream) = cfg.tcp_fount_rx.recv() {
            let session_config = SessionConfig{
                authenticator: cfg.peer_auth_builder.build(),
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

fn reader(reader_rx: Receiver<Session>, crypto_worker_tx: Sender<Packet>, barrier: Arc<Barrier>, is_provider: bool) {
    loop {
        barrier.wait();
        let mut session = match reader_rx.recv() {
            Ok(x) => x,
            Err(e) => {
                warn!("failed to recv session on reader_rx: {}", e);
                return
            },
        };

        loop {
            let cmd = match session.recv_command() {
                Ok(x) => x,
                Err(_) => {
                    session.close();
                    break
                },
            };
            debug!("server received command {:?}", cmd);

            if session.from_client() {
                match &cmd {
                    Command::RetrieveMessage {
                        sequence,
                    } => {
                        debug!("Received RetrieveMessage from peer.");
                        on_retrieve_message(&cmd);
                        continue
                    },
                    Command::GetConsensus {
                        epoch,
                    } => {
                        debug!("Received GetConsensus from peer.");
                        on_get_consensus(&cmd);
                        continue
                    },
                    _ => {},
                }
            }

            match &cmd {
                Command::NoOp{} => {
                    debug!("NoOp received!");
                },
                Command::SendPacket {
                    sphinx_packet
                } => {
                    let mut packet = match Packet::new(sphinx_packet) {
                        Ok(x) => x,
                        Err(e) => {
                            warn!("invalid sphinx packet: {}", e);
                            continue
                        },
                    };
                    packet.must_forward = session.from_client();
                    packet.must_terminate = is_provider && !session.from_client();
                    // XXX fixme: use select statement instead of single channel usage
                    if let Err(e) = crypto_worker_tx.send(packet) {
                        warn!("failed to send to crypto worker channel: {}", e);
                        return
                    }
                },
                Command::Disconnect{} => {

                },
                _ => {
                    debug!("received unhandled command");
                    continue
                }
            } // match cmd {
        }
    }
}

pub fn start_wire_worker(cfg: WireConfig) {
    std_thread::spawn(move || {
        start_wire_worker_runner(cfg);
    });
}

pub fn start_wire_worker_runner(cfg: WireConfig) {
    let barrier = Arc::new(Barrier::new(2));
    let (reader_tx, reader_rx) = unbounded();
    let dispatcher_barrier = barrier.clone();
    let reader_barrier = barrier.clone();
    let crypto_worker_tx = cfg.crypto_worker_tx.clone();
    let is_provider = cfg.is_provider;

    if let Err(_) = thread::scope(|scope| {
        let mut thread_handles = vec![];
        thread_handles.push(Some(scope.spawn(move |_| {
            session_dispatcher(reader_tx, dispatcher_barrier, cfg);
        })));
        thread_handles.push(Some(scope.spawn(move |_| {
            reader(reader_rx, crypto_worker_tx, reader_barrier, is_provider);
        })));
    }) {
        warn!("wire worker failed to spawn thread(s)");
        return
    }
}

fn on_retrieve_message(cmd: &Command) {

}

fn on_get_consensus(cmd: &Command) {

}


#[cfg(test)]
mod tests {
    extern crate rand;
    extern crate ecdh_wrapper;
    extern crate mix_link;

    use std::thread;
    use std::time::Duration;
    use std::collections::HashMap;
    use std::net::{TcpListener, TcpStream};
    use std::thread as std_thread;
    use self::rand::os::OsRng;
    use ecdh_wrapper::{PrivateKey, PublicKey};
    use mix_link::messages::{SessionConfig, PeerAuthenticator, ServerAuthenticatorState};

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
        let static_auth_builder = StaticAuthenticatorBuilder {
            auth: peer_auth.clone(),
        };
        let auth_builder = PeerAuthenticatorBuilder::Static(static_auth_builder);

        let (tcp_fount_tx, tcp_fount_rx) = unbounded();
        let (crypto_worker_tx, crypto_worker_rx) = unbounded();
        let cfg = WireConfig {
            link_private_key: mix_priv_key,
            tcp_fount_rx: tcp_fount_rx,
            crypto_worker_tx: crypto_worker_tx,
            peer_auth_builder: auth_builder,
            is_provider: true,
        };
        start_wire_worker(cfg);

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
