use num_traits::Float;
use crate::ray::Ray2;
use crate::traversal::boundary::BoundaryTraversal;

pub struct PixelTraversal<T> {
    boundary_traversal: BoundaryTraversal<T>,
    last_t: T,
    current_pixel: Option<(i32, i32)>,
}

impl<T: Float> PixelTraversal<T> {
    pub fn new(ray: Ray2<T>) -> Self {
        let boundary_traversal = BoundaryTraversal::new(ray);
        Self {
            last_t: T::zero(),
            current_pixel: Some((boundary_traversal.pixel_x(), boundary_traversal.pixel_y())),
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

impl<T: Float> Iterator for PixelTraversal<T> {
    type Item = PixelSegment<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let (pixel_x, pixel_y) = self.current_pixel?;
        if let Some(crossing) = self.boundary_traversal.next() {
            self.current_pixel = Some((crossing.pixel_x, crossing.pixel_y));
            let start_t = self.last_t;
            self.last_t = crossing.t;
            Some(PixelSegment {
                pixel_x,
                pixel_y,
                start_t,
                end_t: crossing.t,
            })
        } else {
            self.current_pixel = None;
            Some(PixelSegment {
                pixel_x,
                pixel_y,
                start_t: self.last_t,
                end_t: T::one(),
            })
        }
    }
}
