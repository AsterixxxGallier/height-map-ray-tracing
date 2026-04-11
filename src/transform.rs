use crate::tiles::TileCoordinates;

pub fn pixel_to_model(tile_coordinates: TileCoordinates, x: f64, y: f64) -> (f64, f64) {
    let x_origin = tile_coordinates.x as f64 * 1000.0 - 0.25;
    let y_origin = tile_coordinates.y as f64 * 1000.0;
    let x_offset = x / 2.0;
    let y_offset = -y / 2.0;
    (x_origin + x_offset, y_origin + y_offset)
}

pub fn model_to_pixel(model_x: f64, model_y: f64) -> (TileCoordinates, f64, f64) {
    /*

    model_x = x_origin + x_offset
    x_origin = tile_x * 1000 - 0.25
    x_offset = pixel_x / 2.0

    model_x = x_origin + x_offset - 0.25
    x_origin = tile_x * 1000
    x_offset = pixel_x / 2.0

    model_x' := model_x + 0.25

    model_x' = x_origin + x_offset
    x_origin = tile_x * 1000
    x_offset = pixel_x / 2.0

    model_x'' := model_x' * 2.0
    x_origin' := x_origin * 2.0
    x_offset' := x_offset * 2.0

    model_x'' = x_origin' + x_offset'
    x_origin' = tile_x * 2000
    x_offset' = pixel_x

    tile_x = floor(model_x'' / 2000)
    pixel_x = model_x'' % 2000

    x_origin' = floor(model_x'' / 2000) * 2000
    x_offset' = model_x'' % 2000
    model_x'' = floor(model_x'' / 2000) * 2000 + model_x'' % 2000

    model_y = y_origin + y_offset
    y_origin = tile_y * 1000.0
    y_offset = pixel_y / -2.0

    model_y' := model_y * 2.0
    y_origin' := y_origin * 2.0
    y_offset' := y_offset * 2.0

    model_y' = y_origin' + y_offset'
    y_origin' = tile_y * 2000.0
    y_offset' = -pixel_y

    tile_y := ceil(model_y' / 2000.0)
    pixel_y :=

    model_y' = ceil(model_y' / 2000.0) * 2000.0 - pixel_y
    pixel_y = ceil(model_y' / 2000.0) * 2000.0 - model_y'

     */

    let model_x_prime = (model_x + 0.25) * 2.0;
    let tile_x = (model_x_prime / 2000.0).floor();
    let pixel_x = model_x_prime - tile_x * 2000.0;

    let model_y_prime = model_y * 2.0;
    let tile_y = (model_y_prime / 2000.0).ceil();
    let pixel_y = tile_y * 2000.0 - model_y_prime;

    let tile = TileCoordinates {
        x: tile_x as i32,
        y: tile_y as i32,
    };
    
    (tile, pixel_x, pixel_y)
}

#[derive(Debug, PartialEq)]
pub struct Coord {
    /// Typically, `x` is the horizontal position, or longitude for geographic coordinates,
    /// but its interpretation can vary across coordinate systems.
    pub x: f64,
    /// Typically, `y` is the vertical position, or latitude for geographic coordinates,
    /// but its interpretation can vary across coordinate systems.
    pub y: f64,
}

#[derive(Debug, PartialEq)]
pub enum CoordinateTransform {
    AffineTransform {
        transform: [f64; 6],
        inverse_transform: [f64; 6],
    },
    TiePointAndPixelScale {
        raster_point: Coord,
        model_point: Coord,
        pixel_scale: Coord,
    },
}

impl CoordinateTransform {
    pub fn from_transformation_matrix(transformation_matrix: [f64; 16]) -> Self {
        let transform = [
            transformation_matrix[0],
            transformation_matrix[1],
            transformation_matrix[3],
            transformation_matrix[4],
            transformation_matrix[5],
            transformation_matrix[7],
        ];

        let det = transform[0] * transform[4] - transform[1] * transform[3];
        if det.abs() < 0.000000000000001 {
            panic!()
        }

        let inverse_transform = [
            transform[4] / det,
            -transform[1] / det,
            (transform[1] * transform[5] - transform[2] * transform[4]) / det,
            -transform[3] / det,
            transform[0] / det,
            (-transform[0] * transform[5] + transform[2] * transform[3]) / det,
        ];

        CoordinateTransform::AffineTransform {
            transform,
            inverse_transform,
        }
    }

