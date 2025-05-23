use mc_connect::initialization::find_and_connect;

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
