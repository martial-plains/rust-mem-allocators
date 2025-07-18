extern crate test;

use std::{
    alloc::{Allocator, Layout},
    any::Any,
};

use mem_allocs::ArenaAllocator;
use rand::{Rng, SeedableRng, rngs::StdRng};
use test::Bencher;

pub const A: usize = 1e9 as usize;

pub const ALLOCATION_SIZES: &[usize] = &[32, 64, 256, 512, 1024, 2048, 4096];
pub const ALIGNMENTS: &[usize] = &[8, 8, 8, 8, 8, 8, 8];

pub struct Benchmark<'b> {
    bench: &'b mut Bencher,
}

impl<'b> Benchmark<'b> {
    pub const fn new(bench: &'b mut Bencher) -> Self {
        Self { bench }
    }

    pub fn multi_alloc<A: Allocator + 'static>(
        &mut self,
        alloc: &A,
        sizes: &[usize],
        aligns: &[usize],
    ) {
        assert_eq!(sizes.len(), aligns.len());

        for i in 0..sizes.len() {
            self.single_alloc(alloc, sizes[i], aligns[i]);
        }
    }

    pub fn multi_free<A: Allocator>(&mut self, alloc: &A, sizes: &[usize], aligns: &[usize]) {
        assert_eq!(sizes.len(), aligns.len());

        for i in 0..sizes.len() {
            self.single_free(alloc, sizes[i], aligns[i]);
        }
    }

    pub fn random_alloc<A: Allocator + 'static>(
        &mut self,
        alloc: &A,
        sizes: &[usize],
        aligns: &[usize],
    ) {
        assert_eq!(sizes.len(), aligns.len());

        let mut rng = StdRng::seed_from_u64(1);
        let mut ptrs = Vec::with_capacity(10_000);

        self.bench.iter(|| {
            let idx = rng.random_range(0..sizes.len());
            let size = sizes[idx];
            let align = aligns[idx];
            let layout = Layout::from_size_align(size, align).unwrap();
            let ptr = alloc.allocate(layout).unwrap();
            test::black_box(&ptr);
            ptrs.push((ptr, layout));

            if let Some(alloc) = (alloc as &dyn Any).downcast_ref::<ArenaAllocator>() {
                alloc.reset();
            }
        });

        for (ptr, layout) in ptrs {
            unsafe { alloc.deallocate(ptr.as_non_null_ptr(), layout) };
        }
    }

    pub fn random_free<A: Allocator + 'static>(
        &mut self,
        alloc: &A,
        sizes: &[usize],
        aligns: &[usize],
    ) {
        assert_eq!(sizes.len(), aligns.len());

        let mut rng = StdRng::seed_from_u64(1);
        let op_count = 10_000;
        let mut ptrs = Vec::with_capacity(op_count);

        for _ in 0..op_count {
            let idx = rng.random_range(0..sizes.len());
            let size = sizes[idx];
            let align = aligns[idx];
            let layout = Layout::from_size_align(size, align).unwrap();
            let ptr = alloc.allocate(layout).unwrap();
            ptrs.push((ptr, layout));
        }

        self.bench.iter(|| {
            for (ptr, layout) in &ptrs {
                unsafe { alloc.deallocate(ptr.as_non_null_ptr(), *layout) };
            }

            for entry in &mut ptrs {
                let idx = rng.random_range(0..sizes.len());
                let size = sizes[idx];
                let align = aligns[idx];
                let layout = Layout::from_size_align(size, align).unwrap();
                let new_ptr = alloc.allocate(layout).unwrap();
                *entry = (new_ptr, layout);
            }

            if let Some(alloc) = (alloc as &dyn Any).downcast_ref::<ArenaAllocator>() {
                alloc.reset();
            }
        });

        for (ptr, layout) in ptrs {
            unsafe { alloc.deallocate(ptr.as_non_null_ptr(), layout) };
        }
    }

    fn single_alloc<A: Allocator + 'static>(&mut self, alloc: &A, size: usize, align: usize) {
        let layout = Layout::from_size_align(size, align).unwrap();
        let mut addresses = Vec::with_capacity(1_000);

        self.bench.iter(|| {
            let ptr = alloc.allocate(layout).unwrap();
            test::black_box(&ptr);
            addresses.push(ptr);

            if let Some(alloc) = (alloc as &dyn Any).downcast_ref::<ArenaAllocator>() {
                alloc.reset();
            }
        });

        for address in addresses {
            unsafe { alloc.deallocate(address.as_non_null_ptr(), layout) };
        }
    }

    fn single_free<A: Allocator>(&mut self, alloc: &A, size: usize, align: usize) {
        let layout = Layout::from_size_align(size, align).unwrap();

        let mut addresses = Vec::with_capacity(10_000);

        for _ in 0..10_000 {
            let ptr = alloc.allocate(layout).unwrap();
            addresses.push(ptr);
        }

        self.bench.iter(|| {
            for ptr in &addresses {
                unsafe { alloc.deallocate(ptr.as_non_null_ptr(), layout) };
            }
        });
    }
}
