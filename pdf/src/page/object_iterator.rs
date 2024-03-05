use std::io::{Read, Seek};

use crate::page::content_interpreter::ContentInterpreter;
use crate::page::graphics_object::GraphicsObject;

pub struct ObjectIterator<'a, T: Seek + Read> {
    interpretor: ContentInterpreter<'a, T>,
}

impl<'a, T: Seek + Read> ObjectIterator<'a, T> {
    pub fn new(interpretor: ContentInterpreter<'a, T>) -> Self {
        ObjectIterator { interpretor }
    }
}

impl<'a, T: Seek + Read> Iterator for ObjectIterator<'a, T> {
    type Item = GraphicsObject;
    fn next(&mut self) -> Option<Self::Item> {
        let v = self.interpretor.poll();
        v.unwrap()
    }
}
