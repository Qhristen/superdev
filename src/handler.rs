use std::str::FromStr;

use actix_web::{HttpResponse, Responder, web};
use base64::engine::Engine;
use solana_sdk::{instruction::{AccountMeta, Instruction}, pubkey::Pubkey, signature::{Keypair, Signature}, signer::Signer, system_instruction::{self, transfer}};
use spl_token::instruction::{initialize_mint, mint_to};
use bs58;

use crate::{
    error::ApiError,
    response::{AccountMetaResponse, AccountMetaSimple, CreateTokenRequest, KeypairResponse, MintInstructionResponse, MintTokenRequest, SendSolRequest, SendSolResponse, SendTokenRequest, SendTokenResponse, SignMessageRequest, SignMessageResponse, SuccessResponse, TokenInstructionResponse, VerifyMessageRequest, VerifyMessageResponse},
};

pub async fn hello() -> impl Responder {
    "Hello, world!"
}

pub async fn generate_keypair() -> impl Responder {
    let keypair = Keypair::new();
    let pubkey = keypair.pubkey().to_string();
    let secret = keypair.to_base58_string();

    HttpResponse::Ok().json(SuccessResponse {
        success: true,
        data: KeypairResponse { pubkey, secret },
    })
}


pub async fn create_token(req: web::Json<CreateTokenRequest>) -> Result<impl Responder, ApiError> {
    let mint = Pubkey::from_str(&req.mint)
        .map_err(|_| ApiError::BadRequest("Invalid mint pubkey".into()))?;

    let authority = Pubkey::from_str(&req.mint_authority)
        .map_err(|_| ApiError::BadRequest("Invalid mintAuthority pubkey".into()))?;

    let ix = initialize_mint(&spl_token::id(), &mint, &authority, None, req.decimals)
        .map_err(|e| ApiError::InternalError(format!("Instruction creation failed: {}", e)))?;

    let accounts = ix
        .accounts
        .into_iter()
        .map(|meta: AccountMeta| AccountMetaResponse {
            pubkey: meta.pubkey.to_string(),
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        })
        .collect();

    let data = base64::engine::general_purpose::STANDARD.encode(&ix.data);

    Ok(HttpResponse::Ok().json(SuccessResponse {
        success: true,
        data: TokenInstructionResponse {
            program_id: ix.program_id.to_string(),
            accounts,
            instruction_data: data,
        },
    }))
}





pub async fn mint_token(
    body: web::Json<MintTokenRequest>,
) -> Result<impl Responder, ApiError> {
    let mint = Pubkey::from_str(&body.mint)
        .map_err(|_| ApiError::BadRequest("Invalid mint pubkey".into()))?;

    let destination = Pubkey::from_str(&body.destination)
        .map_err(|_| ApiError::BadRequest("Invalid destination pubkey".into()))?;

    let authority = Pubkey::from_str(&body.authority)
        .map_err(|_| ApiError::BadRequest("Invalid authority pubkey".into()))?;

    let instruction: Instruction = mint_to(
        &spl_token::id(),
        &mint,
        &destination,
        &authority,
        &[], // multisig signers
        body.amount,
    )
    .map_err(|e| ApiError::InternalError(format!("Failed to create mint instruction: {e}")))?;

    let accounts = instruction
        .accounts
        .into_iter()
        .map(|meta| AccountMetaResponse {
            pubkey: meta.pubkey.to_string(),
            is_signer: meta.is_signer,
            is_writable: meta.is_writable,
        })
        .collect();

    let data = base64::engine::general_purpose::STANDARD.encode(instruction.data);

    Ok(HttpResponse::Ok().json(SuccessResponse {
        success: true,
        data: MintInstructionResponse {
            program_id: instruction.program_id.to_string(),
            accounts,
            instruction_data: data,
        },
    }))
}





