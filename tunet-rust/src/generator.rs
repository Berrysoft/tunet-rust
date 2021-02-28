use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

pub struct GeneratorIteratorAdapter<G: Generator> {
    gen: Pin<Box<G>>,
    ret: Option<G::Return>,
}

impl<G: Generator> GeneratorIteratorAdapter<G> {
    pub(crate) fn new(gen: G) -> Self {
        Self {
            gen: Box::pin(gen),
            ret: None,
        }
    }

    pub fn into_ret(self) -> Option<G::Return> {
        self.ret
    }
}

impl<G: Generator> Iterator for GeneratorIteratorAdapter<G> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match self.ret.as_ref() {
            None => match self.gen.as_mut().resume(()) {
                GeneratorState::Yielded(x) => Some(x),
                GeneratorState::Complete(ret) => {
                    self.ret = Some(ret);
                    None
                }
            },
            Some(_) => None,
        }
    }
}
