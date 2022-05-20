#![feature(generators)]
#![feature(generator_trait)]
#![feature(core_intrinsics)]

use std::{
    ops::{Generator, GeneratorState},
    pin::Pin,
    time::Instant,
};

use crate::linked_list::List;
mod linked_list;
mod util;
fn main() {
    bench(&[
        ("iter::zip", |l, r| {
            l.into_iter().zip(r).fold(0, |a, (l, r)| a + l + r)
        }),
        ("generator", |l, r| {
            gen_zip(l.into_generator(), r.into_generator())
        }),
        ("generator prefetch", |l, r| {
            gen_zip(l.into_generator_prefetch(), r.into_generator_prefetch())
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
fn gen_zip(
    mut l: impl Generator<Yield = i32, Return = i32>,
    mut r: impl Generator<Yield = i32, Return = i32>,
) -> i32 {
    let mut sum = 0;
    loop {
        let l_pin = unsafe { Pin::new_unchecked(&mut l) };
        let r_pin = unsafe { Pin::new_unchecked(&mut r) };

        match (l_pin.resume(()), r_pin.resume(())) {
            (GeneratorState::Complete(l), GeneratorState::Complete(r)) => {
                return sum + l + r;
            }
            (GeneratorState::Yielded(l), GeneratorState::Yielded(r)) => sum += l + r,
            (GeneratorState::Yielded(l), GeneratorState::Complete(r)) => sum += l + r,
            (GeneratorState::Complete(l), GeneratorState::Yielded(r)) => sum += l + r,
        }
    }
}
