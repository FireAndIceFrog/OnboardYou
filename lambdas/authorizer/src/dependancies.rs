use std::sync::Arc;

use crate::{engine::auth_engine::{AuthEngine, IAuthEngine}, models::AuthConfig, repositories::jwks_repository::{JwksRepository}};

// Public impls for constructing the auth engine and its dependancies.
#[derive(Clone)]
pub struct Dependancies {
    // --engines--
    pub auth_engine: Arc<dyn IAuthEngine>,
}

impl Dependancies {
    pub fn new() -> Self {
        
        let auth_config = Arc::new(AuthConfig::from_env());
        let jwks_repository = JwksRepository::new();


        Self {
            auth_engine: AuthEngine::new(jwks_repository.clone(), auth_config.clone()),
        }
    }
}