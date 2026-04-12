use std::collections::HashMap;
use indicatif::ProgressIterator;
use crate::ray::Ray2;
use crate::tiles::TileCoordinates;
use crate::transform::TileSpacePositionAcrossTiles;
use crate::traversal::pixel::PixelTraversal;

/// A segment of a ray that is entirely contained within one tile.
#[derive(Copy, Clone)]
pub struct TileRay {
    pub tile_coordinates: TileCoordinates,
    pub start_t: f64,
    pub end_t: f64,
    /// The sub-ray in pixel-space coordinates relative to `tile_coordinates`.
    pub ray: Ray2<f64>,
}

/// Decomposes `ray` into an iterator of [`TileRay`]s. `ray` is assumed to be in tile space
/// coordinates.
pub fn tile_rays(ray: Ray2<f64>) -> impl Iterator<Item = TileRay> {
    PixelTraversal::new(ray).map(move |tile_segment| {
        let tile_coordinates = TileCoordinates {
            x: tile_segment.pixel_x,
            y: tile_segment.pixel_y,
        };
        let sub_ray = ray.sub_ray(tile_segment.start_t, tile_segment.end_t);
        let sub_ray_start = TileSpacePositionAcrossTiles {
            x: sub_ray.start_x,
            y: sub_ray.start_y,
        };
        let sub_ray_end = TileSpacePositionAcrossTiles {
            x: sub_ray.end_x(),
            y: sub_ray.end_y(),
        };
        let sub_ray_start_position = sub_ray_start.position_in(tile_coordinates);
        let sub_ray_end_position = sub_ray_end.position_in(tile_coordinates);
        let ray_in_tile = Ray2 {
            start_x: sub_ray_start_position.x,
            start_y: sub_ray_start_position.y,
            diff_x: sub_ray_end_position.x - sub_ray_start_position.x,
            diff_y: sub_ray_end_position.y - sub_ray_start_position.y,
        };

        TileRay {
            tile_coordinates,
            start_t: tile_segment.start_t,
            end_t: tile_segment.end_t,
            ray: ray_in_tile,
        }
    })
}

/// Decomposes `rays` into [`TileRay`]s and collects these in a `HashMap`, grouped by their tile
/// coordinates.
pub fn tile_rays_by_tile(
    rays: impl ExactSizeIterator<Item = Ray2<f64>>,
) -> HashMap<TileCoordinates, Vec<(TileRay, usize)>> {
    let mut tile_rays_by_tile: HashMap<TileCoordinates, Vec<(TileRay, usize)>> = HashMap::new();
    for (ray_index, ray) in rays.enumerate().progress() {
        for tile_ray in tile_rays(ray) {
            tile_rays_by_tile
                .entry(tile_ray.tile_coordinates)
                .or_default()
                .push((tile_ray, ray_index));
        }
    }
    tile_rays_by_tile
}
