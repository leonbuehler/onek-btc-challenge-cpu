use std::hash::Hash;

use bitcoin::{Address, Network, PrivateKey, PublicKey, base58, hashes::hash160, key::Secp256k1};
use bnum::BUint;
use rand::Rng;

pub const CHALLENGE: u32 = 71;
pub const TARGET: &str = "1PWo3JeB9jrGwfHDNpdGK54CRas7fsVzXU";

fn main() {
    simple_logger::init().unwrap();
    log::info!("cracking challenge {CHALLENGE}");

    let mut lower_bound: BUint<4> = BUint::ZERO;
    lower_bound.set_bit(CHALLENGE - 1, true);
    let mut upper_bound: BUint<4> = BUint::ZERO;
    upper_bound.set_bit(CHALLENGE, true);

    log::info!("range: {:x}:{:x}", lower_bound, upper_bound);

    let mut search_key: BUint<4> = generate_random_start();
    log::info!("starting at {search_key:x}",);

    let secp = Secp256k1::new();
    loop {
        let privkey = PrivateKey::from_slice(&search_key.to_be_bytes(), Network::Bitcoin).unwrap();
        let pubkey = privkey.public_key(&secp);
        let address = format_pubkey(&pubkey);

        log::info!("trying: {search_key:?}");
        log::info!("addr: {address}");
        search_key += BUint::ONE;

        if address == TARGET {
            log::error!("FOUND");
            log::error!("Seed HEX: {:x}", search_key);
            log::error!("Seed num: {}", search_key);
            log::error!("WIF key: {}", privkey.to_wif());
            log::error!("pr key: {}", privkey.to_string());

            return;
        }
    }
}

fn format_pubkey(pubkey: &PublicKey) -> String {
    let hash = pubkey.pubkey_hash();
    let mut prefixed = [0; 21];
    prefixed[0] = 0;
    prefixed[1..].copy_from_slice(&hash[..]);
    return base58::encode_check(&prefixed[..]);
}

fn generate_random_start() -> BUint<4> {
    let mut lower_bound: BUint<4> = BUint::ZERO;
    lower_bound.set_bit(CHALLENGE - 1, true);
    let mut upper_bound: BUint<4> = BUint::ZERO;
    upper_bound.set_bit(CHALLENGE, true);

    let mut new = BUint::ZERO;
    new.set_bit(CHALLENGE - 1, true);
    let mut rng = rand::thread_rng();
    for i in 0..CHALLENGE {
        if rng.gen_bool(0.5) {
            new.set_bit(i, true);
        }
    }
    if new < lower_bound {
        log::error!("some error happened");
        panic!()
    }
    if new > upper_bound {
        log::error!("some other error happened");
        panic!()
    }
    return new;
}
