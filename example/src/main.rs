use std::env;

use mc_connect::initialization::find_and_connect;

#[allow(dead_code)]
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
    mc.run(
        env::current_exe()
            .unwrap()
            .parent()
            .unwrap()
            .join("libexample.so"),
        "in_mc".to_string(),
    )
    .await
    .unwrap()
    .get_result()
    .await;
}
