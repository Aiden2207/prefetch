# Prefetch Benchmarks

After viewing [this](https://stackoverflow.com/questions/72297210/prefetching-and-yielding-to-hide-cache-misses-in-rust) stack overflow question and reading one of the linked papers I got thoroughly nerd-sniped. The result of that is this code- using Rust async and generator based code to implement interleaved prefetching of values to reduce the impact of cache misses during pointer chasing. The papers I mentioned previously tested the technique on various database style data structures, but I went with something simpler by using an interleaved traversal of 1 Gib linked lists.

## Benchmarks

There are two benchmarks, each with eight different algorithms. The first benchmark, `bench owned`, builds a very large `list<i32>` that is consumed, summing all of the elements. This benchmark is dominated by deallocation, rather than the traversal. The second is `bench ref` where an immutable reference is used to traverse the `List<i32>` and sum it, with deallocation times not being measured.

The `iter::zip` and `iter::chain` algorithms use a simple iterator in combination with `Iterator::zip` and `Iterator::chain` to combine the list. The `generator zip` and `generator chain` algorithms use a custom baked code combine [`Generators`](https://doc.rust-lang.org/beta/unstable-book/language-features/generators.html) in a similar fashion to `Iterator::zip` and `Iterator::chain`. The `generator prefetch` is the same as `generator zip` with prefetching. The `stream` tests work the same as the equivalent `generator` ones, but using [`futures::Stream`s](https://docs.rs/futures/latest/futures/stream/trait.Stream.html) instead.

## Running the Benchmarks

You'll need nightly rust and 8 Gb of RAM to run the tests. Then, it's just `cargo +nightly run --release`.

## Results

This is the output on my machine, which has an Intel(R) Core(TM) i5-8365U CPU and 24 Gb of RAM.

```
Bench owned
bench: iter::zip result: 67108864 time: 10.4643775s
bench: iter::chain result: 67108864 time: 9.3184379s
bench: generator zip result: 67108864 time: 18.3766084s
bench: generator chain result: 67108864 time: 9.87299s
bench: generator prefetch result: 67108864 time: 15.2534273s
bench: stream::zip result: 67108864 time: 16.1874495s
bench: stream::chain result: 67108864 time: 8.4043329s
bench: stream::zip prefetch result: 67108864 time: 11.6481492s
Bench ref
bench: iter::zip result: 67108864 time: 813.3859ms
bench: iter::chain result: 67108864 time: 1.1475895s
bench: generator zip result: 67108864 time: 931.5528ms
bench: generator chain result: 67108864 time: 1.4673188s
bench: generator prefetch result: 67108864 time: 902.6999ms
bench: stream::zip result: 67108864 time: 1.0911242s
bench: stream::chain result: 67108864 time: 1.6229788s
bench: stream::zip prefetch result: 67108864 time: 1.2560384s
```

In conclusion, the prefetching technique can provide meaningful performance improvements, as shown in the `bench owned` benchmark, but not always, and not better than what using a simpler design could.

## License

Licensed under either of

* Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
* MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
