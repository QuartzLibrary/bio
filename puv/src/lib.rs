#![cfg_attr(target_arch = "wasm32", allow(unused_imports))]

#[cfg(not(target_arch = "wasm32"))]
mod metadata;

pub mod function;
#[cfg(not(target_arch = "wasm32"))]
pub mod map;
pub mod script;
