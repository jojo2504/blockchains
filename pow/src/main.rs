use std::error::Error;

use pow::start_node;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    start_node().await?;

    Ok(())
}