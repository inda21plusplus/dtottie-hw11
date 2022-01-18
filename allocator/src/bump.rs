use std::alloc::GlobalAlloc;
use super::{MutualExclusion, round_address};
use core::ptr;
pub struct BumpAllocator {
    space_start: usize,
    space_end: usize,
    next: usize,
    allocations: usize,
}

impl BumpAllocator {
    pub const fn new() -> BumpAllocator {
        let space_start = 0;
        let space_end = 0;
        let next = 0;
        let allocations = 0;
        BumpAllocator{space_start,space_end,next,allocations}
    }
    pub unsafe fn initialize(&mut self, space_start: usize, space_end: usize) {
        self.space_start = space_start;
        self.space_end = space_end;
        self.next = space_start;
    }

}

unsafe impl GlobalAlloc for MutualExclusion<BumpAllocator> {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let mut bump = self.lock();

        let space_start = round_address(bump.next, layout.align());
        let space_end = match space_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if space_end > bump.space_end {
            return ptr::null_mut()
        } else {
            bump.next = space_end;
            bump.allocations += 1;
            space_start as *mut u8
        }

    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: std::alloc::Layout) {
        let mut bump = self.lock();

        bump.allocations -= 1;
        if bump.allocations == 0 {
            bump.next = bump.space_start
        }
    }
}

