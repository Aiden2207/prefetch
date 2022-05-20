use futures::Stream;
use std::intrinsics::*;
use std::ops::Generator;
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
    pub fn is_nil(&self) -> bool {
        matches!(self, List::Nil)
    }
    pub fn into_generator(mut self) -> impl Generator<Yield = T, Return = T> {
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
    pub fn into_generator_prefetch(
        mut self,
        locality: i32,
    ) -> impl Generator<Yield = T, Return = T> {
        move || loop {
            match self {
                List::Cons(t, tail) => {
                    if tail.is_nil() {
                        return t;
                    } else {
                        self = *tail;
                        match &self {
                            List::Cons(_, next) => unsafe { prefetch_read_data(&*next, locality) },
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
impl<T> IntoIterator for List<T> {
    type IntoIter = ListIter<T>;
    type Item = T;
    fn into_iter(self) -> Self::IntoIter {
        ListIter(self)
    }
}
