#![cfg_attr(not(target_os = "windows"), allow(dead_code))]

pub mod app_args;
pub mod diagnostics;

#[cfg(target_os = "windows")]
pub mod windows;
