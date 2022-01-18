use std::alloc::{GlobalAlloc, Layout};
use std::thread::current;
use super::{MutualExclusion, round_address};
use core::ptr;
use core::mem;


//all we need to store in the metadata of each region is the size and the location
//of the next memory block
pub struct Metadata{
    next: Option<&'static mut Metadata>,
    size: usize,
}

impl Metadata {
    const fn new(size: usize) -> Self {
        Self {next: None, size}
    }
    fn space_start(&self) -> usize {
        self as *const Self as usize
    }
    fn space_end(&self) -> usize {
        self.space_start() + self.size
    }
}

pub struct LinkAllocator {
    h: Metadata
}

impl LinkAllocator {
    pub const fn new() -> Self {
        Self {
            h: Metadata::new(0)
        }
    }

    fn search(&mut self, size: usize, rounded: usize) -> Option<(&'static mut Metadata, usize)> {
        let mut front = &mut self.h;
        while let Some(ref mut region) = front.next {
            if let Ok(space_start) = Self::alloc_to(&region, size, rounded) {
                let next = region.next.take();
                let ret = Some((front.next.take().unwrap(), space_start));
                front.next = next;
                return ret;
            } else {
                front = front.next.as_mut().unwrap();
            }
        }
        None
    }

    fn alloc_to(region: &Metadata, size: usize, rounded: usize) -> Result<usize,()> {
        let space_start = round_address(region.space_start(), rounded);
        let space_end = space_start.checked_add(size).ok_or(())?;
        if region.space_end() < space_end {
            return Err(());
        }

        let residual = region.space_end() - space_end;
        if residual > 0 && residual < mem::size_of::<Metadata>() {
            return Err(());
        }

        Ok(space_start)
    }

    pub unsafe fn initialize(&mut self, space_start: usize, size: usize) {
        self.addreg(space_start,size);
    }
    
    unsafe fn addreg(&mut self, space_start: usize, size: usize) {
        //check so we don't overflow the region with the metadata
        assert_eq!(round_address(space_start, mem::align_of::<Metadata>()), space_start);
        assert!(size >= mem::align_of::<Metadata>());
        let mut entry = Metadata::new(size);
        entry.next = self.h.next.take();
        let address = space_start as *mut Metadata;
        address.write(entry);
        //the new region becomes the new next in the allocator
        self.h.next = Some(&mut *address);
    }

    fn size_align(layout: Layout) -> (usize, usize) {
        let layout = layout
            .align_to(mem::align_of::<Metadata>())
            .expect("adjusting alignment failed")
            .pad_to_align();
        let size = layout.size().max(mem::size_of::<Metadata>());
        (size, layout.align())
    }

}

//implementation of global allocator for this linked allocator found on https://os.phil-opp.com/allocator-designs/#linked-list-allocator
unsafe impl GlobalAlloc for MutualExclusion<LinkAllocator> {
    unsafe fn alloc(&self, layout: std::alloc::Layout) -> *mut u8 {
        let (size, rounded) = LinkAllocator::size_align(layout);
        let mut allocator = self.lock();

        if let Some((region, alloc_start)) = allocator.search(size, rounded) {
            let alloc_end = alloc_start.checked_add(size).expect("overflow");
            let excess_size = region.space_end() - alloc_end;
            if excess_size > 0 {
                allocator.addreg(alloc_end, excess_size);
            }
            alloc_start as *mut u8
        } else {
            ptr::null_mut()
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: std::alloc::Layout) {
        todo!()
    }
}
