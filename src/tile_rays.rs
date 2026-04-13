use crate::ray::Ray2;
use crate::tiles::TileCoordinates;
use crate::transform::TileSpacePositionAcrossTiles;
use rayon::prelude::{IndexedParallelIterator, ParallelIterator};

/// A segment of a ray that is entirely contained within one tile.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TileRay {
    pub start_t: f64,
    pub end_t: f64,
    /// The sub-ray in pixel-space coordinates relative to the tile.
    pub ray: Ray2<f64>,
}

pub fn tile_rays_for_tile(
    tile: TileCoordinates,
    rays: impl IndexedParallelIterator<Item = Ray2<f64>>,
) -> impl ParallelIterator<Item = (TileRay, usize)> {
    rays.enumerate().filter_map(move |(ray_index, ray)| {
        tile_ray_in_tile(tile, ray).map(|tile_ray| (tile_ray, ray_index))
    })
}

fn tile_ray_in_tile(tile: TileCoordinates, ray: Ray2<f64>) -> Option<TileRay> {
    let start = TileSpacePositionAcrossTiles {
        x: ray.start_x,
        y: ray.start_y,
    };
    let end = TileSpacePositionAcrossTiles {
        x: ray.end_x(),
        y: ray.end_y(),
    };

    let mut start_t: f64 = 0.0;
    let mut end_t: f64 = 1.0;

    let t_delta_x = ray.diff_x.recip().abs();
    let t_delta_y = ray.diff_y.recip().abs();

    if start.x < tile.x as f64 {
        let dist_x = tile.x as f64 - start.x;
        start_t = start_t.max(t_delta_x * dist_x);
    } else if start.x > tile.x as f64 + 1.0 {
        let dist_x = start.x - (tile.x as f64 + 1.0);
        start_t = start_t.max(t_delta_x * dist_x)
    }

    if start.y < tile.y as f64 {
        let dist_y = tile.y as f64 - start.y;
        start_t = start_t.max(t_delta_y * dist_y);
    } else if start.y > tile.y as f64 + 1.0 {
        let dist_y = start.y - (tile.y as f64 + 1.0);
        start_t = start_t.max(t_delta_y * dist_y);
    }

    if end.x < tile.x as f64 {
        let dist_x = tile.x as f64 - end.x;
        end_t = end_t.min(1.0 - t_delta_x * dist_x);
    } else if end.x > tile.x as f64 + 1.0 {
        let dist_x = end.x - (tile.x as f64 + 1.0);
        end_t = end_t.min(1.0 - t_delta_x * dist_x)
    }

    if end.y < tile.y as f64 {
        let dist_y = tile.y as f64 - end.y;
        end_t = end_t.min(1.0 - t_delta_y * dist_y);
    } else if end.y > tile.y as f64 + 1.0 {
        let dist_y = end.y - (tile.y as f64 + 1.0);
        end_t = end_t.min(1.0 - t_delta_y * dist_y);
    }

    if start_t + 1e-9 >= end_t {
        return None;
    }

    let sub_ray = ray.sub_ray(start_t, end_t);
    let sub_ray_start = TileSpacePositionAcrossTiles {
        x: sub_ray.start_x,
        y: sub_ray.start_y,
    };
    let sub_ray_end = TileSpacePositionAcrossTiles {
        x: sub_ray.end_x(),
        y: sub_ray.end_y(),
    };
    let sub_ray_start_position = sub_ray_start.position_in(tile);
    let sub_ray_end_position = sub_ray_end.position_in(tile);
    let ray_in_tile = Ray2 {
        start_x: sub_ray_start_position.x,
        start_y: sub_ray_start_position.y,
        diff_x: sub_ray_end_position.x - sub_ray_start_position.x,
        diff_y: sub_ray_end_position.y - sub_ray_start_position.y,
    };

    Some(TileRay {
        start_t,
        end_t,
        ray: ray_in_tile,
    })
}

#[cfg(test)]
#[test]
fn test_tile_ray_in_tile() {
    let ray = Ray2 {
        start_x: 0.0,
        start_y: 0.0,
        diff_x: 1.5,
        diff_y: 1.5,
    };
    assert!(tile_ray_in_tile(TileCoordinates { x: 0, y: 0 }, ray).is_some());
    assert!(tile_ray_in_tile(TileCoordinates { x: 1, y: 1 }, ray).is_some());
    assert_eq!(tile_ray_in_tile(TileCoordinates { x: 0, y: 1 }, ray), None);
    assert_eq!(tile_ray_in_tile(TileCoordinates { x: 1, y: 0 }, ray), None);
    assert_eq!(tile_ray_in_tile(TileCoordinates { x: 2, y: 2 }, ray), None);
}
