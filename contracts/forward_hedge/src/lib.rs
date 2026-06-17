#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, BytesN, Env, Symbol};

mod hedge;
mod settlement;

#[contracttype]
pub enum DataKey {
    Admin,
    CropToken,
    CollateralVault,
    HedgeCounter,
    Hedge(u64),
    RevealedPrice(u64),
}

#[derive(Clone)]
#[contracttype]
pub struct HedgeState {
    pub buyer: Address,
    pub farmer: Address,
    pub commodity: Symbol,
    pub quantity: i128,
    pub commitment: BytesN<32>,
    pub expiry: u64,
    pub status: Symbol,
    pub placed_at: u64,
}

#[contract]
pub struct ForwardHedge;

#[contractimpl]
impl ForwardHedge {
    pub fn initialize(env: Env, admin: Address, crop_token: Address, collateral_vault: Address) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CropToken, &crop_token);
        env.storage()
            .instance()
            .set(&DataKey::CollateralVault, &collateral_vault);
        env.storage()
            .instance()
            .set(&DataKey::HedgeCounter, &0u64);
    }

    pub fn place_hedge(
        env: Env,
        buyer: Address,
        commodity: Symbol,
        quantity: i128,
        commitment: BytesN<32>,
        expiry: u64,
    ) -> u64 {
        hedge::execute_place_hedge(env, buyer, commodity, quantity, commitment, expiry)
    }

    pub fn accept_hedge(env: Env, hedge_id: u64, farmer: Address) {
        hedge::execute_accept_hedge(env, hedge_id, farmer)
    }

    pub fn reveal(env: Env, hedge_id: u64, price: i128, salt: i128) {
        hedge::execute_reveal(env, hedge_id, price, salt)
    }

    pub fn settle(env: Env, hedge_id: u64, settlement_type: Symbol, caller: Address) {
        settlement::execute_settle(env, hedge_id, settlement_type, caller)
    }

    pub fn cancel(env: Env, hedge_id: u64, caller: Address) {
        settlement::execute_cancel(env, hedge_id, caller)
    }

    pub fn get_hedge(env: Env, hedge_id: u64) -> HedgeState {
        env.storage()
            .instance()
            .get(&DataKey::Hedge(hedge_id))
            .expect("Hedge not found")
    }

    pub fn get_revealed_price(env: Env, hedge_id: u64) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::RevealedPrice(hedge_id))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::{Bytes, Env};

    #[contracttype]
    enum MockDataKey {
        Balance(Address),
    }

    #[contract]
    struct MockToken;

    #[contractimpl]
    impl MockToken {
        pub fn balance(env: Env, id: Address) -> i128 {
            env.storage()
                .persistent()
                .get(&MockDataKey::Balance(id))
                .unwrap_or(0)
        }

        pub fn mint(env: Env, to: Address, amount: i128) {
            let bal: i128 = env
                .storage()
                .persistent()
                .get(&MockDataKey::Balance(to.clone()))
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&MockDataKey::Balance(to.clone()), &(bal + amount));
        }

        pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
            from.require_auth();
            let from_bal: i128 = env
                .storage()
                .persistent()
                .get(&MockDataKey::Balance(from.clone()))
                .unwrap_or(0);
            assert!(from_bal >= amount, "insufficient balance");
            env.storage()
                .persistent()
                .set(&MockDataKey::Balance(from.clone()), &(from_bal - amount));
            let to_bal: i128 = env
                .storage()
                .persistent()
                .get(&MockDataKey::Balance(to.clone()))
                .unwrap_or(0);
            env.storage()
                .persistent()
                .set(&MockDataKey::Balance(to.clone()), &(to_bal + amount));
        }
    }

    fn make_commitment(env: &Env, price: i128, salt: i128) -> BytesN<32> {
        let price_arr = price.to_be_bytes();
        let salt_arr = salt.to_be_bytes();
        let mut input = Bytes::new(env);
        input.append(&Bytes::from_slice(env, &price_arr));
        input.append(&Bytes::from_slice(env, &salt_arr));
        env.crypto().sha256(&input).into()
    }

    fn setup_env() -> (Env, ForwardHedgeClient<'static>, Address, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let admin = Address::generate(&env);
        let buyer = Address::generate(&env);
        let farmer = Address::generate(&env);

        let crop_id = env.register_contract(None, MockToken);
        let crop_client = MockTokenClient::new(&env, &crop_id);

        let vault_id = Address::generate(&env);

        let hedge_id = env.register_contract(None, ForwardHedge);
        let hedge_client = ForwardHedgeClient::new(&env, &hedge_id);

        hedge_client.initialize(&admin, &crop_id, &vault_id);

        crop_client.mint(&farmer, &1_000_000_000i128);
        crop_client.mint(&buyer, &1_000_000_000i128);

        (env, hedge_client, admin, buyer, farmer, crop_id)
    }

    #[test]
    fn test_place_hedge() {
        let (env, hedge, _, buyer, _farmer, _) = setup_env();

        let price: i128 = 500_000_000;
        let salt: i128 = 12345;
        let commitment = make_commitment(&env, price, salt);

        let hedge_id = hedge.place_hedge(
            &buyer,
            &Symbol::new(&env, "Maize"),
            &1000i128,
            &commitment,
            &2_000_000u64,
        );

        assert_eq!(hedge_id, 1u64);

        let state = hedge.get_hedge(&hedge_id);
        assert_eq!(state.buyer, buyer);
        assert_eq!(state.commodity, Symbol::new(&env, "Maize"));
        assert_eq!(state.quantity, 1000);
        assert_eq!(state.commitment, commitment);
        assert_eq!(state.expiry, 2_000_000);
        assert_eq!(state.status, Symbol::new(&env, "Placed"));
    }

    #[test]
    fn test_accept_hedge() {
        let (env, hedge, _, buyer, farmer, _) = setup_env();

        let price: i128 = 500_000_000;
        let salt: i128 = 12345;
        let commitment = make_commitment(&env, price, salt);

        let hedge_id = hedge.place_hedge(
            &buyer,
            &Symbol::new(&env, "Maize"),
            &1000i128,
            &commitment,
            &2_000_000u64,
        );

        hedge.accept_hedge(&hedge_id, &farmer);

        let state = hedge.get_hedge(&hedge_id);
        assert_eq!(state.farmer, farmer);
        assert_eq!(state.status, Symbol::new(&env, "Accepted"));
    }

    #[test]
    fn test_settle_physical() {
        let (env, hedge, _, buyer, farmer, crop_id) = setup_env();

        let price: i128 = 500_000_000;
        let salt: i128 = 12345;
        let commitment = make_commitment(&env, price, salt);

        let hedge_id = hedge.place_hedge(
            &buyer,
            &Symbol::new(&env, "Maize"),
            &1000i128,
            &commitment,
            &2_000_000u64,
        );

        hedge.accept_hedge(&hedge_id, &farmer);
        hedge.reveal(&hedge_id, &price, &salt);

        env.ledger().set_timestamp(2_000_001);

        let farmer_bal_before = MockTokenClient::new(&env, &crop_id).balance(&farmer);
        let buyer_bal_before = MockTokenClient::new(&env, &crop_id).balance(&buyer);

        hedge.settle(&hedge_id, &Symbol::new(&env, "Physical"), &farmer);

        let farmer_bal_after = MockTokenClient::new(&env, &crop_id).balance(&farmer);
        let buyer_bal_after = MockTokenClient::new(&env, &crop_id).balance(&buyer);

        assert_eq!(farmer_bal_before - farmer_bal_after, 1000);
        assert_eq!(buyer_bal_after - buyer_bal_before, 1000);

        let state = hedge.get_hedge(&hedge_id);
        assert_eq!(state.status, Symbol::new(&env, "SettledPhysical"));
    }

    #[test]
    fn test_settle_usdc() {
        let (env, hedge, _, buyer, farmer, crop_id) = setup_env();

        let price: i128 = 500_000_000;
        let salt: i128 = 12345;
        let commitment = make_commitment(&env, price, salt);

        let hedge_id = hedge.place_hedge(
            &buyer,
            &Symbol::new(&env, "Maize"),
            &1000i128,
            &commitment,
            &2_000_000u64,
        );

        hedge.accept_hedge(&hedge_id, &farmer);
        hedge.reveal(&hedge_id, &price, &salt);

        env.ledger().set_timestamp(2_000_001);

        let buyer_bal_before = MockTokenClient::new(&env, &crop_id).balance(&buyer);
        let farmer_bal_before = MockTokenClient::new(&env, &crop_id).balance(&farmer);

        hedge.settle(&hedge_id, &Symbol::new(&env, "Cash"), &buyer);

        let expected = 1000 * 500_000_000 / 1_000_000_000i128;

        let buyer_bal_after = MockTokenClient::new(&env, &crop_id).balance(&buyer);
        let farmer_bal_after = MockTokenClient::new(&env, &crop_id).balance(&farmer);

        assert_eq!(buyer_bal_before - buyer_bal_after, expected);
        assert_eq!(farmer_bal_after - farmer_bal_before, expected);

        let state = hedge.get_hedge(&hedge_id);
        assert_eq!(state.status, Symbol::new(&env, "SettledCash"));
    }

    #[test]
    fn test_cancel_before_expiry() {
        let (env, hedge, _, buyer, _farmer, _) = setup_env();

        let price: i128 = 500_000_000;
        let salt: i128 = 12345;
        let commitment = make_commitment(&env, price, salt);

        let hedge_id = hedge.place_hedge(
            &buyer,
            &Symbol::new(&env, "Maize"),
            &1000i128,
            &commitment,
            &2_000_000u64,
        );

        hedge.cancel(&hedge_id, &buyer);

        let state = hedge.get_hedge(&hedge_id);
        assert_eq!(state.status, Symbol::new(&env, "Cancelled"));
    }

    #[test]
    fn test_reveal_mismatch_panics() {
        let (env, hedge, _, buyer, farmer, _) = setup_env();

        let price: i128 = 500_000_000;
        let salt: i128 = 12345;
        let commitment = make_commitment(&env, price, salt);

        let hedge_id = hedge.place_hedge(
            &buyer,
            &Symbol::new(&env, "Maize"),
            &1000i128,
            &commitment,
            &2_000_000u64,
        );

        hedge.accept_hedge(&hedge_id, &farmer);

        let wrong_price: i128 = 600_000_000;
        let wrong_salt: i128 = 99999;

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            hedge.reveal(&hedge_id, &wrong_price, &wrong_salt);
        }));
        assert!(result.is_err(), "Reveal with mismatched price should panic");
    }
}
