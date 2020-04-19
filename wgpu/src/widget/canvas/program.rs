use crate::canvas::{Event, Layer, Size, Drawable, layer::Cache};

pub trait Program {
    fn layers<'a>(&'a self) -> Vec<Box<dyn Layer + 'a>>;

    fn update<'a>(&'a mut self, _event: Event, _bounds: Size) {}
}

#[derive(Debug)]
pub struct BasicProgram<T>
where
    T: Drawable + std::fmt::Debug,
{
    pub input: T,
    pub layer: Cache<T>,
}

impl<T> Program for BasicProgram<T>
where
    T: Drawable + std::fmt::Debug,
{
    fn layers<'a>(&'a self) -> Vec<Box<dyn Layer + 'a>> {
        vec![Box::new(self.layer.with(&self.input))]
    }
}


