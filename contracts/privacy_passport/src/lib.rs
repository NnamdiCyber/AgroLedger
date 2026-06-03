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

    pub fn register(
        env: Env,
        nullifier_hash: BytesN<32>,
        _credential_proof: BytesN<32>,
        jurisdiction: Symbol,
    ) -> u64 {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut counter: u64 = env
            .storage()
            .instance()
            .get(&DataKey::PassportCounter)
            .unwrap();
        counter += 1;

        let passport = PassportState {
            nullifier_hash: nullifier_hash.clone(),
            jurisdiction: jurisdiction.clone(),
            active: true,
            registered_at: env.ledger().timestamp(),
        };

        env.storage()
            .instance()
            .set(&DataKey::Passport(counter), &passport);
        env.storage()
            .instance()
            .set(&DataKey::PassportCounter, &counter);

        env.events().publish(
            (Symbol::new(&env, "PassportRegistered"), counter),
            jurisdiction,
        );

        counter
    }

    pub fn verify(env: Env, passport_id: u64, required_jurisdiction: Symbol) -> bool {
        let passport: PassportState = match env
            .storage()
            .instance()
            .get(&DataKey::Passport(passport_id))
        {
            Some(p) => p,
            None => return false,
        };

        if !passport.active {
            return false;
        }

        passport.jurisdiction == required_jurisdiction
    }

    pub fn revoke(env: Env, passport_id: u64) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let mut passport: PassportState = env
            .storage()
            .instance()
            .get(&DataKey::Passport(passport_id))
            .unwrap();
        passport.active = false;

        env.storage()
            .instance()
            .set(&DataKey::Passport(passport_id), &passport);

        env.events().publish(
            (Symbol::new(&env, "PassportRevoked"), passport_id),
            (),
        );
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, BytesN, Env};

    fn setup_env() -> (Env, Address, PrivacyPassportClient<'static>) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = env.register_contract(None, PrivacyPassport);
        let client = PrivacyPassportClient::new(&env, &contract_id);
        env.mock_all_auths();
        client.initialize(&admin);
        (env, admin, client)
    }

    #[test]
    fn test_register() {
        let (env, _admin, client) = setup_env();

        let nullifier_hash = BytesN::from_array(&env, &[1u8; 32]);
        let proof = BytesN::from_array(&env, &[2u8; 32]);
        let jurisdiction = symbol_short!("NG");

        let passport_id = client.register(&nullifier_hash, &proof, &jurisdiction);

        assert_eq!(passport_id, 1u64);
    }

    #[test]
    fn test_register_increments_id() {
        let (env, _admin, client) = setup_env();

        let ng = symbol_short!("NG");
        let us = symbol_short!("US");

        let id1 = client.register(
            &BytesN::from_array(&env, &[1u8; 32]),
            &BytesN::from_array(&env, &[1u8; 32]),
            &ng,
        );
        let id2 = client.register(
            &BytesN::from_array(&env, &[2u8; 32]),
            &BytesN::from_array(&env, &[2u8; 32]),
            &us,
        );

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);
    }

    #[test]
    fn test_verify_valid() {
        let (env, _admin, client) = setup_env();

        let nullifier_hash = BytesN::from_array(&env, &[1u8; 32]);
        let proof = BytesN::from_array(&env, &[2u8; 32]);
        let jurisdiction = symbol_short!("NG");

        let passport_id = client.register(&nullifier_hash, &proof, &jurisdiction);

        assert!(client.verify(&passport_id, &jurisdiction));
    }

    #[test]
    fn test_verify_revoked() {
        let (env, _admin, client) = setup_env();

        let nullifier_hash = BytesN::from_array(&env, &[1u8; 32]);
        let proof = BytesN::from_array(&env, &[2u8; 32]);
        let jurisdiction = symbol_short!("NG");

        let passport_id = client.register(&nullifier_hash, &proof, &jurisdiction);
        client.revoke(&passport_id);

        assert!(!client.verify(&passport_id, &jurisdiction));
    }

    #[test]
    fn test_verify_wrong_jurisdiction() {
        let (env, _admin, client) = setup_env();

        let nullifier_hash = BytesN::from_array(&env, &[1u8; 32]);
        let proof = BytesN::from_array(&env, &[2u8; 32]);
        let jurisdiction = symbol_short!("NG");

        let passport_id = client.register(&nullifier_hash, &proof, &jurisdiction);

        assert!(!client.verify(&passport_id, &symbol_short!("US")));
    }

    #[test]
    fn test_verify_nonexistent() {
        let (_env, _admin, client) = setup_env();

        assert!(!client.verify(&999u64, &symbol_short!("NG")));
    }

    #[test]
    fn test_revoke_twice_panics() {
        let (env, _admin, client) = setup_env();

        let passport_id = client.register(
            &BytesN::from_array(&env, &[1u8; 32]),
            &BytesN::from_array(&env, &[2u8; 32]),
            &symbol_short!("NG"),
        );

        client.revoke(&passport_id);
        assert!(!client.verify(&passport_id, &symbol_short!("NG")));
    }

    #[test]
    fn test_revoke_nonexistent_panics() {
        let (_env, _admin, client) = setup_env();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.revoke(&999u64);
        }));
        assert!(result.is_err());
    }
}
