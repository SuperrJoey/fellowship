use axum::http::StatusCode;
use axum::response::Json;
use axum::extract::Json as ExtractJson;
use axum::{
    routing::post,
    Router
};

use serde::{Deserialize, Serialize};

use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::pubkey::Pubkey;

use spl_token::instruction as token_instruction;

// Base64 for encoding instruction data  
use base64::prelude::*;

use std::str::FromStr;

use std::net::SocketAddr;

#[derive(Serialize)]
struct SuccessResponse {
    success: bool,
    data: KeypairData,
}

#[derive(Deserialize)]
struct CreateTokenRequest {
    #[serde(rename = "mintAuthority")]
    mint_authority: String,
    mint: String,
    decimals: u8,
}

#[derive(Serialize)]
struct CreateTokenResponse {
    success: bool,
    data: TokenInstructionData,
}

#[derive(Serialize)]
struct TokenInstructionData {
    program_id: String,
    accounts: Vec<AccountInfo>,
    instruction_data: String,
}

#[derive(Serialize)]
struct AccountInfo {
    pubkey: String,
    is_signer: bool,
    is_writable: bool,
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

// Handler function for POST /token/create
// This function creates an SPL token initialize mint instruction
async fn create_token(
    ExtractJson(request): ExtractJson<CreateTokenRequest>
) -> Result<Json<CreateTokenResponse>, (StatusCode, Json<ErrorResponse>)> {
    
    // Step 1: Parse the public keys from base58 strings
    let mint_authority = match Pubkey::from_str(&request.mint_authority) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid mint authority public key".to_string(),
            })
        ))
    };
    
    let mint = match Pubkey::from_str(&request.mint) {
        Ok(pubkey) => pubkey,
        Err(_) => return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Invalid mint public key".to_string(),
            })
        ))
    };
    
    // Step 2: Create the SPL Token initialize mint instruction
    let instruction = match token_instruction::initialize_mint(
        &spl_token::id(),           // SPL Token program ID
        &mint,                      // Mint account
        &mint_authority,            // Mint authority
        None,                       // Freeze authority (optional)
        request.decimals,           // Number of decimals
    ) {
        Ok(instruction) => instruction,
        Err(_) => return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                success: false,
                error: "Failed to create initialize mint instruction".to_string(),
            })
        ))
    };
    
    // Step 3: Convert instruction data to base64
    let instruction_data = BASE64_STANDARD.encode(&instruction.data);
    
    // Step 4: Convert accounts to our format
    let accounts: Vec<AccountInfo> = instruction.accounts
        .iter()
        .map(|account| AccountInfo {
            pubkey: account.pubkey.to_string(),
            is_signer: account.is_signer,
            is_writable: account.is_writable,
        })
        .collect();
    
    // Step 5: Create and return the response
    let response = CreateTokenResponse {
        success: true,
        data: TokenInstructionData {
            program_id: instruction.program_id.to_string(),
            accounts,
            instruction_data,
        },
    };
    
    Ok(Json(response))
}


#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/keypair", post(generate_keypair))
        .route("/token/create", post(create_token));

        let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
        println!("Running on http://{}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        axum::serve(listener, app).await.unwrap();
}