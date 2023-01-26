use lazy_static::lazy_static;
use nanorand::{Rng, WyRand};
use std::sync::Mutex;

lazy_static! {
    static ref RNG: Mutex<WyRand> = Mutex::new(WyRand::new());
}

#[no_coverage]
pub fn percent() -> u8 {
    let mut rng = RNG.lock().expect("could not lock RNG");
    rng.generate_range(1..101)
}
