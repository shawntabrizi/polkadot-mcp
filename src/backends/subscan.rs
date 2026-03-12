/// Subscan backend: indexed/historical data via Subscan REST API.
///
/// Used for data that isn't efficiently queryable from runtime storage:
///   - Transaction/transfer history
///   - Historical staking rewards
///   - Extrinsic search
///
/// API docs: https://support.subscan.io/

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Subscan API client for a specific chain.
pub struct SubscanClient {
    /// Base URL, e.g., "https://polkadot.api.subscan.io"
    base_url: String,
    /// Optional API key for higher rate limits.
    api_key: Option<String>,
    client: reqwest::Client,
}

impl SubscanClient {
    pub fn new(chain_name: &str, api_key: Option<String>) -> Self {
        let base_url = match chain_name {
            "polkadot" => "https://polkadot.api.subscan.io".to_string(),
            "kusama" => "https://kusama.api.subscan.io".to_string(),
            "westend" => "https://westend.api.subscan.io".to_string(),
            "polkadot-asset-hub" => "https://assethub-polkadot.api.subscan.io".to_string(),
            "polkadot-collectives" => "https://collectives-polkadot.api.subscan.io".to_string(),
            other => format!("https://{}.api.subscan.io", other),
        };

        Self {
            base_url,
            api_key,
            client: reqwest::Client::new(),
        }
    }

    /// Fetch recent transfers for an account.
    pub async fn get_transfers(
        &self,
        address: &str,
        limit: u32,
    ) -> Result<Vec<TransferRecord>> {
        let url = format!("{}/api/v2/scan/transfers", self.base_url);

        let mut req = self.client.post(&url).json(&serde_json::json!({
            "address": address,
            "row": limit,
            "page": 0,
        }));

        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp: SubscanResponse<TransfersData> = req.send().await?.json().await?;

        Ok(resp.data.map(|d| d.transfers).unwrap_or_default())
    }

    /// Fetch staking reward history for an account.
    pub async fn get_reward_history(
        &self,
        address: &str,
        limit: u32,
    ) -> Result<Vec<RewardRecord>> {
        let url = format!("{}/api/v2/scan/account/reward_slash", self.base_url);

        let mut req = self.client.post(&url).json(&serde_json::json!({
            "address": address,
            "row": limit,
            "page": 0,
        }));

        if let Some(key) = &self.api_key {
            req = req.header("X-API-Key", key);
        }

        let resp: SubscanResponse<RewardsData> = req.send().await?.json().await?;

        Ok(resp.data.map(|d| d.list).unwrap_or_default())
    }
}

// --- Response types ---

#[derive(Debug, Deserialize)]
struct SubscanResponse<T> {
    code: i32,
    message: String,
    data: Option<T>,
}

#[derive(Debug, Deserialize)]
struct TransfersData {
    transfers: Vec<TransferRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TransferRecord {
    pub from: String,
    pub to: String,
    pub amount: String,
    pub block_num: u64,
    pub block_timestamp: u64,
    pub hash: String,
    pub success: bool,
}

#[derive(Debug, Deserialize)]
struct RewardsData {
    list: Vec<RewardRecord>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewardRecord {
    pub event_index: String,
    pub amount: String,
    pub block_num: u64,
    pub block_timestamp: u64,
    pub era: Option<u32>,
}
