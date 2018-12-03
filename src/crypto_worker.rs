// crypto_worker.rs - Mix crypto worker.
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


extern crate crossbeam_channel;
extern crate epoch;
extern crate ecdh_wrapper;
extern crate sphinx_replay_cache;
extern crate sphinxcrypto;

use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use std::collections::HashMap;

use epoch::Clock;
use crossbeam_channel::{Receiver, Select};
use sphinx_replay_cache::{MixKeys, MixKey, Tag};
use sphinxcrypto::server::sphinx_packet_unwrap;

use super::packet::Packet;
use super::errors::UnwrapPacketError;
use super::constants;


pub struct CryptoWorkerConfig {
    pub crypto_worker_rx: Receiver<Packet>,
    pub update_rx: Receiver<bool>,
    pub halt_rx: Receiver<bool>,
    pub slack_time: u64,
    pub clock: Clock,
    pub mix_keys: MixKeys,
    pub is_provider: bool,
}

pub fn start_crypto_worker(cfg: CryptoWorkerConfig) {
    thread::spawn(move || {
        crypto_worker(cfg)
    });
}

fn unwrap_packet(packet: &mut Packet, clock: &Clock, shadow_mix_keys: &mut HashMap<u64, MixKey>) -> Result<(),UnwrapPacketError>{
    // Figure out the candidate mix private keys for this packet.
    let time = clock.now();
    let mut epochs: Vec<u64> = vec![];

    if !shadow_mix_keys.contains_key(&time.epoch) {
        return Err(UnwrapPacketError::NoKey);
    }
    epochs.push(time.epoch);
    if time.elapsed < constants::GRACE_PERIOD {
        epochs.push(time.epoch - 1);
    } else if time.till < constants::GRACE_PERIOD {
        epochs.push(time.epoch + 1);
    }

    for epoch in epochs.iter_mut() {
        let mut key = match shadow_mix_keys.get_mut(epoch) {
            Some(x) => x,
            None => {
                continue
            },
        };
        let (final_payload, replay_tag, cmds, err) = sphinx_packet_unwrap(key.private_key(), &mut packet.raw);
        if err.is_some() {
            continue
        }

        if let Some(tag) = replay_tag {
            match key.is_replay(Tag::new(tag)) {
                Ok(is_replay) => {
                    if is_replay {
                        warn!("packet replay detected");
                        return Err(UnwrapPacketError::Replay)
                    }
                },
                Err(e) => {
                    warn!("replay cache errpr: {}", e);
                    return Err(UnwrapPacketError::CacheFail)
                },
            }
        }

        packet.set_payload(final_payload);
        if let Some(commands) = cmds {
            packet.set_commands(commands);
        }
    }
    Ok(())
}

fn crypto_worker(cfg: CryptoWorkerConfig) {
    let mut shadow_mix_keys: HashMap<u64, MixKey> = HashMap::new();
    let clock = &cfg.clock;
    let mut sel = Select::new();
    let oper1 = sel.recv(&cfg.crypto_worker_rx);
    let oper2 = sel.recv(&cfg.update_rx);
    let oper3 = sel.recv(&cfg.halt_rx);
    loop {
        let mut packet = Packet::default();
        let oper = sel.select();
        match oper.index() {
            i if i == oper1 => {
                packet = match oper.recv(&cfg.crypto_worker_rx) {
                    Ok(x) => x,
                    Err(e) => {
                        warn!("crypto worker failed to receive packet: {}", e);
                        return
                    },
                };
            },
            i if i == oper2 => {
                if let Err(e) = oper.recv(&cfg.update_rx) {
                    warn!("failed to receive on update chan: {}", e);
                    return
                }
                let mut mix_keys = cfg.mix_keys.clone();
                mix_keys.shadow(&mut shadow_mix_keys);
                continue
            },
            i if i == oper3 => {
                oper.recv(&cfg.halt_rx).unwrap();
                return
            },
            _ => unreachable!(),
        }

        // Drop the packet if it has been sitting in the queue waiting to
	// be decrypted for way too long.
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(x) => x,
            Err(e) => {
                warn!("crypto worker failed to read time from clock: {}", e);
                return
            },
        };
        let dwell_time = now - Duration::from_millis(packet.receive_time);
        if dwell_time > Duration::from_millis(cfg.slack_time) {
            debug!("dropping packet, dwelled too long.");
            continue
        } else {
            debug!("crypto worker packet queue delay {:?}", dwell_time);
        }

	// Attempt to unwrap the packet.
        if let Err(e) = unwrap_packet(&mut packet, clock, &mut shadow_mix_keys) {
            warn!("failed to unwrap packet: {}", e);
            continue
        }

        // Route the packet.
        if packet.is_forward() {
            if packet.must_terminate {
                debug!("Dropping packet: (Provider received forward packet from mix)");
                continue
            }

	    // Check and adjust the delay for queue dwell time.
            // XXX drop packet if delay is too big
            // continue
            // XXX
            let packet_delay = Duration::from_millis(packet.delay.clone().unwrap().delay as u64);
            if packet_delay > dwell_time {

            // XXX } else if packet_delay == 0 {

            } else {

            }
        } else if !cfg.is_provider {
	    // This may be a decoy traffic response.
            if packet.is_surb_reply() {
                debug!("Handing off decoy response packet");
                // XXX decoy_fsm.on_packet(packet)...
                continue
            }
            debug!("Dropping invalid mix packet.");
            continue
        }

        // This node is a provider and the packet is not destined for another
	// node.  Both of the operations here end up hitting up disk among
	// other things, so are just shunted off to a separate worker so that
	// packet processing does not get blocked.
        if packet.must_forward {
            debug!("Dropping client packet");
            continue
        }

        if packet.is_to_user() || packet.is_unreliable_to_user() || packet.is_surb_reply() {
            // XXX Provider processing of packet here.
        } else {
            debug!("Dropping invalid user packet.");
        }
    }
}
