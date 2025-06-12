use super::CHALLENGE;
use bnum::BUint;
use rand::Rng;

// this checks that the random num is inside the bounds
pub fn generate_random_start_checked() -> BUint<4> {
    let mut lower_bound: BUint<4> = BUint::ZERO;
    lower_bound.set_bit(CHALLENGE - 1, true);
    let mut upper_bound: BUint<4> = BUint::ZERO;
    upper_bound.set_bit(CHALLENGE, true);

    let new = generate_random_start();

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

// for every bit in the number, we generate a random bool, that determines if that bit gets flipped
fn generate_random_start() -> BUint<4> {
    let mut new = BUint::ZERO;
    new.set_bit(CHALLENGE - 1, true);
    let mut rng = rand::thread_rng();
    for i in 0..CHALLENGE {
        if rng.gen_bool(0.5) {
            new.set_bit(i, true);
        }
    }
    return new;
}
