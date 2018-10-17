#[cfg(test)]
mod tests {
    extern crate ecdh_wrapper;
    extern crate clear_on_drop;
    extern crate rand;
    extern crate rustc_serialize;

    use self::ecdh_wrapper::PrivateKey;
    use self::clear_on_drop::ClearOnDrop;
    use self::rand::Rng;
    use self::rand::os::OsRng;
    use self::rustc_serialize::hex::ToHex;

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
}
