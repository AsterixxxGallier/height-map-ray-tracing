use crate::intersection::intersection_t;
use crate::map::Map;
use crate::ray::Ray3;
use num_traits::Float;
use std::fmt::Debug;
use std::fs;
use std::path::Path;

const CHUNK_SIZES: [usize; 3] = [100, 8, 1];

const _: () = {
    let mut i = 0;
    while i < CHUNK_SIZES.len() {
        assert!(2000usize.is_multiple_of(CHUNK_SIZES[i]));
        i += 1;
    }
    assert!(CHUNK_SIZES[CHUNK_SIZES.len() - 1] == 1);
};

/// Optimization structure containing height map data of a single tile. A tile is 2000 by 2000
/// pixels, corresponding to 1000 by 1000 meters.
pub struct Tile {
    maps: [Map<f32>; CHUNK_SIZES.len()],
}

fn downsize_map(map: &Map<f32>, chunk_size: usize) -> impl Fn(usize, usize) -> f32 + use<'_> {
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

impl Tile {
    pub fn new(map: Map<f32>) -> Self {
        assert_eq!(map.x_len(), 2000);
        assert_eq!(map.y_len(), 2000);
        let maps = CHUNK_SIZES.map(|chunk_size| {
            Map::from_fn(2000 / chunk_size, 2000 / chunk_size, downsize_map(&map, chunk_size))
        });
        Self { maps }
    }

    pub fn regenerate(&mut self, change_map: impl FnOnce(&mut Map<f32>)) {
        let (original_map, other_maps) = self.maps.split_last_mut().unwrap();
        change_map(original_map);
        for index in 0..(CHUNK_SIZES.len() - 1) {
            let chunk_size = CHUNK_SIZES[index];
            let map = &mut other_maps[index];
            map.regenerate_from_fn(downsize_map(original_map, chunk_size));
        }
    }

    pub fn map(&self) -> &Map<f32> {
        self.maps.last().unwrap()
    }

    pub fn save_as_images(&self, white_value: f32, directory: impl AsRef<Path>) {
        fs::create_dir_all(directory.as_ref()).unwrap();
        for (chunk_size, map) in CHUNK_SIZES.into_iter().zip(&self.maps) {
            let file_name = format!("chunk size {chunk_size}.png");
            let path = directory.as_ref().join(file_name);
            map.save_as_image(white_value, path);
        }
    }

    pub fn is_line_free<T: Float + Debug>(&self, mut ray: Ray3<T>) -> bool {
        for (chunk_size, map) in CHUNK_SIZES.into_iter().zip(&self.maps) {
            let scaled_ray = ray.scale_x_y(T::from(1.0 / chunk_size as f64).unwrap());
            if let Some(intersection_t) = intersection_t(map, scaled_ray) {
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
        false
    }
}
