
use std::sync::Arc;

mod config;
mod workers;
mod status;
mod retry;
mod tiles;
mod utils;
mod db;


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config = config::Config::load("config.yaml").unwrap();

    let file_appender = tracing_appender::rolling::hourly("logs", "maps_locations.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .json()
        .init();

    let (tx_fetcher, rx_fetcher) = async_channel::bounded(1000);
    let (tx_out, rx_out) = async_channel::bounded(1000);

    let program_status = Arc::new(status::ProgramStatus::new());
    let client = utils::initialize_client()?;

    let channel_info = Arc::new(status::ChannelInfo {
        tx_fetcher: tx_fetcher.clone(),
        tx_out: tx_out.clone(),
    });

    let status_clone = program_status.clone();
    tokio::spawn(async move {
        status::run_status_logger(status_clone, channel_info).await;
    });

    let mut output_handles = vec![];

    if config.options.output == "database" {
        let session = db::create_db_session(&config.database.uri, &config.database.keyspace).await;
        for _ in 0..config.database.workers {
            let session = Arc::clone(&session);
            let rx = rx_out.clone();
            let output_handle = tokio::spawn(async move {
                db::handle_db_insertions(session, rx).await
            });
            output_handles.push(output_handle);
        }
    } else if config.options.output == "file" {
        let output_handle = tokio::spawn(workers::file_writer(rx_out.clone(), "output.csv"));
        output_handles.push(output_handle)
    } else {
        panic!("no valid output specified! choose either `file` or `database`");
    }
    

    // spawn fetchers
    let mut fetcher_handles = vec![];
    for _ in 0..config.options.fetchers {
        let fetcher_rx = rx_fetcher.clone();
        let fetcher_tx_out = tx_out.clone();
        let fetcher_status = Arc::clone(&program_status);
        let fetcher_client = client.clone();
        let handle = tokio::spawn(async move {
            workers::fetcher(fetcher_client, config.options.max_retries, fetcher_rx, fetcher_tx_out, fetcher_status).await
        });
        fetcher_handles.push(handle);
    }

    // Start tile generation and sending
    tiles::generate_and_send_tiles(tx_fetcher.clone(), &config.tile_generation).await?;

    tx_fetcher.close();

    // Wait for fetchers to complete
    for handle in fetcher_handles {
        handle.await??;
    }

    tx_out.close();
    
    for output_handle in output_handles {
        output_handle.await??;
    } 

    Ok(())
}