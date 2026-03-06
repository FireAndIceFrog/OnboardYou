use serde::Deserialize;

#[derive(Debug)]
pub struct ApiClient {
    client: reqwest::Client,
    base_url: String,
    id_token: String,
}

#[derive(Debug, Deserialize)]
struct LoginResponse {
    id_token: String,
}

impl ApiClient {
    /// Authenticate and return a ready-to-use client.
    pub async fn login(base_url: &str, email: &str, password: &str) -> anyhow::Result<Self> {
        let client = reqwest::Client::new();
        let resp = client
            .post(format!("{base_url}/auth/login"))
            .json(&serde_json::json!({ "email": email, "password": password }))
            .send()
            .await?
            .error_for_status()?
            .json::<LoginResponse>()
            .await?;

        Ok(Self {
            client,
            base_url: base_url.to_string(),
            id_token: resp.id_token,
        })
    }

    pub async fn post(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .post(format!("{}{path}", self.base_url))
            .bearer_auth(&self.id_token)
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("API {status}: {text}");
        }
        Ok(serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text)))
    }

    pub async fn get(&self, path: &str) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .get(format!("{}{path}", self.base_url))
            .bearer_auth(&self.id_token)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("API {status}: {text}");
        }
        Ok(serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text)))
    }

    pub async fn put(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let resp = self
            .client
            .put(format!("{}{path}", self.base_url))
            .bearer_auth(&self.id_token)
            .json(body)
            .send()
            .await?;
        let status = resp.status();
        let text = resp.text().await?;
        if !status.is_success() {
            anyhow::bail!("API {status}: {text}");
        }
        Ok(serde_json::from_str(&text).unwrap_or(serde_json::Value::String(text)))
    }
}
