use num_traits::Float;
use crate::ray::Ray;
use crate::boundary_traversal::{BoundaryCrossing, CombinedBoundaryTraversal};

#[cfg(test)]
mod tests;

pub struct CombinedPixelTraversal<T> {
    boundary_traversal: CombinedBoundaryTraversal<T>,
    last_t: T,
    current: Option<(i32, i32)>,
}

impl<T: Float> CombinedPixelTraversal<T> {
    pub fn new(ray: Ray<T>) -> Self {
        let boundary_traversal = CombinedBoundaryTraversal::new(ray);
        Self {
            last_t: T::zero(),
            current: Some((boundary_traversal.pixel_x(), boundary_traversal.pixel_y())),
            boundary_traversal,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PixelSegment<T> {
    pub pixel_x: i32,
    pub pixel_y: i32,
    pub start_t: T,
    pub end_t: T,
}

impl<T: Float> Iterator for CombinedPixelTraversal<T> {
    type Item = PixelSegment<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (pixel_x, pixel_y) = self.current?;
        if let Some(crossing) = self.boundary_traversal.next() {
            let (new_t, new_x, new_y) = match crossing {
                BoundaryCrossing::X {
                    t,
                    last_x_index: _,
                    next_x_index,
                    y_index,
                } => (t, next_x_index, y_index),
                BoundaryCrossing::Y {
                    t,
                    x_index,
                    last_y_index: _,
                    next_y_index,
                } => (t, x_index, next_y_index),
                BoundaryCrossing::XY {
                    t,
                    last_x_index: _,
                    next_x_index,
                    last_y_index: _,
                    next_y_index,
                } => (t, next_x_index, next_y_index),
            };
            self.current = Some((new_x, new_y));
            let start_t = self.last_t;
            self.last_t = new_t;
            Some(PixelSegment {
                pixel_x,
                pixel_y,
                start_t,
                end_t: new_t,
            })
        } else {
            self.current = None;
            Some(PixelSegment {
                pixel_x,
                pixel_y,
                start_t: self.last_t,
                end_t: T::one(),
            })
        }
    }
}
