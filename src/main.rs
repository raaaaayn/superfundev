use axum;
use axum::response::IntoResponse;
use axum::Json;
use axum::extract::State;
use dotenv::dotenv;
use serde::Deserialize;
use serde_json::json;
use solana_client::rpc_client::RpcClient;
use solana_sdk::program_pack::Pack;
use solana_sdk::signer::Signer;
use solana_sdk::signer::keypair::Keypair;
use solana_sdk::{system_instruction, transaction::Transaction};
use spl_token::{instruction as token_instruction, state::Mint};
use std::sync::Arc;

struct AppState {
    client: Arc<RpcClient>,
}

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

    let shared_state = Arc::new(AppState {
        client: Arc::new(RpcClient::new(config.rpc.to_string())),
    });

    let app = axum::Router::new()
        .route("/keypair", axum::routing::post(keypair))
        .route("/token/create", axum::routing::post(create_token))
        .with_state(shared_state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();

    println!("Hello, world!");
}

async fn keypair() -> impl IntoResponse {
    let keypair = Keypair::new();

    Json(json!({
        "success": true,
        "data": {
            "pubkey": keypair.try_pubkey().unwrap().to_string(),
            "secret": keypair.to_base58_string(),
        }
    }))
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    mint_authority: String,
    mint: String,
    decimals: i32,
}

async fn create_token(
    State(state): State<Arc<AppState>>,
    Json(body): Json<CreateTokenRequest>,
) -> impl IntoResponse {
    let payer = Keypair::new();
    let mint_authority = Keypair::new();
    let mint_keypair = Keypair::new();

    // Use Mint::LEN constant (82 bytes)
    let mint_rent = state
        .client
        .get_minimum_balance_for_rent_exemption(Mint::LEN)
        .unwrap();

    let create_mint_account_ix = system_instruction::create_account(
        &payer.pubkey(),
        &mint_keypair.pubkey(),
        mint_rent,
        Mint::LEN as u64, // Use Mint::LEN here
        &spl_token::id(),
    );

    let init_mint_ix = token_instruction::initialize_mint(
        &spl_token::id(),
        &mint_keypair.pubkey(),
        &mint_authority.pubkey(),
        Some(&mint_authority.pubkey()),
        9, // decimals
    )
    .unwrap();

    let recent_blockhash = state.client.get_latest_blockhash().unwrap();
    let transaction = Transaction::new_signed_with_payer(
        &[create_mint_account_ix, init_mint_ix],
        Some(&payer.pubkey()),
        &[&payer, &mint_keypair],
        recent_blockhash,
    );

    let signature = state
        .client
        .send_and_confirm_transaction(&transaction)
        .unwrap();
    println!("Token created! Signature: {}", signature);
    println!("Mint address: {}", mint_keypair.pubkey());

    Json(json!({}))
}
