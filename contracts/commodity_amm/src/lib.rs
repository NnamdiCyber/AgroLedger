#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    CropToken,
}

#[contract]
pub struct CommodityAmm;

#[contractimpl]
impl CommodityAmm {
    pub fn initialize(env: Env, admin: Address, crop_token: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CropToken, &crop_token);
    }
}
