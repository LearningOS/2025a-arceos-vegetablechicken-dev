#![no_std]

use core::ptr::NonNull;
use allocator::{BaseAllocator, ByteAllocator, PageAllocator, AllocError, };

/// Early memory allocator
/// Use it before formal bytes-allocator and pages-allocator can work!
/// This is a double-end memory range:
/// - Alloc bytes forward
/// - Alloc pages backward
///
/// [ bytes-used | avail-area | pages-used ]
/// |            | -->    <-- |            |
/// start       b_pos        p_pos       end
///
/// For bytes area, 'count' records number of allocations.
/// When it goes down to ZERO, free bytes-used area.
/// For pages area, it will never be freed!
///
pub struct EarlyAllocator<const SIZE: usize> {
    /// start pos
    start: usize,
    /// total memory for allocation
    total: usize,
    /// The memory allocated for bytes
    used_bytes: usize,
    /// The count for bytes deallocation
    count: isize,
    /// The memory allocated for pages
    used_pages: usize,
}

impl<const SIZE: usize> EarlyAllocator<SIZE> {
    pub const fn new() -> Self {
        Self {
            total: 0,
            used_pages: 0,
            used_bytes: 0,
            start: 0,
            count: 0,
        }
    }
}
fn align_up(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}
#[allow(dead_code)]
fn align_down(pos: usize, align: usize) -> usize {
    (pos + align - 1) & !(align - 1)
}
impl<const SIZE: usize> BaseAllocator for EarlyAllocator<SIZE> {
    fn init(&mut self, start: usize, size: usize) {
        self.start = start;
        self.total = size;
    }
    /// Unsupported
    fn add_memory(&mut self, _start: usize, _size: usize) -> allocator::AllocResult {
        Err(AllocError::NoMemory)
    }
}

impl<const SIZE: usize> ByteAllocator for EarlyAllocator<SIZE> {
    fn alloc(
        &mut self,
        layout: core::alloc::Layout,
    ) -> allocator::AllocResult<NonNull<u8>> {
        let cur = align_up(self.start + self.used_bytes, layout.align());
        let size = layout.size();
        let end = cur.checked_add(size).ok_or(AllocError::MemoryOverlap)?;
        if end > self.start + self.total - self.used_pages {
            return Err(AllocError::NoMemory);
        }
        self.used_bytes = end - self.start;
        self.count += 1;
        unsafe {
            Ok(NonNull::new_unchecked(cur as *mut u8))
        }
    }

    fn dealloc(&mut self, pos: NonNull<u8>, layout: core::alloc::Layout) {
        let pos_addr = pos.as_ptr() as usize;
        if pos_addr < self.start || pos_addr > self.start + self.used_bytes {
            return;
        }
        self.count -= 1;
        if self.count <= 0 {
            unsafe {
                core::ptr::write_bytes(self.start as *mut u8, 0, self.used_bytes);
            }
            self.used_bytes = 0;
            self.count = 0;
        }
    }

    fn total_bytes(&self) -> usize {
        self.total
    }

    fn used_bytes(&self) -> usize {
        self.used_bytes
    }

    fn available_bytes(&self) -> usize {
        self.total_bytes() - self.used_bytes() - self.used_pages
    }
}

impl<const SIZE: usize> PageAllocator for EarlyAllocator<SIZE> {
    const PAGE_SIZE: usize = SIZE;

    fn alloc_pages(
        &mut self,
        num_pages: usize,
        align_pow2: usize,
    ) -> allocator::AllocResult<usize> {
        if num_pages == 0 {
            return Err(AllocError::InvalidParam);
        }
        let need_size = num_pages.checked_mul(Self::PAGE_SIZE)
            .ok_or(AllocError::MemoryOverlap)?;
        if need_size > self.available_bytes() {
            return Err(AllocError::NoMemory);
        }
        if !align_pow2.is_power_of_two() {
            return Err(AllocError::InvalidParam);
        }
        // align no less than page_size
        if align_pow2 < Self::PAGE_SIZE {
            return Err(AllocError::InvalidParam);
        }
        let cur = align_down(self.start + self.total - self.used_pages, align_pow2);
        let end = cur.checked_sub(need_size).ok_or(AllocError::MemoryOverlap)?;
        let end = align_down(end, align_pow2);
        if end < self.used_bytes + self.start {
            return Err(AllocError::NoMemory);
        }
        Ok(end)
    }

    fn dealloc_pages(&mut self, _pos: usize, _num_pages: usize) {
        // Nothing to do
    }

    fn total_pages(&self) -> usize {
        self.total / Self::PAGE_SIZE
    }

    fn used_pages(&self) -> usize {
        self.used_pages / Self::PAGE_SIZE
    }

    fn available_pages(&self) -> usize {
        self.available_bytes() / Self::PAGE_SIZE
    }
}