// utils/time.rs
//! Utilities related to time

#[inline]
pub fn timestamp() -> String { chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string() }
