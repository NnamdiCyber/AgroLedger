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
