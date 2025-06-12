mod logger;
use logger::Logger;

mod random_bnum;
use random_bnum::generate_random_start_checked;

use bitcoin::{
    Network, PrivateKey, PublicKey, base58,
    hashes::{Hash, HashEngine, hash160},
    key::Secp256k1,
};
use bnum::BUint;

pub const CHALLENGE: u32 = 71;
pub const TARGET: &str = "1PWo3JeB9jrGwfHDNpdGK54CRas7fsVzXU";

fn main() {
    simple_logger::init().unwrap();
    let logger = Logger::new();

    // generate a random key to start searching at
    let mut search_key: BUint<4> = generate_random_start_checked();
    log::info!("starting at {search_key:x}",);

    let secp = Secp256k1::new();
    loop {
        logger.increase();
        let privkey = PrivateKey::from_slice(&search_key.to_be_bytes(), Network::Bitcoin).unwrap();
        let pubkey = privkey.public_key(&secp);
        let address = format_pubkey(&pubkey);

        search_key += BUint::ONE;

        if address == TARGET {
            log::error!("FOUND");
            log::error!("Seed HEX: {:x}", search_key);
            log::error!("WIF key: {}", privkey.to_wif());
            log::error!("pr key: {}", privkey.to_string());

            return;
        }
    }
}

fn hash(data: &[u8]) -> hash160::Hash {
    let mut engine = hash160::Hash::engine();
    engine.input(data);
    return hash160::Hash::from_engine(engine);
}

fn format_pubkey(pubkey: &PublicKey) -> String {
    let serialized = pubkey.inner.serialize();
    let hash1 = hash(&serialized);
    let hash = pubkey.pubkey_hash();
    assert_eq!(hash1, hash.into());
    let mut prefixed = [0; 21];
    prefixed[0] = 0;
    prefixed[1..].copy_from_slice(&hash[..]);
    return base58::encode_check(&prefixed[..]);
}
