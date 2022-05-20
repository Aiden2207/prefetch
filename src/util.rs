use futures::{Future, Stream};
use std::ops::{Generator, GeneratorState};
use std::pin::Pin;
use std::task::Poll;
pub fn fn_stream<T, F: Future<Output = T>>(stream: impl FnMut() -> F) -> impl Stream<Item = T> {
    FnStream {
        f: stream,
        cache: None,
    }
}
struct FnStream<Func, Fut> {
    f: Func,
    cache: Option<Fut>,
}
impl<Func: FnMut() -> Fut, Fut: Future<Output = T>, T> Stream for FnStream<Func, Fut> {
    type Item = T;
    fn poll_next(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let stream = unsafe { self.get_unchecked_mut() };
        if let Some(fut) = stream.cache.as_mut() {
            let fut = unsafe { Pin::new_unchecked(fut) };
            let result = fut.poll(cx);
            if result.is_ready() {
                stream.cache = None;
            }
            result.map(Some)
        } else {
            stream.cache = Some((stream.f)());
            self = unsafe { Pin::new_unchecked(stream) };
            self.poll_next(cx)
        }
    }
}
pub trait GenIter<T> = Generator<Yield = T, Return = T>;
pub fn gen_zip<T, U>(mut l: impl GenIter<T>, mut r: impl GenIter<U>) -> impl GenIter<(T, U)> {
    move || loop {
        let l = unsafe { Pin::new_unchecked(&mut l) };
        let r = unsafe { Pin::new_unchecked(&mut r) };
        match (l.resume(()), r.resume(())) {
            (GeneratorState::Complete(l), GeneratorState::Complete(r)) => return (l, r),
            (GeneratorState::Complete(l), GeneratorState::Yielded(r)) => return (l, r),
            (GeneratorState::Yielded(l), GeneratorState::Complete(r)) => return (l, r),
            (GeneratorState::Yielded(l), GeneratorState::Yielded(r)) => yield (l, r),
        }
    }
}
