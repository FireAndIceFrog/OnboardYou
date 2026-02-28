use std::sync::Arc;

use crate::{
    engine::auth_engine::{AuthEngine, IAuthEngine},
    models::AuthConfig,
    repositories::jwks_repository::{IJwksRepository, JwksRepository},
};

// Public impls for constructing the auth engine and its dependancies.
#[derive(Clone)]
pub struct Dependancies {
    // --engines--
    pub auth_engine: Arc<dyn IAuthEngine>,
    pub jwks_repository: Arc<dyn IJwksRepository>,
}

impl Dependancies {
    pub fn create_env() -> Arc<AuthConfig> {
        Arc::new(AuthConfig::from_env())
    }

    pub fn new(cfg: Arc<AuthConfig>) -> Self {
        let jwks_repository = JwksRepository::new();

        Self {
            auth_engine: AuthEngine::new(cfg.clone()),
            jwks_repository: jwks_repository,
        }
    }
}
