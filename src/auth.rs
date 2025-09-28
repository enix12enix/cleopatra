use axum::{
    async_trait,
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, TokenData};
use std::{fs, sync::Arc};

use crate::{models::Claims, state::AppState, config::Config};

#[async_trait]
pub trait JwtVerifier: Send + Sync {
    async fn verify(&self, token: &str) -> Result<TokenData<Claims>, (StatusCode, String)>;
}

pub struct AuthProvider {
    decoding_key: DecodingKey,
    validation: Validation,
}

impl AuthProvider {
    pub fn new(config: &Config) -> anyhow::Result<Self> {
        let algorithm_str = config.auth.algorithm.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Auth algorithm is required when auth is enabled"))?;
        let key_file_path = config.auth.secret_path.as_ref()
            .ok_or_else(|| anyhow::anyhow!("Auth secret_path is required when auth is enabled"))?;

        let algorithm = match algorithm_str.to_uppercase().as_str() {
            "HS256" => Algorithm::HS256,
            "RS256" => Algorithm::RS256,
            "ES256" => Algorithm::ES256,
            _ => anyhow::bail!("Unsupported algorithm: {}", algorithm_str),
        };

        let key_content = fs::read_to_string(key_file_path)
            .map_err(|e| anyhow::anyhow!("Failed to read key file {}: {}", key_file_path, e))?
            .trim()
            .to_string();

        let decoding_key = match algorithm {
            Algorithm::HS256 => DecodingKey::from_secret(key_content.as_ref()),
            Algorithm::RS256 => DecodingKey::from_rsa_pem(key_content.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid RSA key: {}", e))?,
            Algorithm::ES256 => DecodingKey::from_ec_pem(key_content.as_bytes())
                .map_err(|e| anyhow::anyhow!("Invalid EC key: {}", e))?,
            _ => anyhow::bail!("Unsupported algorithm {:?}", algorithm),
        };

        Ok(Self {
            validation: Validation::new(algorithm),
            decoding_key,
        })
    }
}

#[async_trait]
impl JwtVerifier for AuthProvider {
    async fn verify(&self, token: &str) -> Result<TokenData<Claims>, (StatusCode, String)> {
        decode::<Claims>(token, &self.decoding_key, &self.validation)
            .map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or expired token".into()))
    }
}

/// JWT authentication middleware
/// 
/// Checks for Bearer token in Authorization header and validates it using the AuthProvider.
/// If auth_provider is None (auth is disabled), bypasses validation.
pub async fn jwt_auth_middleware(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, (StatusCode, String)> {
    // If auth provider is None, bypass JWT validation
    let auth_provider = match &state.auth_provider {
        Some(provider) => provider,
        None => return Ok(next.run(request).await),
    };

    // Extract the token from the Authorization header
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|v| v.to_str().ok())
        .ok_or((StatusCode::UNAUTHORIZED, "Missing Authorization header".to_string()))?;

    if !auth_header.starts_with("Bearer ") {
        return Err((StatusCode::UNAUTHORIZED, "Invalid authorization header format".to_string()));
    }

    let token = auth_header.trim_start_matches("Bearer ").trim();

    // Verify the token using the auth provider
    match auth_provider.verify(token).await {
        Ok(_token_data) => Ok(next.run(request).await),
        Err((_status, error)) => Err((StatusCode::UNAUTHORIZED, error)),
    }
}
