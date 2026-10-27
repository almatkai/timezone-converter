#[tokio::main(worker_threads = 2)]
async fn main() {
    if let Err(e) = tezbar_extension_sdk::serve(timezone_converter_v2::Adapter::default()).await {
        eprintln!("Timezone Converter: {e}");
        std::process::exit(1);
    }
}
