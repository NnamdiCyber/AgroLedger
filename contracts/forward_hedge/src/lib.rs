#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    CropToken,
    CollateralVault,
}

#[contract]
pub struct ForwardHedge;

#[contractimpl]
impl ForwardHedge {
    pub fn initialize(env: Env, admin: Address, crop_token: Address, collateral_vault: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CropToken, &crop_token);
        env.storage()
            .instance()
            .set(&DataKey::CollateralVault, &collateral_vault);
    }
}
