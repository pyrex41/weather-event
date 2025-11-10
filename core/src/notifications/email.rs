use crate::ai::RescheduleOption;
use crate::models::Booking;
use anyhow::{Context, Result};
use serde::Serialize;

pub struct EmailClient {
    client: reqwest::Client,
    api_key: String,
    from_email: String,
}

#[derive(Serialize)]
struct ResendEmailRequest {
    from: String,
    to: Vec<String>,
    subject: String,
    html: String,
}

impl EmailClient {
    pub fn new(api_key: String, from_email: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            api_key,
            from_email,
        }
    }

    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("RESEND_API_KEY")
            .context("RESEND_API_KEY environment variable not set")?;
        let from_email = std::env::var("FROM_EMAIL")
            .unwrap_or_else(|_| "alerts@flightschedulepro.com".to_string());

        Ok(Self::new(api_key, from_email))
    }

    pub async fn send_conflict_email(
        &self,
        to: &str,
        booking: &Booking,
        options: &[RescheduleOption],
    ) -> Result<()> {
        let html = self.build_email_html(booking, options);

        let request = ResendEmailRequest {
            from: self.from_email.clone(),
            to: vec![to.to_string()],
            subject: format!(
                "Flight Lesson Cancelled Due to Weather - {}",
                booking.scheduled_date.format("%Y-%m-%d %H:%M")
            ),
            html,
        };

        let response = self
            .client
            .post("https://api.resend.com/emails")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .context("Failed to send email")?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("Resend API returned status {}: {}", status, body);
        }

        tracing::info!("Email sent to {} for booking {}", to, booking.id);
        Ok(())
    }

    fn build_email_html(&self, booking: &Booking, options: &[RescheduleOption]) -> String {
        let options_html: String = options
            .iter()
            .map(|opt| {
                format!(
                    r#"
                    <div style="border: 1px solid #e0e0e0; border-radius: 8px; padding: 16px; margin: 12px 0; background: #f9f9f9;">
                        <h3 style="margin: 0 0 8px 0; color: #2563eb;">
                            {}
                        </h3>
                        <p style="margin: 4px 0; color: #666;">
                            <strong>Reason:</strong> {}
                        </p>
                        <p style="margin: 4px 0; color: #666;">
                            <strong>Weather Score:</strong> {:.1}/10
                        </p>
                        <p style="margin: 4px 0; color: #666;">
                            <strong>Instructor:</strong> {}
                        </p>
                    </div>
                "#,
                    opt.date_time.format("%A, %B %d, %Y at %I:%M %p UTC"),
                    opt.reason,
                    opt.weather_score,
                    if opt.instructor_available {
                        "Available"
                    } else {
                        "Check availability"
                    }
                )
            })
            .collect();

        format!(
            r#"
<!DOCTYPE html>
<html>
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
</head>
<body style="font-family: Arial, sans-serif; line-height: 1.6; color: #333; max-width: 600px; margin: 0 auto; padding: 20px;">
    <div style="background: linear-gradient(135deg, #667eea 0%, #764ba2 100%); color: white; padding: 30px; border-radius: 8px; text-align: center;">
        <h1 style="margin: 0; font-size: 28px;">⛈️ Weather Alert</h1>
        <p style="margin: 10px 0 0 0; font-size: 16px;">Your flight lesson has been cancelled</p>
    </div>

    <div style="background: #fff; padding: 24px; margin: 20px 0; border-radius: 8px; box-shadow: 0 2px 4px rgba(0,0,0,0.1);">
        <h2 style="color: #dc2626; margin-top: 0;">Flight Lesson Cancelled</h2>
        <p>Unfortunately, your scheduled flight lesson has been cancelled due to unsafe weather conditions.</p>

        <div style="background: #fef2f2; border-left: 4px solid #dc2626; padding: 16px; margin: 16px 0; border-radius: 4px;">
            <p style="margin: 0;"><strong>Original Booking:</strong></p>
            <p style="margin: 8px 0 0 0;">
                {} at {}
            </p>
        </div>

        <h2 style="color: #2563eb; margin-top: 32px;">Suggested Reschedule Options</h2>
        <p>We've identified the following alternative times with better weather conditions:</p>

        {}

        <div style="background: #eff6ff; border-left: 4px solid #2563eb; padding: 16px; margin: 24px 0; border-radius: 4px;">
            <p style="margin: 0;"><strong>💡 What's Next?</strong></p>
            <p style="margin: 8px 0 0 0;">
                Please log in to your dashboard to select one of these options or choose a different time that works for you.
            </p>
        </div>

        <div style="text-align: center; margin-top: 32px;">
            <a href="https://flightschedulepro.com/dashboard" style="display: inline-block; background: #2563eb; color: white; padding: 14px 32px; text-decoration: none; border-radius: 6px; font-weight: bold;">
                View Dashboard
            </a>
        </div>
    </div>

    <div style="text-align: center; color: #666; font-size: 12px; margin-top: 32px; padding-top: 20px; border-top: 1px solid #e0e0e0;">
        <p>Flight Schedule Pro - Weather-Aware Flight Training</p>
        <p>Questions? Contact us at support@flightschedulepro.com</p>
    </div>
</body>
</html>
            "#,
            booking.scheduled_date.format("%A, %B %d, %Y"),
            booking.scheduled_date.format("%I:%M %p UTC"),
            options_html
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{BookingStatus, Location};
    use chrono::Utc;

    #[test]
    fn test_email_html_generation() {
        let client = EmailClient::new("test_key".to_string(), "test@example.com".to_string());

        let booking = Booking {
            id: "test123".to_string(),
            student_id: "student1".to_string(),
            aircraft_type: "Cessna 172".to_string(),
            scheduled_date: Utc::now(),
            departure_location: Location {
                lat: 33.8113,
                lon: -118.1515,
                name: "KTOA".to_string(),
            },
            status: BookingStatus::Cancelled,
        };

        let options = vec![
            RescheduleOption {
                date_time: Utc::now() + chrono::Duration::days(1),
                reason: "Clear skies".to_string(),
                weather_score: 9.5,
                instructor_available: true,
            },
        ];

        let html = client.build_email_html(&booking, &options);

        assert!(html.contains("Weather Alert"));
        assert!(html.contains("Clear skies"));
        assert!(html.contains("9.5/10"));
    }
}
