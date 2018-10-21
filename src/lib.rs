// lib.rs - Crate for implementing mix servers.
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

extern crate threadpool;
extern crate ecdh_wrapper;

pub mod server;
pub mod tcp_listener;
pub mod wire_worker;


#[cfg(test)]
mod tests {
    extern crate ecdh_wrapper;
    extern crate rand;
    extern crate clear_on_drop;
    extern crate drop_guard;

    use self::ecdh_wrapper::PrivateKey;

    use self::rand::os::OsRng;
    use self::clear_on_drop::ClearOnDrop;
    use self::drop_guard::DropGuard;

    #[test]
    fn clear_ecdh_key_on_drop_test() {
        let mut keypair = PrivateKey::default();
        let zeros = vec![0u8; 32];
        {
            let mut key = ClearOnDrop::new(&mut keypair);
            let mut rng = OsRng::new().unwrap();
            assert_eq!(key.to_vec(), zeros);
            key.regenerate(&mut rng);
            assert_ne!(key.to_vec(), zeros);
        }   // key is dropped here
        assert_eq!(keypair, PrivateKey::default());
        assert_eq!(keypair.to_vec(), zeros);
    }

    #[test]
    fn guard_on_drop_test() {
        let mut keypair = PrivateKey::default();
        let zeros = vec![0u8; 32];

        {
            let mut rng = OsRng::new().unwrap();
            keypair.regenerate(&mut rng);
            assert_ne!(keypair.to_vec(), zeros);

            // The guard must have a name. _ will drop it instantly, which would lead to unexpected results
            let _g = DropGuard::new(keypair, |_|{
                keypair.reset()
            });
        }
        assert_eq!(keypair.to_vec(), zeros);
    }
}
