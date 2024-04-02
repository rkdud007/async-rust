use std::{
    collections::{HashMap, HashSet},
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, Result};
use reqwest::{header, Client};
use serde_json::{from_value, json, Value};

#[derive(Debug, Clone)]
pub struct RpcProvider {
    pub pending_requests: Arc<Mutex<HashSet<u64>>>,
    pub request_cache: Arc<Mutex<HashMap<u64, u64>>>,
    client: Client,
    url: &'static str,
}

impl RpcProvider {
    pub fn new(rpc_url: &'static str) -> Self {
        Self {
            client: Client::new(),
            url: rpc_url,
            pending_requests: Arc::new(Mutex::new(HashSet::new())),
            request_cache: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl RpcProvider {
    pub async fn get_latest_block_number(&self) -> Result<u64> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_blockNumber",
            "params": [],
            "id": 1,
        });

        let response = self
            .client
            .post(self.url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&rpc_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        // Check if the response status is success
        if !response.status().is_success() {
            return Err(anyhow!(
                "RPC request `eth_blockNumber` failed with status: {}",
                response.status()
            ));
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        let block_number: String = from_value(result.clone())?;
        let block_number_u64 = u64::from_str_radix(&block_number[2..], 16).unwrap();

        Ok(block_number_u64)
    }

    pub async fn get_transaction_count(&self, address: &str, block_number: u64) -> Result<u64> {
        let rpc_request: Value = json!({
            "jsonrpc": "2.0",
            "method": "eth_getTransactionCount",
            "params": [address, format!("0x{:x}", block_number)],
            "id": 1,
        });

        let response = self
            .client
            .post(self.url)
            .header(header::CONTENT_TYPE, "application/json")
            .json(&rpc_request)
            .send()
            .await
            .map_err(|e| anyhow!("Failed to send request: {}", e))?;

        // Check if the response status is success
        if !response.status().is_success() {
            return Err(anyhow!(
                "RPC request `eth_getTransactionCount` failed with status: {}",
                response.status()
            ));
        }

        // Parse the response body as JSON
        let rpc_response: Value = response
            .json()
            .await
            .map_err(|e| anyhow!("Failed to parse response: {}", e))?;
        let result = &rpc_response["result"];

        let tx_count: String = from_value(result.clone())?;
        let tx_count_u64 = u64::from_str_radix(&tx_count[2..], 16).unwrap();

        Ok(tx_count_u64)
    }
}
