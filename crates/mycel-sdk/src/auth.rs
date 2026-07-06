use std::sync::{Arc, RwLock};

use tonic::{metadata::MetadataValue, service::Interceptor, Request, Status};

const AUTHORIZATION_HEADER: &str = "authorization";

#[derive(Debug, Clone, Default)]
pub struct TokenSource {
    token: Arc<RwLock<String>>,
}

impl TokenSource {
    pub fn new(token: impl Into<String>) -> Self {
        Self {
            token: Arc::new(RwLock::new(token.into().trim().to_string())),
        }
    }

    pub fn set(&self, token: impl Into<String>) {
        *self.token.write().expect("token lock poisoned") = token.into().trim().to_string();
    }

    pub fn token(&self) -> String {
        self.token.read().expect("token lock poisoned").clone()
    }

    pub fn interceptor(&self) -> AuthInterceptor {
        AuthInterceptor {
            tokens: self.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthInterceptor {
    tokens: TokenSource,
}

impl Interceptor for AuthInterceptor {
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        let token = self.tokens.token();
        if !token.is_empty() {
            let value = format!("Bearer {token}")
                .parse::<MetadataValue<_>>()
                .map_err(|_| Status::unauthenticated("invalid access token metadata"))?;
            request.metadata_mut().insert(AUTHORIZATION_HEADER, value);
        }
        Ok(request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_source_trims_and_updates() {
        let tokens = TokenSource::new(" abc ");
        assert_eq!(tokens.token(), "abc");
        tokens.set(" def ");
        assert_eq!(tokens.token(), "def");
    }
}
