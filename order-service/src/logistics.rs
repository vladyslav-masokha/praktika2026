use common::errors::AppError;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct ShipmentInfo {
    pub provider: String,
    pub tracking_number: String,
    pub tracking_url: Option<String>,
}

#[derive(Debug, Serialize)]
struct ExternalShipmentRequest {
    order_id: i64,
    user_id: i64,
}

#[derive(Debug, Deserialize)]
struct ExternalShipmentResponse {
    tracking_number: Option<String>,
    tracking_url: Option<String>,
}

pub async fn create_shipment(
    client: &Client,
    provider: &str,
    api_base_url: Option<&str>,
    api_key: Option<&str>,
    order_id: i64,
    user_id: i64,
) -> Result<ShipmentInfo, AppError> {
    match provider {
        "novaposhta" | "dhl" => {
            let Some(base_url) = api_base_url else {
                return Ok(mock_shipment(provider, order_id));
            };

            let url = format!("{}/shipments", base_url.trim_end_matches('/'));
            let request = ExternalShipmentRequest { order_id, user_id };
            let mut call = client.post(url).json(&request);

            if let Some(key) = api_key {
                call = call.bearer_auth(key);
            }

            let response = call
                .send()
                .await
                .map_err(|e| AppError::Internal(format!("{} logistics call failed: {}", provider, e)))?;

            if !response.status().is_success() {
                return Err(AppError::Internal(format!(
                    "{} logistics returned HTTP {}",
                    provider,
                    response.status()
                )));
            }

            let payload: ExternalShipmentResponse = response
                .json()
                .await
                .map_err(|e| AppError::Internal(format!("{} logistics response parse failed: {}", provider, e)))?;

            Ok(ShipmentInfo {
                provider: provider.to_string(),
                tracking_number: payload
                    .tracking_number
                    .unwrap_or_else(|| format!("{}-{}", provider.to_ascii_uppercase(), order_id)),
                tracking_url: payload.tracking_url,
            })
        }
        _ => Ok(mock_shipment("mock", order_id)),
    }
}

fn mock_shipment(provider: &str, order_id: i64) -> ShipmentInfo {
    ShipmentInfo {
        provider: provider.to_string(),
        tracking_number: format!("{}-TTN-{}", provider.to_ascii_uppercase(), order_id),
        tracking_url: Some(format!(
            "https://tracking.local/{}/{}",
            provider.to_ascii_lowercase(),
            order_id
        )),
    }
}
