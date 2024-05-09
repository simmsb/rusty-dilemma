const HEAP_SIZE: usize = 20 * 1024;
static mut HEAP: [u8; HEAP_SIZE] = [0; HEAP_SIZE];

#[global_allocator]
static ALLOCATOR: embedded_alloc::Heap = embedded_alloc::Heap::empty();

pub fn init() {
    unsafe {
        ALLOCATOR.init(core::ptr::addr_of!(HEAP) as *const u8 as usize, HEAP_SIZE);
    }
}
