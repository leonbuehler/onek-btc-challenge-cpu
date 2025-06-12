mod logger;
use bitcoin_hashes::{HashEngine, hash160};
use logger::Logger;

mod random_bnum;
use random_bnum::generate_random_start_checked;

use bnum::BUint;
use secp256k1::{PublicKey, Secp256k1, SecretKey};

// pub const CHALLENGE: u32 = 71;
// pub const TARGET: &str = "1PWo3JeB9jrGwfHDNpdGK54CRas7fsVzXU";
// pub const TARGET_PKH: &str = "f6f5431d25bbf7b12e8add9af5e3475c44a0a5b8";

pub const CHALLENGE: u32 = 24;
pub const TARGET: &str = "1rSnXMr63jdCuegJFuidJqWxUPV7AtUf7";
pub const TARGET_PKH: &str = "0959e80121f36aea13b3bad361c15dac26189e2f";

fn main() {
    simple_logger::init().unwrap();
    let logger = Logger::new();

    let target = &hex::decode(TARGET_PKH).unwrap()[..];

    // generate a random key to start searching at
    let mut search_key: BUint<4> = generate_random_start_checked();
    log::info!("starting at {search_key:x}",);

    let secp = Secp256k1::new();
    loop {
        logger.increase();
        search_key += BUint::ONE;

        let privkey = SecretKey::from_slice(&search_key.to_be_bytes()).unwrap();
        let pubkey = PublicKey::from_secret_key(&secp, &privkey);
        let pubkey_hash = hash(&pubkey.serialize()).to_byte_array();

        if pubkey_hash == target {
            log::error!("FOUND OTHER");
            log::error!("Seed HEX: {:x}", search_key);
            log::error!("WIF key: {}", fmt_wif(&privkey));
            return;
        }
    }
}

fn hash(data: &[u8]) -> hash160::Hash {
    let mut engine = hash160::Hash::engine();
    engine.input(data);
    return hash160::Hash::from_engine(engine);
}

pub fn fmt_wif(key: &SecretKey) -> String {
    let mut ret = [0; 34];
    ret[0] = 128;

    ret[1..33].copy_from_slice(&key[..]);
    ret[33] = 1;
    let privkey = base58ck::encode_check(&ret[..]);
    return privkey;
}
