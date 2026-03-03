use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WelcomeMessageRecord {
    pub id: String,
    pub guild_id: String,
    pub message_id: String,
    pub channel_id: String,
    pub roles: String,
    pub captcha: String,
    pub captcha_type: String,
    pub captcha_len: Option<u32>,
}

#[derive(Serialize, Deserialize)]
struct PocketBaseListResponse<T> {
    pub items: Vec<T>,
}

#[derive(Clone)]
pub struct PbClient {
    client: Client,
    base_url: String,
}

impl PbClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::builder()
                .no_proxy()
                .build()
                .unwrap_or_else(|_| reqwest::Client::new()),
            base_url: base_url.trim_end_matches('/').to_string(),
        }
    }

    pub async fn get_welcome_message(&self, message_id: &str) -> Option<WelcomeMessageRecord> {
        let url = format!(
            "{}/api/collections/welcome_messages/records?filter=(message_id='{}')",
            self.base_url, message_id
        );

        if let Ok(res) = self.client.get(&url).send().await {
            if res.status().is_success() {
                match res
                    .json::<PocketBaseListResponse<WelcomeMessageRecord>>()
                    .await
                {
                    Ok(data) => return data.items.into_iter().next(),
                    Err(e) => tracing::error!("Failed to parse PocketBase welcome message: {}", e),
                }
            } else {
                tracing::error!("PocketBase returned error status: {}", res.status());
            }
        } else {
            tracing::error!("Failed to reach PocketBase");
        }
        None
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn save_welcome_message(
        &self,
        guild_id: &str,
        message_id: &str,
        channel_id: &str,
        roles: &str,
        captcha: &str,
        captcha_type: &str,
        captcha_len: u32,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = format!("{}/api/collections/welcome_messages/records", self.base_url);

        let payload = serde_json::json!({
            "guild_id": guild_id,
            "message_id": message_id,
            "channel_id": channel_id,
            "roles": roles,
            "captcha": captcha,
            "captcha_type": captcha_type,
            "captcha_len": captcha_len,
        });

        let res = self.client.post(&url).json(&payload).send().await?;
        if !res.status().is_success() {
            let status = res.status();
            let text = res.text().await.unwrap_or_default();
            tracing::error!("PocketBase save failed: {} - {}", status, text);
            return Err(format!("PocketBase API error: {}", status).into());
        }
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GreetingRecord {
    pub id: String,
    pub guild_id: String,
    pub channel_id: String,
    pub enabled: String,
    pub text: Option<String>,
}

impl PbClient {
    pub async fn save_greeting(
        &self,
        guild_id: &str,
        channel_id: &str,
        enabled: &str,
        text: &str,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let payload = serde_json::json!({
            "guild_id": guild_id,
            "channel_id": channel_id,
            "enabled": enabled,
            "text": text,
        });

        if let Some(existing) = self.get_greeting(guild_id).await {
            let update_url = format!(
                "{}/api/collections/greetings/records/{}",
                self.base_url, existing.id
            );
            self.client.patch(&update_url).json(&payload).send().await?;
        } else {
            let url = format!("{}/api/collections/greetings/records", self.base_url);
            self.client.post(&url).json(&payload).send().await?;
        }
        Ok(())
    }

    pub async fn get_greeting(&self, guild_id: &str) -> Option<GreetingRecord> {
        let url = format!(
            "{}/api/collections/greetings/records?filter=(guild_id='{}')",
            self.base_url, guild_id
        );
        if let Ok(res) = self.client.get(&url).send().await
            && let Ok(json) = res.json::<serde_json::Value>().await
            && let Some(items) = json["items"].as_array()
            && !items.is_empty()
        {
            return serde_json::from_value(items[0].clone()).ok();
        }
        None
    }
}
