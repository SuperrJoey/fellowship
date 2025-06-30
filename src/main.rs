use axum::http::StatusCode;
use axum::response::Json;
use axum::{
    routing::post,
    Router
};

use serde::Serialize;

use solana_sdk::signature::{Keypair, Signer};
use std::net::SocketAddr;

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
    data: KeypairData,
}

#[derive(Serialize)]
struct KeypairData {
    pubkey: String,
    secret: String,
}

#[derive(Serialize)]
struct ErrorResponse {
    success: bool,
    error: String,
}

async fn generate_keypair() -> Result<Json<SuccessResponse>, (StatusCode, Json<ErrorResponse>)> {
    match std::panic::catch_unwind(|| {
        let keypair = Keypair::new();

        let pubkey = bs58::encode(keypair.pubkey().as_ref()).into_string();
        let secret = bs58::encode(&keypair.to_bytes()).into_string();

        SuccessResponse {
            success: true,
            data: KeypairData { pubkey, secret },
        }
    }) {
        Ok(response) => Ok(Json(response)),
        Err(_) => Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Failed to generate keypair".to_string(),
            }),
        )),
    }
}


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair));

        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        println!("Running on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
}