pub async fn sign_message(
    body: web::Json<SignMessageRequest>,
) -> Result<impl Responder, ApiError> {
    if body.message.trim().is_empty() || body.secret.trim().is_empty() {
        return Err(ApiError::BadRequest("Missing required fields".to_string()));
    }
    let secret_bytes = bs58::decode(&body.secret)
        .into_vec()
        .map_err(|_| ApiError::BadRequest("Invalid base58 secret key".into()))?;

    let keypair = Keypair::from_bytes(&secret_bytes)
        .map_err(|_| ApiError::BadRequest("Invalid secret key format".into()))?;

    let signature = keypair.sign_message(body.message.as_bytes());

    Ok(HttpResponse::Ok().json(SuccessResponse {
        success: true,
        data: SignMessageResponse {
            signature: base64::engine::general_purpose::STANDARD.encode(signature),
            public_key: keypair.pubkey().to_string(),
            message: body.message.clone(),

        },
    }))
}


pub async fn verify_message(
    body: web::Json<VerifyMessageRequest>,
) -> Result<impl Responder, ApiError> {
    if body.message.trim().is_empty() || body.signature.trim().is_empty() || body.pubkey.trim().is_empty() {
        return Err(ApiError::BadRequest("Missing required fields".to_string()));
    }

    let pubkey = Pubkey::from_str(&body.pubkey)
        .map_err(|_| ApiError::BadRequest("Invalid public key".into()))?;

    let signature_bytes = base64::decode(&body.signature)
        .map_err(|_| ApiError::BadRequest("Invalid base64 signature".into()))?;

    let signature = Signature::try_from(signature_bytes.as_slice())
        .map_err(|_| ApiError::BadRequest("Invalid signature format".into()))?;

    let is_valid = signature.verify(pubkey.as_ref(), body.message.as_bytes());

    Ok(HttpResponse::Ok().json(SuccessResponse {
        success: true,
        data: VerifyMessageResponse {
            valid: is_valid,
            message: body.message.clone(),
            pubkey: body.pubkey.clone(),
        },
    }))
}


pub async fn send_sol(
    body: web::Json<SendSolRequest>,
) -> Result<impl Responder, ApiError> {
    if body.lamports == 0 {
        return Err(ApiError::BadRequest("lamports must be greater than 0".into()));
    }

    let from = Pubkey::from_str(&body.from)
        .map_err(|_| ApiError::BadRequest("Invalid 'from' address".into()))?;

    let to = Pubkey::from_str(&body.to)
        .map_err(|_| ApiError::BadRequest("Invalid 'to' address".into()))?;

    let instruction: Instruction = system_instruction::transfer(&from, &to, body.lamports);

    let account_keys = instruction.accounts.iter().map(|acc| acc.pubkey.to_string()).collect();
    let data = base64::encode(instruction.data);

    Ok(HttpResponse::Ok().json(SuccessResponse {
        success: true,
        data: SendSolResponse {
            program_id: instruction.program_id.to_string(),
            accounts: account_keys,
            instruction_data: data,
        },
    }))
}



pub async fn send_token(
    body: web::Json<SendTokenRequest>,
) -> Result<impl Responder, ApiError> {
    if body.amount == 0 {
        return Err(ApiError::BadRequest("amount must be greater than 0".into()));
    }

    let owner = Pubkey::from_str(&body.owner)
        .map_err(|_| ApiError::BadRequest("Invalid owner pubkey".into()))?;

    let destination = Pubkey::from_str(&body.destination)
        .map_err(|_| ApiError::BadRequest("Invalid destination pubkey".into()))?;

    let mint = Pubkey::from_str(&body.mint)
        .map_err(|_| ApiError::BadRequest("Invalid mint pubkey".into()))?;

 
    let source_token_account = spl_associated_token_account::get_associated_token_address(&owner, &mint);
    let destination_token_account = spl_associated_token_account::get_associated_token_address(&destination, &mint);

    let instruction: Instruction = spl_token::instruction::transfer(
        &spl_token::id(),
        &source_token_account,
        &destination_token_account,
        &owner,
        &[],
        body.amount,
    )
    .map_err(|e| ApiError::InternalError(format!("Failed to create token transfer instruction: {e}")))?;

    let accounts = instruction
        .accounts
        .iter()
        .map(|acc| AccountMetaSimple {
            pubkey: acc.pubkey.to_string(),
            is_signer: acc.is_signer,
        })
        .collect();

    let data = base64::encode(instruction.data);

    Ok(HttpResponse::Ok().json(SuccessResponse {
        success: true,
        data: SendTokenResponse {
            program_id: instruction.program_id.to_string(),
            accounts,
            instruction_data: data,
        },
    }))
}