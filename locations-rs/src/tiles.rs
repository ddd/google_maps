
use async_channel::Sender;
use crate::config;

const BATCH_SIZE: usize = 160;

pub fn create_tile_iterator(
    min_x: usize,
    max_x: usize,
    min_y: usize,
    max_y: usize,
    reverse: bool
) -> impl Iterator<Item = (usize, usize)> {
    let width = max_x - min_x + 1;
    let height = max_y - min_y + 1;

    (0..width).flat_map(move |x| {
        let current_x = if reverse {
            max_x - x
        } else {
            min_x + x
        };
        (0..height).map(move |y| (current_x, min_y + y))
    })
}

pub async fn generate_and_send_tiles(
    tx_fetcher: Sender<Vec<maps::tiles::Tile>>,
    tile_config: &config::TileGenerationConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut batch = Vec::with_capacity(BATCH_SIZE);

    for (x, y) in create_tile_iterator(
        tile_config.min_x,
        tile_config.max_x,
        tile_config.min_y,
        tile_config.max_y,
        tile_config.reverse
    ) {
        batch.push(maps::tiles::Tile { x, y, zoom: tile_config.zoom });

        if batch.len() == BATCH_SIZE {
            tx_fetcher.send(batch).await?;
            batch = Vec::with_capacity(BATCH_SIZE);
        }
    }

    // Send any remaining tiles
    if !batch.is_empty() {
        tx_fetcher.send(batch).await?;
    }

    println!("dropped tx fetcher");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_tile_iterator() {
        // Test case 1: Small forward range
        let iter = create_tile_iterator(0, 2, 0, 2, false);
        let result: Vec<(usize, usize)> = iter.collect();
        assert_eq!(result, vec![
            (0, 0), (0, 1), (0, 2),
            (1, 0), (1, 1), (1, 2),
            (2, 0), (2, 1), (2, 2)
        ]);

        // Test case 2: Small reverse range
        let iter = create_tile_iterator(0, 2, 0, 2, true);
        let result: Vec<(usize, usize)> = iter.collect();
        assert_eq!(result, vec![
            (2, 0), (2, 1), (2, 2),
            (1, 0), (1, 1), (1, 2),
            (0, 0), (0, 1), (0, 2)
        ]);

        // Test case 3: Non-zero min values
        let iter = create_tile_iterator(10, 12, 20, 22, false);
        let result: Vec<(usize, usize)> = iter.collect();
        assert_eq!(result, vec![
            (10, 20), (10, 21), (10, 22),
            (11, 20), (11, 21), (11, 22),
            (12, 20), (12, 21), (12, 22)
        ]);

        // Test case 4: Non-zero min values, reverse
        let iter = create_tile_iterator(10, 12, 20, 22, true);
        let result: Vec<(usize, usize)> = iter.collect();
        assert_eq!(result, vec![
            (12, 20), (12, 21), (12, 22),
            (11, 20), (11, 21), (11, 22),
            (10, 20), (10, 21), (10, 22)
        ]);

    }
}