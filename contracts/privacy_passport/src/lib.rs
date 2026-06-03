#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

mod passport;
mod revocation;

#[contracttype]
pub enum DataKey {
    Admin,
    PassportCounter,
    Passport(u64),
}

#[derive(Clone)]
#[contracttype]
pub struct PassportState {
    pub nullifier_hash: BytesN<32>,
    pub jurisdiction: Symbol,
    pub active: bool,
    pub registered_at: u64,
}

#[contract]
pub struct PrivacyPassport;

#[contractimpl]
impl PrivacyPassport {
    pub fn initialize(env: Env, admin: Address) {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PassportCounter, &0u64);
    }
}
