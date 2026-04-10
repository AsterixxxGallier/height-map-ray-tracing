use crate::ray::Ray;
use num_traits::Float;

/// The coordinate of the first pixel entered by the ray that starts at `start` and moves in the
/// direction given by `diff`. If `start` has a non-zero fractional part, i.e. is between two
/// integers, then `start.floor() as i32` is returned.
///
/// If `start` is an integer, the return value depends on the sign of `diff`. For `diff >= 0.0`,
/// `start as i32` is returned, and for `diff < 0.0`, `start as i32 - 1` is returned.
#[inline]
pub(crate) fn initial_pixel_coordinate<T: Float>(start: T, diff: T) -> i32 {
    if diff >= T::zero() {
        start.floor().to_i32().unwrap()
    } else {
        start.ceil().to_i32().unwrap() - 1
    }
}

/// The absolute difference between `start` and the next (non-equal) integer in direction of `diff`.
///
/// For `diff >= 0.0`, this is the absolute difference between `start` and the smallest integer
/// greater than `start`. For `diff < 0.0`, this is the absolute difference between `start` and the
/// largest integer less than `start`.
///
/// For integer values of `start`, this is always `1.0`.
///
/// In all cases, the return value of this function is greater than `0.0`.
#[inline]
pub(crate) fn distance_to_integer_boundary<T: Float>(start: T, diff: T) -> T {
    if diff >= T::zero() {
        // If start is an integer, this is just 1.
        // Otherwise, this is start.ceil() - start (the difference to the next-up integer).
        start.floor() - start + T::one()
    } else {
        // If start is an integer, this is just 1.
        // Otherwise, this is start - start.floor() (the difference to the next-down integer).
        start - start.ceil() + T::one()
    }
}

/// The number of integers greater than `start` and less than `end`.
///
/// For example, for `start = 2.0` and `end = 5.0`, this is `2`, because there are only two integers
/// (three and four) greater than `2.0` and less than `5.0`.
#[inline]
pub(crate) fn integers_between_ordered<T: Float>(start: T, end: T) -> usize {
    // If start = end and both are integers, `end.ceil() - start.floor()` will be zero. Subtracting
    // one at the end would result in an underflow error. We avoid this by using `saturating_sub`,
    // which yields zero in this case, which is the correct result.
    (end.ceil() - start.floor())
        .to_usize()
        .unwrap()
        .saturating_sub(1)
}

/// The number of integers in the open range bounded by `start` on one side and `end` on the other.
///
/// If `start <= end`, this is `integers_between_ordered(start, end)`, and for `end < start`, this
/// is `integers_between_ordered(end, start)`. See [`integers_between_ordered`] for more
/// information.
#[inline]
pub(crate) fn integers_between<T: Float>(start: T, end: T) -> usize {
    if start <= end {
        integers_between_ordered(start, end)
    } else {
        integers_between_ordered(end, start)
    }
}

/// Integer and floating-point variables essential to the pixel boundary traversal algorithm
/// introduced by Amanatides and Woo in their paper "A Fast Voxel Traversal Algorithm".
#[derive(Debug, Clone)]
pub(crate) struct CombinedBoundaryTraversalVariables<T> {
    pub(crate) step_x: i32,
    pub(crate) step_y: i32,
    pub(crate) pixel_x: i32,
    pub(crate) pixel_y: i32,
    pub(crate) t_delta_x: T,
    pub(crate) t_delta_y: T,
    pub(crate) t_max_x: T,
    pub(crate) t_max_y: T,
}

impl<T: Float> CombinedBoundaryTraversalVariables<T> {
    pub(crate) fn new(ray: Ray<T>) -> Self {
        let step_x = ray.diff_x.signum().to_i32().unwrap();
        let step_y = ray.diff_y.signum().to_i32().unwrap();

        let pixel_x = initial_pixel_coordinate(ray.start_x, ray.diff_x);
        let pixel_y = initial_pixel_coordinate(ray.start_y, ray.diff_y);

        // difference in t that corresponds to a difference in x of exactly 1.0
        let t_delta_x = ray.diff_x.recip().abs();
        // difference in t that corresponds to a difference in y of exactly 1.0
        let t_delta_y = ray.diff_y.recip().abs();

        // absolute difference between start_x and the next x-boundary
        let dist_x = distance_to_integer_boundary(ray.start_x, ray.diff_x);
        // absolute difference between start_y and the next y-boundary
        let dist_y = distance_to_integer_boundary(ray.start_y, ray.diff_y);

        // the value of t at which the next x-boundary is crossed
        // = the value of t at which x is maximal before crossing over the next x-boundary
        let t_max_x = t_delta_x * dist_x;
        // the value of t at which the next y-boundary is crossed
        // = the value of t at which y is maximal before crossing over the next y-boundary
        let t_max_y = t_delta_y * dist_y;

        Self {
            step_x,
            step_y,
            pixel_x,
            pixel_y,
            t_delta_x,
            t_delta_y,
            t_max_x,
            t_max_y,
        }
    }

    #[inline]
    pub(crate) fn step_x(&mut self) {
        self.t_max_x = self.t_max_x + self.t_delta_x;
        self.pixel_x += self.step_x;
    }

    #[inline]
    pub(crate) fn step_y(&mut self) {
        self.t_max_y = self.t_max_y + self.t_delta_y;
        self.pixel_y += self.step_y;
    }
}
