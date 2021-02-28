use std::ops::{Generator, GeneratorState};
use std::pin::Pin;

pub struct GeneratorIteratorAdapter<G: Generator>(Pin<Box<G>>);

impl<G: Generator> GeneratorIteratorAdapter<G> {
    pub(crate) fn new(gen: G) -> Self {
        Self(Box::pin(gen))
    }
}

impl<G: Generator> Iterator for GeneratorIteratorAdapter<G> {
    type Item = G::Yield;

    fn next(&mut self) -> Option<Self::Item> {
        match self.0.as_mut().resume(()) {
            GeneratorState::Yielded(x) => Some(x),
            GeneratorState::Complete(_) => None,
        }
    }
}
