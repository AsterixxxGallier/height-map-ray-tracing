use crate::tiles::TileCoordinates;

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PositionInTile {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct ModelSpacePosition {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PixelSpacePositionAcrossTiles {
    pub x: f64,
    pub y: f64,
}

impl PixelSpacePositionAcrossTiles {
    pub fn combine(tile_coordinates: TileCoordinates, position_in_tile: PositionInTile) -> Self {
        Self {
            x: tile_coordinates.x as f64 * 2000.0 + position_in_tile.x,
            y: tile_coordinates.y as f64 * 2000.0 - position_in_tile.y,
        }
    }

    pub fn split(self) -> (TileCoordinates, PositionInTile) {
        let tile_x = (self.x / 2000.0).floor();
        let tile_y = (self.y / 2000.0).ceil();
        let pixel_x = self.x - tile_x * 2000.0;
        let pixel_y = tile_y * 2000.0 - self.y;

        let tile_coordinates = TileCoordinates {
            x: tile_x as i32,
            y: tile_y as i32,
        };
        let position_in_tile = PositionInTile {
            x: pixel_x,
            y: pixel_y,
        };

        (tile_coordinates, position_in_tile)
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct TileSpacePositionAcrossTiles {
    pub x: f64,
    pub y: f64,
}

impl TileSpacePositionAcrossTiles {
    pub fn combine(tile_coordinates: TileCoordinates, position_in_tile: PositionInTile) -> Self {
        Self {
            x: tile_coordinates.x as f64 + position_in_tile.x / 2000.0,
            y: tile_coordinates.y as f64 - position_in_tile.y / 2000.0,
        }
    }

    pub fn split(self) -> (TileCoordinates, PositionInTile) {
        let tile_x = self.x.floor();
        let tile_y = self.y.ceil();
        let pixel_x = (self.x - tile_x) * 2000.0;
        let pixel_y = (tile_y - self.y) * 2000.0;

        let tile_coordinates = TileCoordinates {
            x: tile_x as i32,
            y: tile_y as i32,
        };
        let position_in_tile = PositionInTile {
            x: pixel_x,
            y: pixel_y,
        };

        (tile_coordinates, position_in_tile)
    }
}

impl From<PixelSpacePositionAcrossTiles> for TileSpacePositionAcrossTiles {
    fn from(value: PixelSpacePositionAcrossTiles) -> Self {
        Self {
            x: value.x / 2000.0,
            y: value.y / 2000.0,
        }
    }
}

impl From<TileSpacePositionAcrossTiles> for PixelSpacePositionAcrossTiles {
    fn from(value: TileSpacePositionAcrossTiles) -> Self {
        Self {
            x: value.x * 2000.0,
            y: value.y * 2000.0,
        }
    }
}

impl From<PixelSpacePositionAcrossTiles> for ModelSpacePosition {
    fn from(value: PixelSpacePositionAcrossTiles) -> Self {
        let (tile_coordinates, position_within_tile) = value.split();
        let x_origin = tile_coordinates.x as f64 * 1000.0 - 0.25;
        let y_origin = tile_coordinates.y as f64 * 1000.0;
        let x_offset = position_within_tile.x / 2.0;
        let y_offset = -position_within_tile.y / 2.0;

        ModelSpacePosition {
            x: x_origin + x_offset,
            y: y_origin + y_offset,
        }
    }
}

impl From<ModelSpacePosition> for PixelSpacePositionAcrossTiles {
    fn from(value: ModelSpacePosition) -> Self {
        PixelSpacePositionAcrossTiles {
            x: (value.x + 0.25) * 2.0,
            y: value.y * 2.0,
        }
    }
}

#[cfg(test)]
#[test]
fn test() {
    let position_across_tiles = PixelSpacePositionAcrossTiles {
        x: 18925.1985,
        y: 26935.5237,
    };
    let (tile_coordinates, position_in_tile) = position_across_tiles.split();
    assert_eq!(
        position_across_tiles,
        PixelSpacePositionAcrossTiles::combine(tile_coordinates, position_in_tile),
    );

    assert_eq!(
        position_across_tiles,
        PixelSpacePositionAcrossTiles::from(ModelSpacePosition::from(position_across_tiles)),
    );
}
