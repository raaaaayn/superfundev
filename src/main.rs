use axum::{Router, routing::get};
use dotenv::dotenv;
use serde::Deserialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use std::str::FromStr;

#[derive(Debug, Deserialize)]
struct Config {
    rpc: String,
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = match envy::from_env::<Config>() {
        Ok(config) => config,
        Err(_) => {
            panic!("no rpc url provided")
        }
    };

    // build our application with a single route
    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    let client = RpcClient::new(config.rpc.to_string());

    // Replace with your desired account's public key (base58 string)
    let pubkey = Pubkey::from_str("9Paysbs5evoh9BiWiS77NNutMCG9koUK2xyAsJm89Rfh")
        .expect("Invalid public key");

    // Get SOL balance
    match client.get_balance(&pubkey) {
        Ok(balance) => println!("Balance: {} lamports", balance),
        Err(err) => eprintln!("Error: {}", err),
    }
    println!("Hello, world!");
}
