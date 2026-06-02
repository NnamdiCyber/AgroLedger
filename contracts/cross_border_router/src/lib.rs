#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

#[contracttype]
pub enum DataKey {
    Admin,
    ComplianceRegistry,
}

#[contract]
pub struct CrossBorderRouter;

#[contractimpl]
impl CrossBorderRouter {
    pub fn initialize(env: Env, admin: Address, compliance_registry: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ComplianceRegistry, &compliance_registry);
    }
}
