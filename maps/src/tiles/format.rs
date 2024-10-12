use crate::tiles::types::Tile;

pub fn format_tiles(tiles: &Vec<Tile>) -> String {
    let mut result = String::from("");

    for tile in tiles {
        result.push_str(&format!("!1m4!1m3!1i{}!2i{}!3i{}", tile.zoom, tile.x, tile.y));
    }

    result.push_str("!2m3!1e0!2sm!3i702451461!3m12!2sen-US!3sUS!5e18!12m4!1e68!2m2!1sset!2sRoadmap!12m3!1e37!2m1!1ssmartmaps!4e3!12m1!5b1");

    result
}