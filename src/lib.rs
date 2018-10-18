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
