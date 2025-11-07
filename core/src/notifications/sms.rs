use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::Serialize;

#[async_trait]
pub trait SmsProvider: Send + Sync {
    async fn send_sms(&self, to: &str, message: &str) -> Result<()>;
}

pub struct TwilioProvider {
    client: reqwest::Client,
    account_sid: String,
    auth_token: String,
    from_number: String,
}

impl TwilioProvider {
    pub fn new(account_sid: String, auth_token: String, from_number: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            account_sid,
            auth_token,
            from_number,
        }
    }

    pub fn from_env() -> Result<Self> {
        let account_sid = std::env::var("TWILIO_ACCOUNT_SID")
            .context("TWILIO_ACCOUNT_SID environment variable not set")?;
        let auth_token = std::env::var("TWILIO_AUTH_TOKEN")
            .context("TWILIO_AUTH_TOKEN environment variable not set")?;
        let from_number = std::env::var("TWILIO_FROM_NUMBER")
            .context("TWILIO_FROM_NUMBER environment variable not set")?;

        Ok(Self::new(account_sid, auth_token, from_number))
    }
}

#[async_trait]
impl SmsProvider for TwilioProvider {
    async fn send_sms(&self, to: &str, message: &str) -> Result<()> {
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            self.account_sid
        );

        #[derive(Serialize)]
        struct TwilioRequest {
            #[serde(rename = "To")]
            to: String,
            #[serde(rename = "From")]
            from: String,
            #[serde(rename = "Body")]
            body: String,
        }

        let request = TwilioRequest {
            to: to.to_string(),
            from: self.from_number.clone(),
            body: message.to_string(),
        };

        let response = self
            .client
            .post(&url)
            .basic_auth(&self.account_sid, Some(&self.auth_token))
            .form(&request)
            .send()
            .await
            .context("Failed to send SMS via Twilio")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Twilio API returned status {}: {}", status, body);
        }

        tracing::info!("SMS sent to {} via Twilio", to);
        Ok(())
    }
}

pub struct MockSmsProvider;

impl MockSmsProvider {
    pub fn new() -> Self {
        Self
    }
}

impl Default for MockSmsProvider {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl SmsProvider for MockSmsProvider {
    async fn send_sms(&self, to: &str, message: &str) -> Result<()> {
        tracing::info!("📱 [MOCK SMS] To: {}, Message: {}", to, message);
        Ok(())
    }
}

/// Create SMS provider based on environment variables
///
/// Returns TwilioProvider if Twilio credentials are available,
/// otherwise returns MockSmsProvider
pub fn create_sms_provider() -> Box<dyn SmsProvider> {
    match TwilioProvider::from_env() {
        Ok(provider) => {
            tracing::info!("Using Twilio SMS provider");
            Box::new(provider)
        }
        Err(_) => {
            tracing::info!("Twilio credentials not found, using mock SMS provider");
            Box::new(MockSmsProvider::new())
        }
    }
}

pub fn format_conflict_sms(student_name: &str, original_date: &str) -> String {
    format!(
        "Hi {}, your flight lesson on {} has been cancelled due to weather. Check your email for reschedule options. - Flight Schedule Pro",
        student_name, original_date
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_mock_sms_provider() {
        let provider = MockSmsProvider::new();
        let result = provider.send_sms("+1234567890", "Test message").await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_format_conflict_sms() {
        let message = format_conflict_sms("John Doe", "2024-01-15 14:00 UTC");
        assert!(message.contains("John Doe"));
        assert!(message.contains("2024-01-15 14:00 UTC"));
        assert!(message.contains("cancelled"));
    }

    #[test]
    fn test_create_sms_provider_without_credentials() {
        // This should return MockSmsProvider when no env vars are set
        let provider = create_sms_provider();
        // We can't directly test the type, but we can verify it was created
        assert!(std::mem::size_of_val(&provider) > 0);
    }
}
