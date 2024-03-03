use cli::bootstrap;

#[tokio::main]
async fn main() {
    bootstrap::all::run().await
}
