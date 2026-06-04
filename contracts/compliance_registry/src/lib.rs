#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, IntoVal, Symbol, Val, Vec};

mod allowlist;
mod fatf;

#[contracttype]
pub enum DataKey {
    Admin,
    PrivacyPassport,
    AllowedJurisdictions,
}

#[contract]
pub struct ComplianceRegistry;

#[contractimpl]
impl ComplianceRegistry {
    pub fn initialize(env: Env, admin: Address, privacy_passport: Address) {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::PrivacyPassport, &privacy_passport);
    }

    pub fn add_jurisdiction(env: Env, code: Symbol) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        allowlist::add_jurisdiction(&env, &code);
    }

    pub fn remove_jurisdiction(env: Env, code: Symbol) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        allowlist::remove_jurisdiction(&env, &code);
    }

    pub fn is_allowed(env: Env, jurisdiction: Symbol) -> bool {
        allowlist::is_allowed(&env, &jurisdiction)
    }

    pub fn verify(env: Env, passport_id: u64, jurisdiction: Symbol) -> bool {
        if !allowlist::is_allowed(&env, &jurisdiction) {
            env.events().publish(
                (Symbol::new(&env, "ComplianceCheck"), passport_id),
                jurisdiction,
            );
            return false;
        }

        let passport_addr: Address = env
            .storage()
            .instance()
            .get(&DataKey::PrivacyPassport)
            .unwrap();

        let args: Vec<Val> = (passport_id, jurisdiction.clone()).into_val(&env);
        let valid: bool = env.invoke_contract(
            &passport_addr,
            &Symbol::new(&env, "verify"),
            args,
        );

        env.events().publish(
            (Symbol::new(&env, "ComplianceCheck"), passport_id),
            jurisdiction,
        );

        valid
    }

    pub fn validate_travel_rule(_env: Env, amount: i128, _jurisdiction: Symbol) -> bool {
        fatf::validate_travel_rule(amount)
    }
}
