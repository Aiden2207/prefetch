#![feature(generators)]
#![feature(generator_trait)]
#![feature(core_intrinsics)]

use std::time::Instant;

use crate::linked_list::List;
mod linked_list;
mod util;
fn main() {
    bench(&[|l, r| l.into_iter().zip(r).fold(0, |a, (l, r)| a + l + r)])
}
const END: i32 = 1024 * 1024 * 1024 / 16;
fn gen_lists() -> (List<i32>, List<i32>) {
    let range = 1..=END;
    (List::new(range.clone()), List::new(range))
}
fn bench(funcs: &[fn(List<i32>, List<i32>) -> i32]) {
    for f in funcs {
        let (l, r) = gen_lists();
        let now = Instant::now();
        println!("res: {} time: {:?}", f(l, r), now.elapsed())
    }
}
