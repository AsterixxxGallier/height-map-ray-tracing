use std::cmp::Ordering;
use crate::bounds::Bounds;
use crate::ray::Ray;

#[cfg(test)]
mod tests;

/// Based on A Fast Voxel Traversal Algorithm by Amanatides and Woo.
///
/// Iterator over [`BoundaryCrossing`]s, where a boundary is considered crossed if the [`Ray`] is
/// on both of its sides. For x-boundaries, this means that the ray is on the left and right sides
/// of the boundary. For y-boundaries, this means that the ray is on the top and bottom sides of the
/// boundary. It does not suffice for the ray start or end to touch a boundary.
///
/// When the ray hits a grid corner (e.g. x and y are both integers at the same time), the
/// x-boundary crossing is produced first, then the y-boundary crossing.
///
/// All crossings produced will be within [`Bounds`], with `t` between `0.0` and `1.0` (both bounds
/// are exclusive, as `t == 0.0` and `t == 1.0` cannot correspond to _crossing_ a boundary, only to
/// _touching_ it). Once `t` is `1.0` or `x` and `y` have moved out of bounds, the iterator returns
/// `None` for all future calls to `next`.
///
/// When the ray moves out of x-bounds while crossing a corner, only the x-boundary crossing will be
/// produced. This behaviour follows from the already stated behaviour of the iterator, as the
/// (immediately following) y-boundary crossing will already have an out-of-bounds x value.
#[derive(Debug)]
pub struct CombinedBoundaryTraversal {
    step_x: i32,
    step_y: i32,
    pixel_x: i32,
    pixel_y: i32,
    t_delta_x: f32,
    t_delta_y: f32,
    t_max_x: f32,
    t_max_y: f32,
    n_iters: usize,
}

