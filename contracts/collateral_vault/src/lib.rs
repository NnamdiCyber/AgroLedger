#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    ComplianceRegistry,
    UsdcToken,
    WarehouseOracle,
    VaultCounter,
}

#[contract]
pub struct CollateralVault;

#[contractimpl]
impl CollateralVault {
    pub fn initialize(
        env: Env,
        admin: Address,
        compliance_registry: Address,
        usdc_token: Address,
        warehouse_oracle: Address,
    ) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ComplianceRegistry, &compliance_registry);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage()
            .instance()
            .set(&DataKey::WarehouseOracle, &warehouse_oracle);
        env.storage().instance().set(&DataKey::VaultCounter, &0u64);
    }
}
