use initialization::find_and_connect;

pub mod communication;
pub mod initialization;
pub mod minecraft;

#[tokio::main]
async fn main() {
    let mut mc = find_and_connect().await;
    let result = mc
        .toast("Test", "Hello from Rust!")
        .await
        .unwrap()
        .get_result()
        .await;
    println!("Result: {:?}", result);
}
