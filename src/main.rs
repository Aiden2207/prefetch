#![feature(generators)]
#![feature(generator_trait)]
#![feature(core_intrinsics)]
#![feature(trait_alias)]
use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
    time::Instant,
};

use util::{gen_zip, GenIter};

use crate::linked_list::List;
mod linked_list;
mod util;
fn main() {
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
    ])
}
const END: i32 = 1024 * 1024 * 1024 / 16;
fn gen_lists() -> (List<i32>, List<i32>) {
    let range = 1..=END;
    (List::new(range.clone()), List::new(range))
}
type BenchFn<T> = fn(List<T>, List<T>) -> T;
fn bench(funcs: &[(&str, BenchFn<i32>)]) {
    for (s, f) in funcs {
        let (l, r) = gen_lists();
        let now = Instant::now();
        println!("bench: {s} res: {} time: {:?}", f(l, r), now.elapsed())
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
