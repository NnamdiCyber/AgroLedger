use soroban_sdk::{contracttype, Address, Env, IntoVal, Symbol, Val, Vec};

use crate::DataKey;

#[derive(Clone)]
#[contracttype]
pub struct PriceData {
    pub price: u64,
    pub timestamp: u64,
}

pub fn compute_ltv(env: &Env, crop_token_amount: i128, commodity: &Symbol, debt_usdc: i128) -> u32 {
    if crop_token_amount <= 0 || debt_usdc <= 0 {
        return 0;
    }

    let oracle: Address = env
        .storage()
        .instance()
        .get(&DataKey::WarehouseOracle)
        .expect("WarehouseOracle not set");

    let args: Vec<Val> = (commodity.clone(),).into_val(env);
    let price_data: PriceData =
        env.invoke_contract(&oracle, &Symbol::new(env, "get_price"), args);

    if price_data.price == 0 {
        return 0;
    }

    let collateral_value = crop_token_amount * (price_data.price as i128);
    if collateral_value <= 0 {
        return 0;
    }

    // LTV as percentage (0-100)
    ((debt_usdc * 100) / collateral_value) as u32
}
