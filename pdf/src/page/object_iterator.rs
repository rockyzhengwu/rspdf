use crate::page::content_interpreter::ContentInterpreter;
use crate::page::graphics_object::GraphicsObject;
use std::io::{Read, Seek};

pub struct ObjectIterator<'a, T: Seek + Read> {
    interator: ContentInterpreter<'a, T>,
}

impl<'a, T: Seek + Read> Iterator for ObjectIterator<'a, T> {
    type Item = GraphicsObject;
    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}
