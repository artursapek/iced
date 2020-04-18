use crate::{
    canvas::{Drawable, Frame, Layer, State},
    Primitive,
};

use iced_native::Size;
use std::{cell::RefCell, marker::PhantomData, sync::Arc};

enum CacheState {
    Empty,
    Filled {
        bounds: Size,
        primitive: Arc<Primitive>,
    },
}

impl Default for CacheState {
    fn default() -> Self {
        CacheState::Empty
    }
}
/// A simple cache that stores generated geometry to avoid recomputation.
///
/// A [`Cache`] will not redraw its geometry unless the dimensions of its layer
/// change or it is explicitly cleared.
///
/// [`Layer`]: ../trait.Layer.html
/// [`Cache`]: struct.Cache.html
#[derive(Debug)]
pub struct Cache<S>
where
    S: State,
{
    handler: PhantomData<dyn Drawable<S>>,
    state: RefCell<CacheState>,
}

impl<S> Default for Cache<S>
where
    S: State,
{
    fn default() -> Self {
        Self {
            handler: PhantomData,
            state: Default::default(),
        }
    }
}

impl<S> Cache<S>
where
    S: State,
{
    /// Creates a new empty [`Cache`].
    ///
    /// [`Cache`]: struct.Cache.html
    pub fn new() -> Self {
        Cache {
            handler: PhantomData,
            state: Default::default(),
        }
    }

    /// Clears the cache, forcing a redraw the next time it is used.
    ///
    /// [`Cached`]: struct.Cached.html
    pub fn clear(&mut self) {
        *self.state.borrow_mut() = CacheState::Empty;
    }

    /// Binds the [`Cache`] with some data, producing a [`Layer`] that can be
    /// added to a [`Canvas`].
    ///
    /// [`Cache`]: struct.Cache.html
    /// [`Layer`]: ../trait.Layer.html
    /// [`Canvas`]: ../../struct.Canvas.html
    pub fn with<'a, T: Drawable<S> + std::fmt::Debug>(&'a self, handler: &'a T) -> impl Layer<S> + 'a {
        Bind {
            cache: self,
            handler: handler,
        }
    }
}

#[derive(Debug)]
struct Bind<'a, S, T>
where
    S: State,
    T: Drawable<S>,
{
    cache: &'a Cache<S>,
    handler: &'a T,
}

impl<'a, S, T> Layer<S> for Bind<'a, S, T>
where
    S: State,
    T: Drawable<S> + std::fmt::Debug,
{
    fn draw(&self, current_bounds: Size, state: &S) -> Arc<Primitive> {
        use std::ops::Deref;

        if let CacheState::Filled { bounds, primitive } =
            self.cache.state.borrow().deref()
        {
            if *bounds == current_bounds {
                return primitive.clone();
            }
        }

        let mut frame = Frame::new(current_bounds.width, current_bounds.height);
        self.handler.draw(&mut frame, &state);

        let primitive = Arc::new(frame.into_primitive());

        *self.cache.state.borrow_mut() = CacheState::Filled {
            bounds: current_bounds,
            primitive: primitive.clone(),
        };

        primitive
    }
}

impl std::fmt::Debug for CacheState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheState::Empty => write!(f, "Empty"),
            CacheState::Filled { primitive, bounds } => f
                .debug_struct("Filled")
                .field("primitive", primitive)
                .field("bounds", bounds)
                .finish(),
        }
    }
}
