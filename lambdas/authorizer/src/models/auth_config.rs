
/// Configuration read once at cold-start.
pub struct AuthConfig {
    pub dev_mode: bool,
    pub user_pool_id: Option<String>,
    pub client_id: Option<String>,
    pub aws_region: String,
}

impl AuthConfig {
    pub fn from_env() -> Self {
        Self {
            dev_mode: std::env::var("AUTH_DEV_MODE").unwrap_or_default() == "true",
            user_pool_id: std::env::var("COGNITO_USER_POOL_ID").ok(),
            client_id: std::env::var("COGNITO_CLIENT_ID").ok(),
            aws_region: std::env::var("AWS_REGION").unwrap_or_else(|_| "eu-west-1".into()),
        }
    }
}