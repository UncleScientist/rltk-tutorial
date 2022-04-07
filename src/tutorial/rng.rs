use lazy_static::lazy_static;
use rltk::prelude::*;
use std::sync::Mutex;

lazy_static! {
    static ref RNG: Mutex<RandomNumberGenerator> = Mutex::new(RandomNumberGenerator::new());
}

// Thread A: Holds the mutex lock for a random number
// Thread B: calls lock() to get the RNG
// Thread C: calls lock() to get the RNG
// --
// Thread A: releases the lock
//
// Who wins between B & C? B can win the lock, or C can win the lock - depends
// on the implementation details of the mutex

pub fn reseed(seed: u64) {
    *RNG.lock().unwrap() = RandomNumberGenerator::seeded(seed);
}

pub fn roll_dice(n: i32, die_type: i32) -> i32 {
    RNG.lock().unwrap().roll_dice(n, die_type)
}

pub fn range(min: i32, max: i32) -> i32 {
    RNG.lock().unwrap().range(min, max)
}

pub fn random_slice_index<T>(slice: &[T]) -> Option<usize> {
    RNG.lock().unwrap().random_slice_index(slice)
}
