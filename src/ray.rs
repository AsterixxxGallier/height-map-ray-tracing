use std::ops::Add;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Ray<T> {
    pub start_x: T,
    pub start_y: T,
    pub diff_x: T,
    pub diff_y: T,
}

impl<T: Copy + Add<Output=T>> Ray<T> {
    pub fn end_x(&self) -> T {
        self.start_x + self.diff_x
    }
    
    pub fn end_y(&self) -> T {
        self.start_y + self.diff_y
    }
}