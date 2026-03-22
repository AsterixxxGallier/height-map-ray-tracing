#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Bounds {
    pub min_x: i32,
    pub min_y: i32,
    pub max_x: i32,
    pub max_y: i32,
}

impl Bounds {
    pub fn origin_rectangle(x_size: i32, y_size: i32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: x_size,
            max_y: y_size,
        }
    }

    pub fn origin_square(size: i32) -> Self {
        Self {
            min_x: 0,
            min_y: 0,
            max_x: size,
            max_y: size,
        }
    }

    pub fn contains(&self, x: i32, y: i32) -> bool {
        self.min_x <= x && x < self.max_x && self.min_y <= y && y < self.max_y
    }
}
