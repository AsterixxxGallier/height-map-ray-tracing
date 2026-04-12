use crate::map::Map;
use crate::ray::Ray3;
use crate::traversal::pixel::PixelTraversal;
use num_traits::Float;

pub fn intersection_t<T: Float>(map: &Map<f32>, ray: Ray3<T>) -> Option<T> {
    let mut pixel_traversal = PixelTraversal::new(ray.as_ray_2());

    if ray.diff_z >= T::zero() {
        pixel_traversal
            .find(|segment| {
                map.get(segment.pixel_x as usize, segment.pixel_y as usize)
                    >= (ray.start_z + segment.start_t * ray.diff_z)
                        .to_f32()
                        .unwrap()
            })
            .map(|segment| segment.start_t)
    } else {
        pixel_traversal
            .find(|segment| {
                map.get(segment.pixel_x as usize, segment.pixel_y as usize)
                    >= (ray.start_z + segment.end_t * ray.diff_z).to_f32().unwrap()
            })
            .map(|segment| segment.start_t)
    }
}
