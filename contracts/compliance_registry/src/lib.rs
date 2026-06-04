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

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use privacy_passport::{PrivacyPassport, PrivacyPassportClient};
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, BytesN, Env};

    fn setup_env() -> (
        Env,
        Address,
        ComplianceRegistryClient<'static>,
        PrivacyPassportClient<'static>,
    ) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let passport_id = env.register_contract(None, PrivacyPassport);
        let passport_client = PrivacyPassportClient::new(&env, &passport_id);
        env.mock_all_auths();
        passport_client.initialize(&admin);

        let contract_id = env.register_contract(None, ComplianceRegistry);
        let client = ComplianceRegistryClient::new(&env, &contract_id);
        client.initialize(&admin, &passport_id);

        (env, admin, client, passport_client)
    }

    fn register_passport(
        env: &Env,
        passport_client: &PrivacyPassportClient<'static>,
        jurisdiction: &soroban_sdk::Symbol,
    ) -> u64 {
        passport_client.register(
            &BytesN::from_array(env, &[1u8; 32]),
            &BytesN::from_array(env, &[2u8; 32]),
            jurisdiction,
        )
    }

    #[test]
    fn test_allowlist_add_remove() {
        let (_env, _admin, client, _passport_client) = setup_env();

        let ng = symbol_short!("NG");

        assert!(!client.is_allowed(&ng));

        client.add_jurisdiction(&ng);
        assert!(client.is_allowed(&ng));

        client.remove_jurisdiction(&ng);
        assert!(!client.is_allowed(&ng));
    }

    #[test]
    fn test_verify_passport_required() {
        let (_env, _admin, client, passport_client) = setup_env();

        let ng = symbol_short!("NG");
        let us = symbol_short!("US");

        client.add_jurisdiction(&ng);

        let passport_id = register_passport(&_env, &passport_client, &ng);

        assert!(client.verify(&passport_id, &ng));
        assert!(!client.verify(&passport_id, &us));
    }

    #[test]
    fn test_verify_blocked_jurisdiction() {
        let (_env, _admin, client, passport_client) = setup_env();

        let ng = symbol_short!("NG");

        let passport_id = register_passport(&_env, &passport_client, &ng);

        assert!(!client.verify(&passport_id, &ng));
    }

    #[test]
    fn test_travel_rule_threshold() {
        let (_env, _admin, client, _passport_client) = setup_env();

        let ng = symbol_short!("NG");

        assert!(client.validate_travel_rule(&5_000_000_000i128, &ng));
        assert!(client.validate_travel_rule(&10_000_000_000i128, &ng));
        assert!(!client.validate_travel_rule(&10_000_000_001i128, &ng));
    }
}
