use std::{intrinsics::*, pin::Pin, task::Poll};

use futures::Stream;

use crate::util::GenIter;
#[derive(Debug, Clone)]
pub enum List<T> {
    Cons(T, Box<List<T>>),
    Nil,
}
impl<T> List<T> {
    pub fn new(iter: impl IntoIterator<Item = T>) -> Self {
        let mut tail = List::Nil;
        for item in iter {
            tail = List::Cons(item, Box::new(tail));
        }
        tail
    }
    ///Completely clears the the [`List`]. Nessecary to use if dropping a very large list all at once, because the default drop glue will blow out the stack, and manually implementing [`Drop`] breaks everything else.
    pub fn clear(&mut self) {
        if self.is_nil() {
            return;
        }
        let mut tmp = List::Nil;
        std::mem::swap(self, &mut tmp);
        loop {
            match tmp {
                List::Nil => break,
                List::Cons(_, next) => tmp = *next,
            }
        }
    }
    pub fn is_nil(&self) -> bool {
        matches!(self, List::Nil)
    }
    pub fn iter(&self) -> ListIterRef<'_, T> {
        self.into_iter()
    }
    pub fn into_generator(mut self) -> impl GenIter<T> {
        || loop {
            match self {
                List::Cons(t, tail) => {
                    if tail.is_nil() {
                        return t;
                    } else {
                        self = *tail;
                        yield t;
                    }
                }
                _ => panic!("attempted to `resume` an empty generator"),
            }
        }
    }
    pub fn generator(&self) -> impl GenIter<&T> {
        move || {
            let mut gen = self;
            loop {
                match gen {
                    List::Cons(t, tail) => {
                        if tail.is_nil() {
                            return t;
                        } else {
                            gen = tail;
                            yield t;
                        }
                    }
                    _ => panic!("attempted to `resume` an empty generator"),
                }
            }
        }
    }
    pub fn generator_prefetch(&self) -> impl GenIter<&T> {
        move || {
            let mut gen = self;
            loop {
                match gen {
                    List::Cons(t, tail) => {
                        if tail.is_nil() {
                            return t;
                        } else {
                            gen = tail;
                            match &gen {
                                List::Cons(_, next) => unsafe {
                                    prefetch_read_data::<List<T>>(&**next, 3)
                                },
                                List::Nil => unsafe { unreachable() },
                            }
                            yield t;
                        }
                    }
                    _ => panic!("attempted to `resume` an empty generator"),
                }
            }
        }
    }
    pub fn into_generator_prefetch(mut self) -> impl GenIter<T> {
        move || loop {
            match self {
                List::Cons(t, tail) => {
                    if tail.is_nil() {
                        return t;
                    } else {
                        self = *tail;
                        match &self {
                            List::Cons(_, next) => unsafe {
                                prefetch_read_data::<List<T>>(&**next, 3)
                            },
                            List::Nil => unsafe { unreachable() },
                        }
                        yield t;
                    }
                }
                _ => panic!("attempted to `resume` an empty generator"),
            }
        }
    }
    pub fn into_stream(self) -> ListStream<T> {
        ListStream(self)
    }
    pub fn into_stream_prefetch(self) -> ListStreamPrefetch<T> {
        ListStreamPrefetch(self)
    }
    pub fn stream(&self) -> ListStreamRef<'_, T> {
        ListStreamRef(self)
    }
    pub fn stream_prefetch(&self) -> ListStreamPrefetchRef<'_, T> {
        ListStreamPrefetchRef(self)
    }
    pub fn iter_prefetch(&self) -> ListIterPrefetchRef<'_, T> {
        ListIterPrefetchRef(self)
    }
    pub fn into_iter_prefetch(self) -> ListIterPrefetch<T> {
        ListIterPrefetch(self)
    }
}
pub struct ListIter<T>(List<T>);
impl<T> Iterator for ListIter<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let mut temp = List::Nil;
        std::mem::swap(&mut temp, &mut self.0);
        match temp {
            List::Cons(t, next) => {
                self.0 = *next;
                Some(t)
            }
            List::Nil => None,
        }
    }
}
pub struct ListIterPrefetch<T>(List<T>);
impl<T> Iterator for ListIterPrefetch<T> {
    type Item = T;
    fn next(&mut self) -> Option<T> {
        let mut temp = List::Nil;
        std::mem::swap(&mut temp, &mut self.0);
        match temp {
            List::Cons(t, next) => {
                self.0 = *next;
                if let List::Cons(_, next) = &self.0 {
                    unsafe { prefetch_read_data::<List<T>>(&**next, 3) }
                }
                Some(t)
            }
            List::Nil => None,
        }
    }
}
pub struct ListIterPrefetchRef<'a, T>(&'a List<T>);
impl<'a, T> Iterator for ListIterPrefetchRef<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut temp = &List::Nil;
        std::mem::swap(&mut temp, &mut self.0);
        match temp {
            List::Cons(t, next) => {
                self.0 = next;
                if let List::Cons(_, next) = &self.0 {
                    unsafe { prefetch_read_data::<List<T>>(&**next, 3) }
                }
                Some(t)
            }
            List::Nil => None,
        }
    }
}
impl<T> IntoIterator for List<T> {
    type IntoIter = ListIter<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        ListIter(self)
    }
}
impl<'a, T> IntoIterator for &'a List<T> {
    type IntoIter = ListIterRef<'a, T>;
    type Item = &'a T;
    fn into_iter(self) -> Self::IntoIter {
        ListIterRef(self)
    }
}
pub struct ListIterRef<'a, T>(&'a List<T>);
impl<'a, T> Iterator for ListIterRef<'a, T> {
    type Item = &'a T;
    fn next(&mut self) -> Option<Self::Item> {
        let mut temp = &List::Nil;
        std::mem::swap(&mut temp, &mut self.0);
        match temp {
            List::Cons(t, next) => {
                self.0 = next;

                Some(t)
            }
            List::Nil => None,
        }
    }
}
pub struct ListStream<T>(List<T>);
impl<T: Unpin> Stream for ListStream<T> {
    type Item = T;
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut temp = List::Nil;
        std::mem::swap(&mut self.0, &mut temp);
        match temp {
            List::Nil => Poll::Ready(None),
            List::Cons(t, next) => {
                self.0 = *next;
                Poll::Ready(Some(t))
            }
        }
    }
}
pub struct ListStreamPrefetch<T>(List<T>);
impl<T: Unpin> Stream for ListStreamPrefetch<T> {
    type Item = T;
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut temp = List::Nil;
        std::mem::swap(&mut self.0, &mut temp);
        match temp {
            List::Nil => Poll::Ready(None),
            List::Cons(t, next) => {
                self.0 = *next;
                if let List::Cons(_, next) = &self.0 {
                    unsafe { prefetch_read_data::<List<T>>(&**next, 3) }
                }
                Poll::Ready(Some(t))
            }
        }
    }
}

pub struct ListStreamRef<'a, T>(&'a List<T>);
impl<'a, T> Stream for ListStreamRef<'a, T> {
    type Item = &'a T;
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut temp = &List::Nil;
        std::mem::swap(&mut self.0, &mut temp);
        match temp {
            List::Nil => Poll::Ready(None),
            List::Cons(t, next) => {
                self.0 = next;
                Poll::Ready(Some(t))
            }
        }
    }
}

pub struct ListStreamPrefetchRef<'a, T>(&'a List<T>);
impl<'a, T> Stream for ListStreamPrefetchRef<'a, T> {
    type Item = &'a T;
    fn poll_next(
        mut self: Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        let mut temp = &List::Nil;
        std::mem::swap(&mut self.0, &mut temp);
        match temp {
            List::Nil => Poll::Ready(None),
            List::Cons(t, next) => {
                self.0 = next;
                if let List::Cons(_, next) = &self.0 {
                    unsafe { prefetch_read_data::<List<T>>(&**next, 3) }
                }
                Poll::Ready(Some(t))
            }
        }
    }
}
