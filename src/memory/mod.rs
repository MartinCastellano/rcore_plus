pub use arch::paging::*;
use bit_allocator::{BitAlloc, BitAlloc64K};
use self::stack_allocator::*;
use spin::{Mutex, MutexGuard};
use super::HEAP_ALLOCATOR;
use ucore_memory::{*, cow::CowExt, paging::PageTable};
pub use ucore_memory::memory_set::{MemoryArea, MemoryAttr, MemorySet as MemorySet_, Stack};

pub type MemorySet = MemorySet_<InactivePageTable0>;

mod stack_allocator;

lazy_static! {
    pub static ref FRAME_ALLOCATOR: Mutex<BitAlloc64K> = Mutex::new(BitAlloc64K::default());
}
pub static STACK_ALLOCATOR: Mutex<Option<StackAllocator>> = Mutex::new(None);

pub fn alloc_frame() -> Option<usize> {
    FRAME_ALLOCATOR.lock().alloc().map(|id| id * PAGE_SIZE)
}

pub fn dealloc_frame(target: usize) {
    FRAME_ALLOCATOR.lock().dealloc(target / PAGE_SIZE);
}

pub fn alloc_stack(size_in_pages: usize) -> Stack {
    STACK_ALLOCATOR.lock()
        .as_mut().expect("stack allocator is not initialized")
        .alloc_stack(size_in_pages).expect("no more stack")
}

lazy_static! {
    static ref ACTIVE_TABLE: Mutex<CowExt<ActivePageTable>> = Mutex::new(unsafe {
        CowExt::new(ActivePageTable::new())
    });
}

/// The only way to get active page table
pub fn active_table() -> MutexGuard<'static, CowExt<ActivePageTable>> {
    ACTIVE_TABLE.lock()
}

// Return true to continue, false to halt
pub fn page_fault_handler(addr: usize) -> bool {
    // Handle copy on write
    unsafe { ACTIVE_TABLE.force_unlock(); }
    active_table().page_fault_handler(addr, || alloc_frame().unwrap())
}

pub fn init_heap() {
    use consts::{KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE, KERNEL_STACK_OFFSET, KERNEL_STACK_SIZE};

    unsafe { HEAP_ALLOCATOR.lock().init(KERNEL_HEAP_OFFSET, KERNEL_HEAP_SIZE); }

    *STACK_ALLOCATOR.lock() = Some({
        use ucore_memory::Page;
        StackAllocator::new(Page::range_of(KERNEL_STACK_OFFSET, KERNEL_STACK_OFFSET + KERNEL_STACK_SIZE))
    });
}

//pub mod test {
//    pub fn cow() {
//        use super::*;
//        use ucore_memory::cow::test::test_with;
//        test_with(&mut active_table());
//    }
//}