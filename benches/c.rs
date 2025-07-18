#![cfg(feature = "c_allocator")]
#![feature(test, allocator_api, slice_ptr_get, random)]

extern crate test;

use mem_allocs::CAllocator;

use test::Bencher;

use common::{ALIGNMENTS, ALLOCATION_SIZES, Benchmark};

mod common;

#[bench]
fn bench_c_alloc_multiple_alloc(bench: &mut Bencher) {
    let c = CAllocator;
    let mut benchmark = Benchmark::new(bench);

    benchmark.multi_alloc(&c, ALLOCATION_SIZES, ALIGNMENTS);
}

#[bench]
fn bench_c_alloc_multiple_free(bench: &mut Bencher) {
    let c = CAllocator;
    let mut benchmark = Benchmark::new(bench);

    benchmark.multi_free(&c, ALLOCATION_SIZES, ALIGNMENTS);
}

#[bench]
fn bench_c_alloc_random_alloc(bench: &mut Bencher) {
    let c = CAllocator;
    let mut benchmark = Benchmark::new(bench);

    benchmark.random_alloc(&c, ALLOCATION_SIZES, ALIGNMENTS);
}

#[bench]
fn bench_c_alloc_random_free(bench: &mut Bencher) {
    let c = CAllocator;
    let mut benchmark = Benchmark::new(bench);

    benchmark.random_free(&c, ALLOCATION_SIZES, ALIGNMENTS);
}
