#![feature(generators)]
#![feature(generator_trait)]
#![feature(core_intrinsics)]
#![feature(trait_alias)]
use futures::StreamExt;
use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
    time::Instant,
};
use util::{gen_zip, GenIter};

use crate::{linked_list::List, util::execute};
mod linked_list;
mod util;
fn main() {
    println!("Bench owned");
    bench(&[
        ("iter::zip", |l, r| {
            l.into_iter().zip(r).fold(0, |a, (l, r)| a + l + r)
        }),
        ("generator", |l, r| {
            gen_zip_sum(l.into_generator(), r.into_generator())
        }),
        ("generator prefetch", |l, r| {
            gen_zip_sum(l.into_generator_prefetch(), r.into_generator_prefetch())
        }),
        ("stream::zip", |l, r| {
            execute(
                l.into_stream()
                    .zip(r.into_stream())
                    .fold(0, |a, (l, r)| async move { a + l + r }),
            )
        }),
        ("stream::zip prefetch", |l, r| {
            execute(
                l.into_stream()
                    .zip(r.into_stream())
                    .fold(0, |a, (l, r)| async move { a + l + r }),
            )
        }),
    ]);
    println!("Bench ref");
    bench_ref(&[
        ("iter::zip", |l, r| {
            l.into_iter().zip(r).fold(0, |a, (l, r)| a + l + r)
        }),
        ("generator", |l, r| {
            gen_zip_sum_ref(l.generator(), r.generator())
        }),
        ("generator prefetch", |l, r| {
            gen_zip_sum_ref(l.generator_prefetch(), r.generator_prefetch())
        }),
        ("stream::zip", |l, r| {
            execute(
                l.stream()
                    .zip(r.stream())
                    .fold(0, |a, (l, r)| async move { a + l + r }),
            )
        }),
        ("stream::zip prefetch", |l, r| {
            execute(
                l.stream_prefetch()
                    .zip(r.stream_prefetch())
                    .fold(0, |a, (l, r)| async move { a + l + r }),
            )
        }),
    ])
}
const END: i32 = 1024 * 1024 * 1024 / 16;
fn gen_lists() -> (List<i32>, List<i32>) {
    let range = 1..=END;
    (List::new(range.clone()), List::new(range))
}
type BenchFn<T> = fn(List<T>, List<T>) -> T;
type BenchFnRef<T> = fn(&List<T>, &List<T>) -> T;
fn bench(funcs: &[(&str, BenchFn<i32>)]) {
    for (s, f) in funcs {
        let (l, r) = gen_lists();
        let now = Instant::now();
        println!("bench: {s} res: {} time: {:?}", f(l, r), now.elapsed());
    }
}
fn bench_ref(funcs: &[(&str, BenchFnRef<i32>)]) {
    for (s, f) in funcs {
        let (mut l, mut r) = gen_lists();
        let now = Instant::now();
        println!("bench: {s} res: {} time: {:?}", f(&l, &r), now.elapsed());
        l.clear();
        r.clear();
    }
}
fn gen_zip_sum(l: impl GenIter<i32>, r: impl GenIter<i32>) -> i32 {
    let mut sum = 0;
    let mut gen = gen_zip(l, r);
    loop {
        let pin = unsafe { Pin::new_unchecked(&mut gen) };
        match pin.resume(()) {
            GeneratorState::Complete((l, r)) => return sum + l + r,
            GeneratorState::Yielded((l, r)) => sum += l + r,
        }
    }
}
fn gen_zip_sum_ref<'a>(l: impl GenIter<&'a i32>, r: impl GenIter<&'a i32>) -> i32 {
    let mut sum = 0;
    let mut gen = gen_zip(l, r);
    loop {
        let pin = unsafe { Pin::new_unchecked(&mut gen) };
        match pin.resume(()) {
            GeneratorState::Complete((l, r)) => return sum + l + r,
            GeneratorState::Yielded((l, r)) => sum += l + r,
        }
    }
}
