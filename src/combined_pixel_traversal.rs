use crate::combined_traversal::{BoundaryCrossing, CombinedBoundaryTraversal};
use crate::ray::Ray;

pub struct CombinedPixelTraversal {
    boundary_traversal: CombinedBoundaryTraversal,
    last_t: f32,
    current: Option<(i32, i32)>,
}

impl CombinedPixelTraversal {
    pub fn new(ray: Ray) -> Self {
        let boundary_traversal = CombinedBoundaryTraversal::new(ray);
        Self {
            last_t: 0.0,
            current: Some((boundary_traversal.pixel_x(), boundary_traversal.pixel_y())),
            boundary_traversal,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PixelSegment {
    pub pixel_x: i32,
    pub pixel_y: i32,
    pub start_t: f32,
    pub end_t: f32,
}

impl Iterator for CombinedPixelTraversal {
    type Item = PixelSegment;

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
                end_t: 1.0,
            })
        }
    }
}
