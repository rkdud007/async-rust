use crate::provider::RpcProvider;
use std::sync::Arc;
pub mod provider;

async fn sleep_then_print(provider: Arc<RpcProvider>, block_number: u64) -> Result<u64, String> {
    let start_time = std::time::Instant::now();
    println!("Start timer {}.", block_number);

    {
        let request_cache = provider.request_cache.lock().unwrap();
        if let Some(nonce) = request_cache.get(&block_number) {
            println!("Timer {} is cached.", block_number);
            let elapsed = start_time.elapsed();
            println!("Timer {} took {:?}", block_number, elapsed);
            return Ok(*nonce);
        }
    }

    let mut pending_requests = provider.pending_requests.lock().unwrap();

    if pending_requests.contains(&block_number) {
        println!("Timer {} is pending.", block_number);
        drop(pending_requests);
        while provider
            .pending_requests
            .lock()
            .unwrap()
            .contains(&block_number)
        {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
        let nonce = *provider
            .request_cache
            .lock()
            .unwrap()
            .get(&block_number)
            .ok_or_else(|| "Failed to get nonce after waiting for pending request".to_string())?;
        let elapsed = start_time.elapsed();
        println!("Timer {} took {:?}", block_number, elapsed);
        return Ok(nonce);
    } else {
        pending_requests.insert(block_number);
        drop(pending_requests);
    }

    let nonce = provider
        .get_transaction_count("0x7f2c6f930306d3aa736b3a6c6a98f512f74036d4", block_number)
        .await
        .map_err(|e| format!("Failed to get transaction count: {:?}", e))?;

    let mut request_cache = provider.request_cache.lock().unwrap();
    request_cache.insert(block_number, nonce);

    let mut pending_requests = provider.pending_requests.lock().unwrap();
    pending_requests.remove(&block_number);

    let elapsed = start_time.elapsed();
    println!("Timer {} took {:?}", block_number, elapsed);

    Ok(nonce)
}

#[tokio::main]
async fn main() {
    let provider =
        RpcProvider::new("https://eth-sepolia.g.alchemy.com/v2/xar76cftwEtqTBWdF4ZFy9n8FLHAETDv");
    let provider = Arc::new(provider);

    let start_time = std::time::Instant::now();
    let nonces = tokio::join!(
        sleep_then_print(provider.clone(), 5604994),
        sleep_then_print(provider.clone(), 5604994),
        sleep_then_print(provider.clone(), 5604994),
        sleep_then_print(provider, 5604994),
    );

    let elapsed = start_time.elapsed();
    println!("Total time took {:?}", elapsed);

    match nonces {
        (Ok(nonce1), Ok(nonce2), Ok(nonce3), Ok(nonce4)) => {
            println!("Nonces: {}, {}, {}, {}", nonce1, nonce2, nonce3, nonce4);
        }
        (Err(e), _, _, _) => {
            println!("Error: {:?}", e);
        }
        (_, Err(e), _, _) => {
            println!("Error: {:?}", e);
        }
        (_, _, Err(e), _) => {
            println!("Error: {:?}", e);
        }
        (_, _, _, Err(e)) => {
            println!("Error: {:?}", e);
        }
    }
}
