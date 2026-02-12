#![cfg_attr(not(feature = "std"), no_std)]

pub mod rate_meter;
pub mod utils;
pub mod dencoder;

#[cfg(feature = "std")]
pub mod autotimer;
#[cfg(feature = "std")]
pub mod autodrop_thread;
#[cfg(feature = "std")]
pub mod unbounded_broadcast;