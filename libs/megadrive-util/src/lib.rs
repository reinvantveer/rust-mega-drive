#![no_std]

// In order to use the PanicInfo message() method
#![feature(panic_info_message)]

// In order to transmute to `pub args: &'a [ArgumentV1<'a>]`
#![feature(fmt_internals)]

pub mod rng;
mod panic;
