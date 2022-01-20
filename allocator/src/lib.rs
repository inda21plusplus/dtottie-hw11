#![feature(const_mut_refs)]
#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

pub mod arena;
pub mod linked;
pub mod buddy;

pub struct MutualExclusion<A> {
    contobj: spin::Mutex<A>,
}

impl<A> MutualExclusion<A> {
    pub const fn new(lock: A) -> Self {
        MutualExclusion {
            contobj: spin::Mutex::new(lock)
        }
    }
    
    pub fn lock(&self) -> spin::MutexGuard<A> {
        self.contobj.lock()
    }
}

fn round_address(first: usize, second: usize) -> usize {
    let cmp = first % second;
    if cmp == 0 {
        cmp
    } else {
        first - cmp + second
    }
}

