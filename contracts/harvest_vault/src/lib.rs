#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    CropToken,
    CommodityAmm,
    UsdcToken,
}

#[contract]
pub struct HarvestVault;

#[contractimpl]
impl HarvestVault {
    pub fn initialize(
        env: Env,
        admin: Address,
        crop_token: Address,
        commodity_amm: Address,
        usdc_token: Address,
    ) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CropToken, &crop_token);
        env.storage()
            .instance()
            .set(&DataKey::CommodityAmm, &commodity_amm);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
    }
}
