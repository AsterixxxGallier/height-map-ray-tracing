use crate::intersection::intersection_t;
use crate::map::Map;
use crate::ray::Ray3;
use num_traits::Float;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

const CHUNK_SIZES: [usize; 2] = [100, 8];

const _: () = {
    let mut i = 0;
    while i < CHUNK_SIZES.len() {
        assert!(2000usize.is_multiple_of(CHUNK_SIZES[i]));
        i += 1;
    }
    while i < CHUNK_SIZES.len() - 1 {
        assert!(CHUNK_SIZES[i] > CHUNK_SIZES[i + 1]);
        i += 1;
    }
};

/// Optimization structure containing height map data of a single tile. A tile is 2000 by 2000
/// pixels, corresponding to 1000 by 1000 meters.
pub struct Tile {
    map: Map<f32>,
    min: [Map<f32>; CHUNK_SIZES.len()],
    max: [Map<f32>; CHUNK_SIZES.len()],
}

fn max(map: &Map<f32>, chunk_size: usize) -> impl Fn(usize, usize) -> f32 + use<'_> {
    move |x_index, y_index| {
        let mut max = f32::NEG_INFINITY;
        for i in 0..chunk_size {
            for j in 0..chunk_size {
                max = max.max(map.get(x_index * chunk_size + i, y_index * chunk_size + j));
            }
        }
        max
    }
}

fn min(map: &Map<f32>, chunk_size: usize) -> impl Fn(usize, usize) -> f32 + use<'_> {
    move |x_index, y_index| {
        let mut min = f32::NEG_INFINITY;
        for i in 0..chunk_size {
            for j in 0..chunk_size {
                min = min.min(map.get(x_index * chunk_size + i, y_index * chunk_size + j));
            }
        }
        min
    }
}

impl Tile {
    pub fn new(map: Map<f32>) -> Self {
        assert_eq!(map.x_len(), 2000);
        assert_eq!(map.y_len(), 2000);
        let min = CHUNK_SIZES.map(|chunk_size| {
            Map::from_fn(
                2000 / chunk_size,
                2000 / chunk_size,
                min(&map, chunk_size),
            )
        });
        let max = CHUNK_SIZES.map(|chunk_size| {
            Map::from_fn(
                2000 / chunk_size,
                2000 / chunk_size,
                max(&map, chunk_size),
            )
        });
        Self { map, min, max }
    }

    pub fn regenerate(&mut self, change_map: impl FnOnce(&mut Map<f32>)) {
        change_map(&mut self.map);

        for index in 0..CHUNK_SIZES.len() {
            let chunk_size = CHUNK_SIZES[index];
            self.min[index].regenerate_from_fn(min(&self.map, chunk_size));
            self.max[index].regenerate_from_fn(max(&self.map, chunk_size));
        }
    }

    pub fn map(&self) -> &Map<f32> {
        &self.map
    }

    pub fn save_as_images(&self, white_value: f32, directory: impl AsRef<Path>) {
        fs::create_dir_all(directory.as_ref()).unwrap();

        let path = directory.as_ref().join("original.png");
        self.map.save_as_image(white_value, path);

        for (index, chunk_size) in CHUNK_SIZES.into_iter().enumerate() {
            let file_name = format!("min {chunk_size}.png");
            let path = directory.as_ref().join(file_name);
            self.min[index].save_as_image(white_value, path);

            let file_name = format!("max {chunk_size}.png");
            let path = directory.as_ref().join(file_name);
            self.max[index].save_as_image(white_value, path);
        }
    }

    pub fn is_line_free<T: Float + Debug>(&self, mut ray: Ray3<T>) -> bool {
        for (index, chunk_size) in CHUNK_SIZES.into_iter().enumerate() {
            let min = &self.min[index];
            let max = &self.max[index];

            let scaled_ray = ray.scale_x_y(T::from(1.0 / chunk_size as f64).unwrap());

            if intersection_t(min, scaled_ray).is_some() {
                return false;
            }

            if let Some(intersection_t) = intersection_t(max, scaled_ray) {
                ray = ray.sub_ray(intersection_t, T::one());
                // correct precision errors in sub_ray calculation
                if ray.end_x() < T::zero() {
                    ray.diff_x = -ray.start_x;
                }
                if ray.end_y() < T::zero() {
                    ray.diff_y = -ray.start_y;
                }
                let max = T::from(2000.0).unwrap();
                if ray.end_x() > max {
                    ray.diff_x = max - ray.start_x;
                }
                if ray.end_y() > max {
                    ray.diff_y = max - ray.start_y;
                }
            } else {
                return true;
            }
        }

        intersection_t(&self.map, ray).is_none()
    }
}
