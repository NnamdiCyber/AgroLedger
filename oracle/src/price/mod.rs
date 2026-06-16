use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use tokio::time::Interval;

use crate::{ApiConfig, AppState};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceQuote {
    pub commodity: String,
    pub price_usdc: u64,
    pub source: String,
    pub timestamp: u64,
}

#[async_trait]
pub trait PriceFeedProvider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn fetch_price(&self, commodity: &str) -> Option<PriceQuote>;
}

pub struct AfexFeed {
    config: ApiConfig,
}

#[async_trait]
impl PriceFeedProvider for AfexFeed {
    fn name(&self) -> &'static str {
        "AFEX"
    }

    async fn fetch_price(&self, commodity: &str) -> Option<PriceQuote> {
        if !self.config.enabled {
            return None;
        }
        // Stub: return simulated price
        let price = match commodity {
            "MAIZE" => 250_000_000u64,
            "SOYA" => 450_000_000,
            "COCOA" => 2_500_000_000,
            "RICE" => 600_000_000,
            "COTTON" => 1_200_000_000,
            "COFFEE" => 3_000_000_000,
            _ => return None,
        };
        Some(PriceQuote {
            commodity: commodity.to_string(),
            price_usdc: price,
            source: "AFEX".to_string(),
            timestamp: Utc::now().timestamp() as u64,
        })
    }
}

pub struct GcxFeed;

#[async_trait]
impl PriceFeedProvider for GcxFeed {
    fn name(&self) -> &'static str {
        "GCX"
    }

    async fn fetch_price(&self, commodity: &str) -> Option<PriceQuote> {
        // Stub
        let price = match commodity {
            "MAIZE" => 245_000_000u64,
            "SOYA" => 440_000_000,
            "RICE" => 590_000_000,
            _ => return None,
        };
        Some(PriceQuote {
            commodity: commodity.to_string(),
            price_usdc: price,
            source: "GCX".to_string(),
            timestamp: Utc::now().timestamp() as u64,
        })
    }
}

pub struct CmeFeed;

#[async_trait]
impl PriceFeedProvider for CmeFeed {
    fn name(&self) -> &'static str {
        "CME"
    }

    async fn fetch_price(&self, commodity: &str) -> Option<PriceQuote> {
        // Stub
        let price = match commodity {
            "MAIZE" => 260_000_000u64,
            "SOYA" => 460_000_000,
            "COCOA" => 2_550_000_000,
            "COTTON" => 1_180_000_000,
            "COFFEE" => 3_050_000_000,
            _ => return None,
        };
        Some(PriceQuote {
            commodity: commodity.to_string(),
            price_usdc: price,
            source: "CME".to_string(),
            timestamp: Utc::now().timestamp() as u64,
        })
    }
}

fn select_best_price(quotes: &[PriceQuote]) -> Option<PriceQuote> {
    quotes.iter().min_by_key(|q| q.price_usdc).cloned()
}

pub async fn run_price_poller(state: Arc<AppState>, mut tick: Interval) {
    let feeds: Vec<Box<dyn PriceFeedProvider>> = vec![
        Box::new(AfexFeed {
            config: state.config.price_feeds.afex.clone(),
        }),
        Box::new(GcxFeed),
        Box::new(CmeFeed),
    ];

    let commodities = ["MAIZE", "SOYA", "COCOA", "RICE", "COTTON", "COFFEE"];

    loop {
        tick.tick().await;

        for commodity in &commodities {
            let mut quotes = Vec::new();

            for feed in &feeds {
                if let Some(quote) = feed.fetch_price(commodity).await {
                    println!(
                        "[price] {}: {} = {} (from {})",
                        Utc::now().format("%H:%M:%S"),
                        commodity,
                        quote.price_usdc,
                        feed.name()
                    );
                    quotes.push(quote);
                }
            }

            if let Some(best) = select_best_price(&quotes) {
                let mut prices = state.last_prices.lock().await;
                prices.insert(commodity.to_string(), best);
            }
        }
    }
}
