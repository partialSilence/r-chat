use crate::chat_api::process;

mod chat_api;
#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    match process().await {
        Ok(_) => (),
        Err(err) => log::error!("Error in process: {}", err),
    }
}
