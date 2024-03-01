#![no_std]
#![no_main]

extern crate alloc;

use alloc::string::String;

pub mod apu;
pub mod audio;
pub mod bus;
pub mod cart;
#[macro_use]
pub mod common;
pub mod control_deck;
pub mod cpu;
pub mod genie;
pub mod input;
pub mod mapper;
pub mod mem;
pub mod ppu;
pub mod video;

#[derive(Debug)]
pub struct NesError {
    pub error: String,
}

impl NesError {
    pub fn new(error: String) -> NesError {
        NesError {
            error,
        }
    }
}

pub type NesResult<T> = Result<T, NesError>;

