// packet.rs - Packet struct.
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

extern crate sphinxcrypto;

use std::time::{SystemTime, UNIX_EPOCH};
use std::default::Default;
use sphinxcrypto::constants::PACKET_SIZE;
use sphinxcrypto::commands::{RoutingCommand, NextHop, Recipient, SURBReply, Delay};
use super::errors::PacketError;


pub struct Packet {
    pub id: u64,
    pub raw: Box<[u8; PACKET_SIZE]>,
    pub receive_time: u64,
    pub payload: Option<Vec<u8>>,
    pub next_hop: Option<NextHop>,
    pub recipient: Option<Recipient>,
    pub surb_reply: Option<SURBReply>,
    pub delay: Option<Delay>,
    pub must_forward: bool,
    pub must_terminate: bool,
}

impl Default for Packet {
    fn default() -> Packet {
        Packet {
            id: 0,
            raw: Box::new([0u8; PACKET_SIZE]),
            receive_time: 0,
            payload: None,
            next_hop: None,
            recipient: None,
            surb_reply: None,
            delay: None,
            must_forward: false,
            must_terminate: false,
        }
    }
}

impl Packet {
    pub fn new(raw: &Vec<u8>) -> Result<Self, PacketError> {
        if raw.len() != PACKET_SIZE {
            return Err(PacketError::WrongSize)
        }
        let mut payload = Box::new([0u8; PACKET_SIZE]);
        payload[..].clone_from_slice(&raw);
        let now = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(x) => x,
            Err(_) => {
                panic!("clock went back in time");
            },
        };
        let in_ms = now.as_secs() * 1000 +
            now.subsec_nanos() as u64 / 1_000_000;
        Ok(Packet{
            id: 0, // XXX - FIX ME?
            raw: payload,
            receive_time: in_ms,
            payload: None,
            next_hop: None,
            recipient: None,
            surb_reply: None,
            delay: None,
            must_forward: false,
            must_terminate: false,
        })
    }

    pub fn set_payload(&mut self, payload: Option<Vec<u8>>) {
        self.payload = payload;
    }

    pub fn set_commands(&mut self, cmds: Vec<RoutingCommand>) {
        for cmd in cmds.iter() {
            match cmd {
                RoutingCommand::NextHop(next_hop) => {
                    self.next_hop = Some(next_hop.clone());
                },
                RoutingCommand::Recipient(recipient) => {
                    self.recipient = Some(recipient.clone());
                },
                RoutingCommand::SURBReply(surb_reply) => {
                    self.surb_reply = Some(surb_reply.clone());
                },
                RoutingCommand::Delay(delay) => {
                    self.delay = Some(delay.clone());
                },
            }
        }
    }

    pub fn is_forward(&self) -> bool {
        self.next_hop.is_none() && self.delay.is_some() &&
            self.recipient.is_some() && self.surb_reply.is_none()
    }

    pub fn is_to_user(&self) -> bool {
        self.next_hop.is_none() && self.delay.is_some() &&
            self.recipient.is_some() && self.surb_reply.is_none()
    }

    pub fn is_unreliable_to_user(&self) -> bool {
        self.next_hop.is_none() && self.delay.is_none() && self.recipient.is_some() &&
            self.surb_reply.is_none()
    }

    pub fn is_surb_reply(&self) -> bool {
        self.next_hop.is_none() && self.delay.is_none() &&
            self.recipient.is_some() && self.surb_reply.is_some()
    }
}
