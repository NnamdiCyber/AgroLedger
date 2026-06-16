use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use tokio::time::interval;

mod attestation;
mod price;

#[derive(Debug, Clone, Deserialize)]
pub struct OracleConfig {
    pub stellar: StellarConfig,
    pub signing: SigningConfig,
    pub price_feeds: PriceFeedConfig,
    pub attestation: AttestationConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct StellarConfig {
    pub rpc_url: String,
    pub network_passphrase: String,
    pub horizon_url: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SigningConfig {
    pub secret_key: String,
    pub multisig_threshold: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct PriceFeedConfig {
    pub interval_seconds: u64,
    pub afex: ApiConfig,
    pub gcx: ApiConfig,
    pub cme: ApiConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AttestationConfig {
    pub max_lot_weight_kg: u64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    pub enabled: bool,
    pub api_key: String,
    pub api_url: String,
}

pub struct AppState {
    pub config: OracleConfig,
    pub last_prices: Mutex<HashMap<String, price::PriceQuote>>,
}

impl AppState {
    pub fn new(config: OracleConfig) -> Self {
        Self {
            config,
            last_prices: Mutex::new(HashMap::new()),
        }
    }
}

fn load_config(path: &str) -> Result<OracleConfig, Box<dyn std::error::Error>> {
    let contents = fs::read_to_string(Path::new(path))?;
    let config: OracleConfig = toml::from_str(&contents)?;
    Ok(config)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("AgroLedger Oracle Sidecar v0.1.0");
    println!("--------------------------------");

    let config_path = std::env::var("ORACLE_CONFIG")
        .unwrap_or_else(|_| "oracle/config.toml".to_string());

    let config = load_config(&config_path)?;
    println!("Config loaded from: {}", config_path);
    println!("  Stellar RPC: {}", config.stellar.rpc_url);
    println!("  Network:     {}", config.stellar.network_passphrase);
    println!(
        "  Price feed:  every {}s",
        config.price_feeds.interval_seconds
    );
    println!("  Multisig:    threshold={}", config.signing.multisig_threshold);

    let state = Arc::new(AppState::new(config.clone()));

    // Price polling loop
    let price_state = state.clone();
    let price_interval = interval(Duration::from_secs(config.price_feeds.interval_seconds));
    tokio::spawn(async move {
        price::run_price_poller(price_state, price_interval).await;
    });

    // Attestation HTTP listener (stub)
    let attest_state = state.clone();
    tokio::spawn(async move {
        attestation::run_attestation_server(attest_state).await;
    });

    println!("\nOracle sidecar is running. Press Ctrl+C to stop.\n");

    tokio::signal::ctrl_c().await?;
    println!("Shutting down...");
    Ok(())
}
