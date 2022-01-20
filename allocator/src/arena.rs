use std::alloc::GlobalAlloc;
use super::{MutualExclusion, round_address};
use core::ptr;
pub struct ArenaAllocator {
    space_start: usize,
    space_end: usize,
    next: usize,
    allocations: usize,
}

impl ArenaAllocator {
    pub const fn new() -> ArenaAllocator {
        let space_start = 0;
        let space_end = 0;
        let next = 0;
        let allocations = 0;
        ArenaAllocator{space_start,space_end,next,allocations}
    }
    pub unsafe fn initialize(&mut self, space_start: usize, space_end: usize) {
        self.space_start = space_start;
        self.space_end = space_end;
        self.next = space_start;
    }

}

unsafe impl GlobalAlloc for MutualExclusion<ArenaAllocator> {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let mut Arena = self.lock();

        let space_start = round_address(Arena.next, layout.align());
        let space_end = match space_start.checked_add(layout.size()) {
            Some(end) => end,
            None => return ptr::null_mut(),
        };

        if space_end > Arena.space_end {
            return ptr::null_mut()
        } else {
            Arena.next = space_end;
            Arena.allocations += 1;
            space_start as *mut u8
        }

    }
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: std::alloc::Layout) {
        let mut Arena = self.lock();

        Arena.allocations -= 1;
        if Arena.allocations == 0 {
            Arena.next = Arena.space_start
        }
    }
}

