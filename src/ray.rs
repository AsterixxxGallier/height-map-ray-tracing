use num_traits::Float;
use std::ops::{Add, Mul};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray2<T> {
    pub start_x: T,
    pub start_y: T,
    pub diff_x: T,
    pub diff_y: T,
}

impl<T: Float> Ray2<T> {
    pub fn sub_ray(&self, start_t: T, end_t: T) -> Self {
        Self {
            start_x: self.start_x + self.diff_x * start_t,
            start_y: self.start_y + self.diff_y * start_t,
            diff_x: self.diff_x * (end_t - start_t),
            diff_y: self.diff_y * (end_t - start_t),
        }
    }
}

impl<T: Copy + Add<Output = T>> Ray2<T> {
    pub fn end_x(&self) -> T {
        self.start_x + self.diff_x
    }

    pub fn end_y(&self) -> T {
        self.start_y + self.diff_y
    }
}

impl<T> Ray2<T> {
    pub fn with_z(self, start_z: T, diff_z: T) -> Ray3<T> {
        Ray3 {
            start_x: self.start_x,
            start_y: self.start_y,
            start_z,
            diff_x: self.diff_x,
            diff_y: self.diff_y,
            diff_z,
        }
    }

    pub fn scale<U>(self, factor: U) -> Ray2<T::Output>
    where
        T: Mul<U>,
        U: Copy,
    {
        Ray2 {
            start_x: self.start_x * factor,
            start_y: self.start_y * factor,
            diff_x: self.diff_x * factor,
            diff_y: self.diff_y * factor,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray3<T> {
    pub start_x: T,
    pub start_y: T,
    pub start_z: T,
    pub diff_x: T,
    pub diff_y: T,
    pub diff_z: T,
}

impl<T: Copy + Add<Output = T>> Ray3<T> {
    pub fn end_x(&self) -> T {
        self.start_x + self.diff_x
    }

    pub fn end_y(&self) -> T {
        self.start_y + self.diff_y
    }

    pub fn end_z(&self) -> T {
        self.start_z + self.diff_z
    }

    pub fn as_ray_2(&self) -> Ray2<T> {
        Ray2 {
            start_x: self.start_x,
            start_y: self.start_y,
            diff_x: self.diff_x,
            diff_y: self.diff_y,
        }
    }
}

impl<T: Copy + Mul<Output = T>> Ray3<T> {
    pub fn scale_x_y(self, factor: T) -> Ray3<T> {
        Ray3 {
            start_x: self.start_x * factor,
            start_y: self.start_y * factor,
            start_z: self.start_z,
            diff_x: self.diff_x * factor,
            diff_y: self.diff_y * factor,
            diff_z: self.diff_z,
        }
    }
}
