use core::num;
use std::cmp;


struct BuddyAllocator {
    space_start: usize,
    space_end: usize,
    levels: u8,
    block_size: u16,
    free_lists: Vec<Vec<u32>>,
}

impl BuddyAllocator {
    pub const fn new(space_start: usize, space_end: usize, block_size: u16) -> Self {
        //we calculate the number of non-leaf levels based on the size of the memory space and the block size.
        let mut levels: u8 = 0;
        while ((block_size as usize) << levels as usize) < space_end - space_start {
            levels += 1;
        }
        //list of lists of free blocks in every level.
        let mut free_lists: Vec<Vec<u32>> = Vec::with_capacity(levels as usize + 1);

        //thus each block needs a level. No for loops in const fn :(
        let i = 0;
        while i <= (levels + 1) {
            free_lists.push(Vec::with_capacity(4));
            i += 1;
        }


        free_lists[0].push(0);

        Self {
            space_start, space_end, levels, block_size, free_lists
        }
    }

    fn allocate(&mut self, size: usize, rounded: usize) -> Option<usize> {
        let size = cmp::max(size, rounded);
        self.level_size(size).and_then(|required| {
            self.get_block(required).map(|block| {
                let offset = block as u64 * (self.size_limit() >> required as usize) as u64;
                return self.space_start + rounded;
            })
        })
    }

    fn deallocate(&mut self, addr: usize, size: usize, rounded: usize) {
        let size = cmp::max(size, rounded);
        if let Some(required) = self.level_size(size) {
            let level_block_size = self.size_limit() >> required;
            let block_number = ((addr - self.space_start) / level_block_size) as u32;
            self.free_lists[required].push(block_number);
            self.merge(required, block_number);
        }
    }

    fn merge(&mut self, level: usize, block_num: u32) {
        // toggle last bit to get buddy block
        let buddy_block = block_num ^ 1;
        // if buddy block in free list
        if let Some(buddy_idx) = self.free_lists[level]
            .iter()
            .position(|blk| *blk == buddy_block)
        {
            self.free_lists[level].pop();
            self.free_lists[level].remove(buddy_idx);
            self.free_lists[level - 1].push(block_num / 2);
            self.merge(level - 1, block_num / 2)
        }
    }

    //max size that this allocator can hold
    fn size_limit(&self) -> usize {
        (self.block_size as usize) << (self.levels as usize)
    }

    fn includes(&self, addr: usize) -> bool {
        addr >= self.space_start && addr <= self.space_end
    }

    fn level_size(&self, size: usize) -> Option<usize> {
        //find which level can hold the size
        let size_limit = self.size_limit();
        if size > size_limit {
           return None;
        } 

        let mut next_level = 1;
        while (size_limit >> next_level) >= size {
            next_level += 1;
        }

        let level = cmp::min(next_level - 1, self.levels as usize);
        return Some(level);

    }

    fn get_block(&mut self, level:usize) -> Option<u32> {
        self.free_lists[level].pop().or_else(|| self.split(level))
    }

    fn split(&mut self, level: usize) -> Option<u32> {
        if level == 0 {
            return None;
        }
        self.get_block(level - 1).map(|block| {
            self.free_lists[level].push(block * 2 + 1);
            block * 2
        })
    }

}