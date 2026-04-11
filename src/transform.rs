use crate::tiles::TileCoordinates;

pub fn pixel_to_model(tile_coordinates: TileCoordinates, x: f64, y: f64) -> (f64, f64) {
    let x_origin = tile_coordinates.x as f64 * 1000.0 - 0.25;
    let y_origin = tile_coordinates.y as f64 * 1000.0;
    let x_offset = x / 2.0;
    let y_offset = -y / 2.0;
    (x_origin + x_offset, y_origin + y_offset)
}

pub fn model_to_pixel(model_x: f64, model_y: f64) -> (TileCoordinates, f64, f64) {
    let model_x = (model_x + 0.25) * 2.0;
    let tile_x = (model_x / 2000.0).floor();
    let pixel_x = model_x - tile_x * 2000.0;

    let model_y = model_y * 2.0;
    let tile_y = (model_y / 2000.0).ceil();
    let pixel_y = tile_y * 2000.0 - model_y;

    let tile = TileCoordinates {
        x: tile_x as i32,
        y: tile_y as i32,
    };

    (tile, pixel_x, pixel_y)
}
