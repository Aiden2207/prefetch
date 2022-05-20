use futures::{Future, Stream};
use std::pin::Pin;
use std::task::Poll;
pub fn from_fn<T, F: Future<Output = T>>(stream: impl FnMut() -> F) -> impl Stream<Item = T> {
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
            return result.map(Some);
        } else {
            stream.cache = Some((stream.f)());
            self = unsafe { Pin::new_unchecked(stream) };
            return self.poll_next(cx);
        }
    }
}