    pub(super) fn transform_by_affine_transform(transform: &[f64; 6], coord: &Coord) -> Coord {
        Coord {
            x: coord.x * transform[0] + coord.y * transform[1] + transform[2],
            y: coord.x * transform[3] + coord.y * transform[4] + transform[5],
        }
    }

    pub(super) fn from_tie_point_and_pixel_scale(tie_points: &[f64], pixel_scale: &[f64]) -> Self {
        CoordinateTransform::TiePointAndPixelScale {
            raster_point: Coord {
                x: tie_points[0],
                y: tie_points[1],
            },
            model_point: Coord {
                x: tie_points[3],
                y: tie_points[4],
            },
            pixel_scale: Coord {
                x: pixel_scale[0],
                y: pixel_scale[1],
            },
        }
    }

    pub(super) fn transform_to_model_by_tie_point_and_pixel_scale(
        raster_point: &Coord,
        model_point: &Coord,
        pixel_scale: &Coord,
        coord: &Coord,
    ) -> Coord {
        Coord {
            x: (coord.x - raster_point.x) * pixel_scale.x + model_point.x,
            y: (coord.y - raster_point.y) * -pixel_scale.y + model_point.y,
        }
    }

    pub(super) fn transform_to_raster_by_tie_point_and_pixel_scale(
        raster_point: &Coord,
        model_point: &Coord,
        pixel_scale: &Coord,
        coord: &Coord,
    ) -> Coord {
        Coord {
            x: (coord.x - model_point.x) / pixel_scale.x + raster_point.x,
            y: (coord.y - model_point.y) / -pixel_scale.y + raster_point.y,
        }
    }

    pub(super) fn from_tag_data(
        pixel_scale_data: Option<Vec<f64>>,
        model_tie_points_data: Option<Vec<f64>>,
        model_transformation_data: Option<Vec<f64>>,
    ) -> Self {
        let pixel_scale = pixel_scale_data
            .map(|data| <[f64; 3]>::try_from(data).map_err(|_| panic!()))
            .transpose()
            .unwrap();
        let tie_points = model_tie_points_data.unwrap();
        let transformation_matrix = model_transformation_data
            .map(|data| <[f64; 16]>::try_from(data).map_err(|_| panic!()))
            .transpose()
            .unwrap();

        if let Some(transformation_matrix) = transformation_matrix {
            if pixel_scale.is_some() {
                panic!();
            }

            Self::from_transformation_matrix(transformation_matrix)
        } else {
            if tie_points.len() == 6 {
                let Some(pixel_scale) = pixel_scale else {
                    panic!();
                };

                Self::from_tie_point_and_pixel_scale(&tie_points, &pixel_scale)
            } else {
                panic!()
            }
        }
    }

    pub fn transform_to_model(&self, coord: &Coord) -> Coord {
        match self {
            CoordinateTransform::AffineTransform { transform, .. } => {
                Self::transform_by_affine_transform(transform, coord)
            }
            CoordinateTransform::TiePointAndPixelScale {
                raster_point,
                model_point,
                pixel_scale,
            } => Self::transform_to_model_by_tie_point_and_pixel_scale(
                raster_point,
                model_point,
                pixel_scale,
                coord,
            ),
        }
    }

    pub(super) fn transform_to_raster(&self, coord: &Coord) -> Coord {
        match self {
            CoordinateTransform::AffineTransform {
                inverse_transform, ..
            } => Self::transform_by_affine_transform(inverse_transform, coord),
            CoordinateTransform::TiePointAndPixelScale {
                raster_point,
                model_point,
                pixel_scale,
            } => Self::transform_to_raster_by_tie_point_and_pixel_scale(
                raster_point,
                model_point,
                pixel_scale,
                coord,
            ),
        }
    }
}
