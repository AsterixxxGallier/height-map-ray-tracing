use std::fmt::Debug;
use crate::ray::Ray;
use crate::separate_traversal::{XBoundaryTraversal, YBoundaryTraversal};
use num_traits::Float;

#[cfg(test)]
mod tests;

pub struct XPixelTraversal<T> {
    boundary_traversal: XBoundaryTraversal<T>,
    last_t: T,
    current: Option<(i32, i32)>,
}

impl<T: Float + Debug> XPixelTraversal<T> {
    pub fn new(ray: Ray<T>) -> Option<Self> {
        let boundary_traversal = XBoundaryTraversal::new(ray)?;
        // dbg!(&ray);
        // dbg!(&boundary_traversal);
        Some(Self {
            last_t: T::zero(),
            current: Some((ray.start_x.to_i32().unwrap(), ray.start_y.to_i32().unwrap())),
            boundary_traversal,
        })
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PixelSegment<T> {
    pub pixel_x: i32,
    pub pixel_y: i32,
    pub start_t: T,
    pub end_t: T,
}

impl<T: Float> Iterator for XPixelTraversal<T> {
    type Item = PixelSegment<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (pixel_x, pixel_y) = self.current?;
        if let Some(crossing) = self.boundary_traversal.next() {
            let (new_t, new_x, new_y) = (crossing.t, crossing.next_x_index, crossing.y_index);
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

pub struct YPixelTraversal<T> {
    boundary_traversal: YBoundaryTraversal<T>,
    last_t: T,
    current: Option<(i32, i32)>,
}

impl<T: Float> YPixelTraversal<T> {
    pub fn new(ray: Ray<T>) -> Option<Self> {
        let boundary_traversal = YBoundaryTraversal::new(ray)?;
        Some(Self {
            last_t: T::zero(),
            current: Some((ray.start_x.to_i32().unwrap(), ray.start_y.to_i32().unwrap())),
            boundary_traversal,
        })
    }
}

impl<T: Float> Iterator for YPixelTraversal<T> {
    type Item = PixelSegment<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (pixel_x, pixel_y) = self.current?;
        if let Some(crossing) = self.boundary_traversal.next() {
            let (new_t, new_x, new_y) = (crossing.t, crossing.x_index, crossing.next_y_index);
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
