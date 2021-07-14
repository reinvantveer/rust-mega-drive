#![no_std]
#![feature(allocator_api)]
#![feature(const_mut_refs)]
#![feature(alloc_error_handler)]
#![feature(default_alloc_error_handler)]

pub mod heap;
mod hole;

#[alloc_error_handler]
fn alloc_error_handler(layout: core::alloc::Layout) -> ! {
    panic!("allocation error: {:?}", layout)
}

