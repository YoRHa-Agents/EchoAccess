#[tokio::main]
async fn main() {
    if let Err(e) = echoax_tui::run().await {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
