use crate::errors::Result;
use crate::models::CrawlResult;
use csv::Writer;
use std::fs::File;
use tokio::sync::mpsc;

pub async fn export_results(
    mut result_rx: mpsc::Receiver<CrawlResult>,
    output_path: &str,
) -> Result<()> {
    let file = File::create(output_path)?;
    let mut writer = Writer::from_writer(file);

    // Note: Headers are automatically written by the first serialize() call
    let mut count = 0;
    while let Some(result) = result_rx.recv().await {
        writer.serialize(&result)?;
        // Flush periodically instead of every write
        count += 1;
        if count % 1000 == 0 {
            writer.flush()?;
        }
    }

    // Final flush
    writer.flush()?;
    Ok(())
}
