use crate::is_line_free;
use crate::map::Map;
use crate::ray::Ray3;
use num_traits::Float;

pub struct Tile {
    high_res: Map<f32>,
    low_res: Map<f32>,
}

impl Tile {
    pub fn new(map: Map<f32>) -> Self {
        assert_eq!(map.x_len(), 2000);
        assert_eq!(map.y_len(), 2000);
        let low_res = Map::from_fn(125, 125, |x_index, y_index| {
            let mut max = f32::NEG_INFINITY;
            for i in 0..16 {
                for j in 0..16 {
                    max = max.max(map.get(x_index * 16 + i, y_index * 16 + j));
                }
            }
            max
        });
        Self {
            high_res: map,
            low_res,
        }
    }

    pub fn map(&self) -> &Map<f32> {
        &self.high_res
    }

    pub fn is_line_free<T: Float>(&self, ray: Ray3<T>) -> bool {
        is_line_free(&self.low_res, ray.scale_x_y(T::from(1.0 / 16.0).unwrap()))
            || is_line_free(&self.high_res, ray)
    }
}
