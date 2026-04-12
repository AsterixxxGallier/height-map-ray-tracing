use crate::map::Map;
use crate::ray::Ray3;
use crate::traversal::pixel::PixelTraversal;
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
        Self { high_res: map, low_res }
    }

    pub fn map(&self) -> &Map<f32> {
        &self.high_res
    }

    pub fn is_line_free<T: Float>(&self, ray: Ray3<T>) -> bool {
        let mut scaled_ray = ray.as_ray_2();
        let scale = T::from(1.0 / 16.0).unwrap();
        scaled_ray.start_x = scaled_ray.start_x * scale;
        scaled_ray.start_y = scaled_ray.start_y * scale;
        scaled_ray.diff_x = scaled_ray.diff_x * scale;
        scaled_ray.diff_y = scaled_ray.diff_y * scale;

        let mut low_res_pixel_traversal = PixelTraversal::new(scaled_ray);

        let definitely_free = if ray.diff_z >= T::zero() {
            low_res_pixel_traversal.all(|segment| {
                self.low_res
                    .get(segment.pixel_x as usize, segment.pixel_y as usize)
                    < (ray.start_z + segment.start_t * ray.diff_z)
                        .to_f32()
                        .unwrap()
            })
        } else {
            low_res_pixel_traversal.all(|segment| {
                self.low_res
                    .get(segment.pixel_x as usize, segment.pixel_y as usize)
                    < (ray.start_z + segment.end_t * ray.diff_z).to_f32().unwrap()
            })
        };

        if definitely_free {
            return true;
        }

        let mut high_res_pixel_traversal = PixelTraversal::new(ray.as_ray_2());

        if ray.diff_z >= T::zero() {
            high_res_pixel_traversal.all(|segment| {
                self.high_res
                    .get(segment.pixel_x as usize, segment.pixel_y as usize)
                    < (ray.start_z + segment.start_t * ray.diff_z)
                        .to_f32()
                        .unwrap()
            })
        } else {
            high_res_pixel_traversal.all(|segment| {
                self.high_res
                    .get(segment.pixel_x as usize, segment.pixel_y as usize)
                    < (ray.start_z + segment.end_t * ray.diff_z).to_f32().unwrap()
            })
        }
    }
}
