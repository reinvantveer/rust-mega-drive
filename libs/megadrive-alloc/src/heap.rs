// This is basically a lightly refactored version of Philipp Oppermann's
// https://github.com/phil-opp/linked-list-allocator/blob/v0.3.0/src/lib.rs

use core::mem;
use core::alloc::{AllocError, GlobalAlloc, Layout};
use core::cell::RefCell;

use crate::hole::{Hole, HoleList};
use megadrive_sys::heap;

/// A fixed size heap backed by a linked list of free memory blocks.
pub struct Heap {
    bottom: RefCell<usize>,
    size: RefCell<usize>,
    holes: RefCell<HoleList>,
}

impl Heap {
    /// Creates an empty heap. All allocate calls will return `None`.
    pub const fn empty() -> Heap {
        Heap {
            bottom: RefCell::new(0),
            size: RefCell::new(0),
            holes: RefCell::new(HoleList::empty()),
        }
    }

    // Initializes an empty heap
    //
    // SAFETY:
    // This function must be called at most once and must only be used on an
    // empty heap.
    pub unsafe fn init(&self) {
        // Create the raw slice from the magical heap function from megadrive_sys
        let heap_slice = heap();

        // Take a raw pointer to the slice as the bottom
        let mut bottom = *self.bottom.borrow_mut();
        bottom = heap_slice.as_ptr() as usize;

        // An then the size as the length of the slice
        let mut size = *self.size.borrow_mut();
        size = heap_slice.len();
        *self.holes.borrow_mut() = HoleList::new(bottom as usize, size as usize);
    }

    /// Allocates a chunk of the given size with the given alignment. Returns a pointer to the
    /// beginning of that chunk if it was successful. Else it returns `None`.
    /// This function scans the list of free memory blocks and uses the first block that is big
    /// enough. The runtime is in O(n) where n is the number of free blocks, but it should be
    /// reasonably fast for small allocations.
    pub fn allocate_first_fit(&self, layout: Layout) -> Result<*mut u8, AllocError> {
        let mut size = layout.size();
        if size < HoleList::min_size() {
            size = HoleList::min_size();
        }
        let size = align_up(size, mem::align_of::<Hole>());
        let layout = Layout::from_size_align(size, layout.align()).unwrap();

        self.holes.borrow_mut().allocate_first_fit(layout)
    }

    /// Frees the given allocation. `ptr` must be a pointer returned
    /// by a call to the `allocate_first_fit` function with identical size and alignment. Undefined
    /// behavior may occur for invalid arguments, thus this function is unsafe.
    ///
    /// This function walks the list of free memory blocks and inserts the freed block at the
    /// correct place. If the freed block is adjacent to another free block, the blocks are merged
    /// again. This operation is in `O(n)` since the list needs to be sorted by address.
    pub unsafe fn deallocate(&self, ptr: *mut u8, layout: Layout) {
        let mut size = layout.size();
        if size < HoleList::min_size() {
            size = HoleList::min_size();
        }
        let size = align_up(size, mem::align_of::<Hole>());
        let layout = Layout::from_size_align(size, layout.align()).unwrap();

        self.holes.borrow_mut().deallocate(ptr, layout);
    }

    /// Returns the bottom address of the heap.
    pub fn bottom(&self) -> usize {
        *self.bottom.borrow_mut()
    }

    /// Returns the size of the heap.
    pub fn size(&self) -> usize {
        *self.size.borrow_mut()
    }

    /// Return the top address of the heap
    pub fn top(&self) -> usize {
        *self.bottom.borrow_mut() + *self.size.borrow_mut()
    }

    /// Extends the size of the heap by creating a new hole at the end
    ///
    /// SAFETY:
    ///
    /// The new extended area must be valid
    pub unsafe fn extend(&self, by: usize) {
        let top = self.top();
        let layout = Layout::from_size_align(by, 1).unwrap();
        let mut holes_mut = self.holes.borrow_mut();
        holes_mut.deallocate(top as *mut u8, layout);
        *self.size.borrow_mut() += by;
    }
}

/// SAFETY:
/// The Sync implementation block is required by the compiler in order to have a shared internally
/// mutable Heap.
///
/// This is basically a dummy implementation. The heap cannot be safely used across thread
/// boundaries as it is likely to lead to data races. So: don't use in multi-threaded apps.
unsafe impl Sync for Heap {}

unsafe impl GlobalAlloc for Heap {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        self.allocate_first_fit(layout)
            .ok()
            .map_or(0 as *mut u8, |allocation| allocation)
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.deallocate(ptr, layout)
    }
}

/// Align downwards. Returns the greatest x with alignment `align`
/// so that x <= addr. The alignment must be a power of 2.
pub fn align_down(addr: usize, align: usize) -> usize {
    if align.is_power_of_two() {
        addr & !(align - 1)
    } else if align == 0 {
        addr
    } else {
        panic!("`align` must be a power of 2");
    }
}

/// Align upwards. Returns the smallest x with alignment `align`
/// so that x >= addr. The alignment must be a power of 2.
pub fn align_up(addr: usize, align: usize) -> usize {
    align_down(addr + align - 1, align)
}
