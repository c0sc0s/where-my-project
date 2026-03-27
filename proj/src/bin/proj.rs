use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    proj::cli::run().await
}
