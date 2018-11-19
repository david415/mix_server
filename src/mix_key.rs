// mix_key.rs - Mix key logistics.
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
//
///
/// This module handles Sphinx packet replay detection and storing the mix
/// key and it's associated metadata. To quoate "Sphinx Mix Network Cryptographic
/// Packet Format Specification", section "6. Sphinx Packet Processing" states
/// the following:
///
///   "After a packet has been unwrapped successfully, a replay detection
///   tag is checked to ensure that the packet has not been seen before.
///   If the packet is a replay, the packet MUST be discarded with no
///   additional processing."
///
/// Note: 1GB ethernet line speed is 118 MB/s and 123 MB/s with jumbo frames
/// therefore we can set the line rate to 128974848 = 123 * 1024 * 1024.
///
extern crate sled;
extern crate bloom;
extern crate rand;
extern crate byteorder;
extern crate sphinxcrypto;

use std::thread;
use std::thread::JoinHandle;
use std::collections::hash_map::RandomState;
use std::collections::HashSet;
use std::path::Path;

use self::byteorder::{ByteOrder, LittleEndian};

use self::rand::Rng;
use self::rand::os::OsRng;

use sled::{Tree, PinnedValue};
use bloom::{ASMS,BloomFilter};
use crossbeam_channel as channel;

use sphinxcrypto::constants::{SPHINX_REPLAY_TAG_SIZE, PACKET_SIZE};
use ecdh_wrapper::{PublicKey, PrivateKey};

use super::errors::MixKeyError;

// Flush writeback cache every 10 seconds.
const FLUSH_FREQUENCY: u64 = 10000;

const MIX_CACHE_KEY: &str = "private_key";

const EPOCH_KEY: &str = "epoch";

#[derive(PartialEq, Eq, Hash)]
pub struct Tag([u8; SPHINX_REPLAY_TAG_SIZE]);

impl Tag {
    fn to_vec(&self) -> Vec<u8> {
        self.0.to_vec()
    }
}

impl Clone for Tag {
    fn clone(&self) -> Tag {
        Tag(self.0)
    }
}

pub struct MixKey {
    filter: BloomFilter<RandomState, RandomState>,
    cache: Tree,
    private_key: PrivateKey,
    epoch: u64,
}

impl MixKey {
    pub fn new(line_rate: u64, epoch: u64, epoch_duration: u16, base_dir: String) -> Result<MixKey, MixKeyError> {
        let false_positive_rate: f32 = 0.01;
        let expected_num_items: u32 = (line_rate as f64 / PACKET_SIZE as f64) as u32 * epoch_duration as u32;
        let cache_capacity: usize = (((epoch_duration as u64 * line_rate) / PACKET_SIZE as u64) as usize * SPHINX_REPLAY_TAG_SIZE) / 2;

        let cache_cfg_builder = sled::ConfigBuilder::default()
            .path(Path::new(&base_dir).join(format!("mix_key.{}", epoch)))
            .cache_capacity(cache_capacity)
            .use_compression(false)
            .flush_every_ms(Some(FLUSH_FREQUENCY))
            .snapshot_after_ops(100_000); // XXX
        let cache_cfg = cache_cfg_builder.build();

        let cache = match Tree::start(cache_cfg) {
            Ok(x) => x,
            Err(e) => {
                print!("create cache failed: {}", e);
                return Err(MixKeyError::CreateCacheFailed);
            },
        };

        if let Ok(Some(raw_epoch)) = cache.get(EPOCH_KEY.to_string().as_bytes()) {
            let stored_epoch = LittleEndian::read_u64(&raw_epoch);
            if epoch != stored_epoch {
                warn!("mix key mismatched epoch during load.");
                return Err(MixKeyError::LoadCacheFailed);
            }
        } else {
            let mut raw_epoch = vec![0u8; 8];
            LittleEndian::write_u64(&mut raw_epoch, epoch);
            cache.set(raw_epoch, vec![]);
        }

        let mut private_key = PrivateKey::default();
        if let Ok(Some(key_blob)) = cache.get(MIX_CACHE_KEY.to_string().as_bytes()) {
            private_key.load_bytes(&key_blob)?;
        } else {
            let mut rng = OsRng::new()?;
            private_key = PrivateKey::generate(&mut rng)?;
            if let Err(e) = cache.set(MIX_CACHE_KEY.as_bytes().to_vec(), private_key.to_vec()) {
                warn!("mix key failed to write to disk cache: {}", e);
                return Err(MixKeyError::CreateCacheFailed);
            }
        }

        Ok(MixKey{
            filter: BloomFilter::with_rate(false_positive_rate, expected_num_items),
            cache: cache,
            private_key: private_key,
            epoch: epoch,
        })
    }

    pub fn private_key(&self) -> &PrivateKey {
        &self.private_key
    }

    pub fn public_key(&self) -> PublicKey {
        self.private_key.public_key()
    }

    pub fn is_replay(&mut self, tag: Tag) -> bool {
        let maybe_replay = self.filter.contains(&tag);
        if !maybe_replay {
            self.filter.insert(&tag);
            self.cache.set(tag.to_vec(), vec![]);
            return false
        }
        if let Ok(_) = self.cache.get(&tag.0) {
            return true
        } else {
            self.filter.insert(&tag);
            self.cache.set(tag.to_vec(), vec![]);
            return false
        }
    }

    pub fn flush(&mut self) {
        self.cache.flush().unwrap()
    }
}

#[cfg(test)]
mod tests {

    extern crate tempfile;
    extern crate rand;

    use std::fs::File;
    use self::rand::Rng;
    use self::rand::os::OsRng;
    use self::tempfile::TempDir;
    use super::*;


    #[test]
    fn basic_mix_key_test() {
        let cache_dir = TempDir::new().unwrap();
        {
            let cache_dir_path = cache_dir.path().clone();
            //let epoch_duration = 3 * 60 * 60; // 3 hours
            //let epoch_duration = 1 * 60 * 60; // 1 hours
            let epoch_duration = 1;
            let epoch = 1;
            let mut mix_key = MixKey::new(128974848, epoch, epoch_duration, cache_dir_path.to_str().unwrap().to_string()).unwrap();
            let mut rng = OsRng::new().unwrap();
            let mut raw = [0u8; SPHINX_REPLAY_TAG_SIZE];
            rng.fill_bytes(&mut raw);
            let tag = Tag(raw);
            assert_eq!(mix_key.is_replay(tag.clone()), false);
            assert_eq!(mix_key.is_replay(tag.clone()), true);
            assert_eq!(mix_key.is_replay(tag), true);
            mix_key.flush();
            let mut priv_key = PrivateKey::default();
            priv_key.load_bytes(&mix_key.private_key().to_vec()).unwrap();
            drop(mix_key);

            let mut new_mix_key = MixKey::new(128974848, epoch, epoch_duration, cache_dir_path.to_str().unwrap().to_string()).unwrap();
            assert_eq!(epoch, new_mix_key.epoch);
            assert_eq!(priv_key, *new_mix_key.private_key());
        }
        TempDir::close(cache_dir);
    }
}