impl CombinedBoundaryTraversal {
    pub fn new(ray: Ray, bounds: Bounds) -> Self {
        let step_x = ray.diff_x.signum() as i32;
        let step_y = ray.diff_y.signum() as i32;

        // the x-index of the pixel that the ray starts in
        // if start_x is an integer, this depends on the x-direction of the ray:
        // - for x-increasing rays, start_x is interpreted to be on the x-upper side of the
        //   x-boundary, so pixel_x = start_x
        // - for x-decreasing rays, start_x is interpreted to be on the x-lower side of the
        //   x-boundary, so pixel_x = start_x - 1
        let pixel_x = if ray.diff_x >= 0.0 {
            // for x-increasing rays, this is simply start_x.floor()
            ray.start_x as i32
        } else {
            // for x-decreasing rays, this is start_x - 1 if start_x is an integer, and
            // start_x.floor() otherwise
            ray.start_x.ceil() as i32 - 1
        };
        // the y-index of the pixel that the ray starts in
        // if start_y is an integer, this depends on the y-direction of the ray:
        // - for y-increasing rays, start_y is interpreted to be on the y-upper side of the
        //   y-boundary, so pixel_y = start_y
        // - for y-decreasing rays, start_y is interpreted to be on the y-lower side of the
        //   y-boundary, so pixel_y = start_y - 1
        let pixel_y = if ray.diff_y >= 0.0 {
            // for y-increasing rays, this is simply start_y.floor()
            ray.start_y as i32
        } else {
            // for y-decreasing rays, this is start_y - 1 if start_y is an integer, and
            // start_y.floor() otherwise
            ray.start_y.ceil() as i32 - 1
        };

        // difference in t that corresponds to a difference in x of exactly 1.0
        let t_delta_x = ray.diff_x.recip().abs();
        // difference in t that corresponds to a difference in y of exactly 1.0
        let t_delta_y = ray.diff_y.recip().abs();

        // absolute difference between start_x and the next x-boundary within bounds
        let dist_x = if ray.diff_x >= 0.0 {
            // if start_x is an integer, this is start_x + 1; else, it is ceil(start_x)
            let next_up_x = ray.start_x.floor() + 1.0;
            // the minimum x-boundary that can be crossed
            let min_x_boundary = bounds.min_x as f32;
            let first_x_boundary_crossable = next_up_x.max(min_x_boundary);
            first_x_boundary_crossable - ray.start_x
        } else {
            // if start_x is an integer, this is start_x - 1; else, it is floor(start_x)
            let next_down_x = ray.start_x.ceil() - 1.0;
            // the maximum x-boundary that can be crossed
            let max_x_boundary = bounds.max_x as f32;
            let first_x_boundary_crossable = next_down_x.min(max_x_boundary);
            ray.start_x - first_x_boundary_crossable
        };
        // absolute difference between start_y and the next y-boundary within bounds
        let dist_y = if ray.diff_y >= 0.0 {
            // if start_y is an integer, this is start_y + 1; else, it is ceil(start_y)
            let next_up_y = ray.start_y.floor() + 1.0;
            // the minimum y-boundary that can be crossed
            let min_y_boundary = bounds.min_y as f32;
            let first_y_boundary_crossable = next_up_y.max(min_y_boundary);
            first_y_boundary_crossable - ray.start_y
        } else {
            // if start_y is an integer, this is start_y - 1; else, it is floor(start_y)
            let next_down_y = ray.start_y.ceil() - 1.0;
            // the maximum y-boundary that can be crossed
            let max_y_boundary = bounds.max_y as f32;
            let first_y_boundary_crossable = next_down_y.min(max_y_boundary);
            ray.start_y - first_y_boundary_crossable
        };

        // the value of t at which the next x-boundary is crossed
        // = the value of t at which x is maximal before crossing over the next x-boundary
        let t_max_x = if t_delta_x.is_finite() {
            t_delta_x * dist_x
        } else {
            f32::INFINITY
        };
        // the value of t at which the next y-boundary is crossed
        // = the value of t at which y is maximal before crossing over the next y-boundary
        let t_max_y = if t_delta_y.is_finite() {
            t_delta_y * dist_y
        } else {
            f32::INFINITY
        };

        let x_crossings = if ray.diff_x > 0.0 {
            // number of integers between ray.start_x and ray.end_x()
            (ray.end_x().ceil() - ray.start_x.floor()) as usize - 1
        } else if ray.diff_x < 0.0 {
            (ray.start_x.ceil() - ray.end_x().floor()) as usize - 1
        } else {
            0
        };
        let y_crossings = if ray.diff_y > 0.0 {
            (ray.end_y().ceil() - ray.start_y.floor()) as usize - 1
        } else if ray.diff_y < 0.0 {
            (ray.start_y.ceil() - ray.end_y().floor()) as usize - 1
        } else {
            0
        };
        let n_iters = x_crossings + y_crossings;

        Self {
            step_x,
            step_y,
            pixel_x,
            pixel_y,
            t_delta_x,
            t_delta_y,
            t_max_x,
            t_max_y,
            n_iters,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum BoundaryCrossing {
    X {
        t: f32,
        last_x_index: i32,
        next_x_index: i32,
        y_index: i32,
    },
    Y {
        t: f32,
        x_index: i32,
        last_y_index: i32,
        next_y_index: i32,
    },
}

impl Iterator for CombinedBoundaryTraversal {
    type Item = BoundaryCrossing;

    fn next(&mut self) -> Option<Self::Item> {
        if self.n_iters == 0 {
            return None;
        }

        self.n_iters -= 1;

        let t = self.t_max_x.min(self.t_max_y);

        if self.t_max_x <= self.t_max_y {
            // we cross over the next x-boundary first
            let last_x_index = self.pixel_x;

            self.t_max_x += self.t_delta_x;
            self.pixel_x += self.step_x;

            let next_x_index = self.pixel_x;
            let y_index = self.pixel_y;

            Some(BoundaryCrossing::X {
                t,
                last_x_index,
                next_x_index,
                y_index,
            })
        } else {
            // we cross over the next y-boundary first
            let last_y_index = self.pixel_y;

            self.t_max_y += self.t_delta_y;
            self.pixel_y += self.step_y;

            let next_y_index = self.pixel_y;
            let x_index = self.pixel_x;

            Some(BoundaryCrossing::Y {
                t,
                x_index,
                last_y_index,
                next_y_index,
            })
        }
    }
}
