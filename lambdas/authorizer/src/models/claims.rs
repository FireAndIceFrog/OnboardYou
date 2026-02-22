use serde::{Deserialize, Serialize};



#[derive(Debug, Deserialize, Serialize)]
pub struct Claims {
    pub sub: Option<String>,
    #[serde(rename = "custom:organizationId")]
    pub organization_id: Option<String>,
}
