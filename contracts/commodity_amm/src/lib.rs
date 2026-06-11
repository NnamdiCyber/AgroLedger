#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

mod curve;
mod pool;

#[contracttype]
pub enum DataKey {
    Admin,
    CropToken,
    UsdcToken,
    PoolInfo(Symbol),
    BalanceLP(Address, Symbol),
    TotalLpSupply(Symbol),
}

#[derive(Clone)]
#[contracttype]
pub struct PoolInfo {
    pub commodity: Symbol,
    pub reserve_crop: i128,
    pub reserve_usdc: i128,
    pub total_lp_supply: i128,
    pub created_at: u64,
}

#[contract]
pub struct CommodityAmm;

#[contractimpl]
impl CommodityAmm {
    pub fn initialize(env: Env, admin: Address, crop_token: Address, usdc_token: Address) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CropToken, &crop_token);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
    }

    pub fn create_pool(env: Env, admin: Address, commodity: Symbol) {
        admin.require_auth();
        let stored_admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        assert_eq!(admin, stored_admin, "Only admin can create pools");
        pool::execute_create_pool(&env, commodity);
    }

    pub fn swap(
        env: Env,
        user: Address,
        commodity: Symbol,
        amount_in: i128,
        min_amount_out: i128,
        sell_crop: bool,
    ) -> i128 {
        pool::execute_swap(&env, user, commodity, amount_in, min_amount_out, sell_crop)
    }

    pub fn add_liquidity(
        env: Env,
        user: Address,
        commodity: Symbol,
        amount_crop: i128,
        amount_usdc: i128,
    ) -> (i128, i128, i128) {
        pool::execute_add_liquidity(&env, user, commodity, amount_crop, amount_usdc)
    }

    pub fn remove_liquidity(
        env: Env,
        user: Address,
        commodity: Symbol,
        lp_tokens: i128,
        min_crop: i128,
        min_usdc: i128,
    ) -> (i128, i128) {
        pool::execute_remove_liquidity(&env, user, commodity, lp_tokens, min_crop, min_usdc)
    }

    pub fn get_pool(env: Env, commodity: Symbol) -> PoolInfo {
        env.storage()
            .instance()
            .get(&DataKey::PoolInfo(commodity))
            .unwrap()
    }

    pub fn get_lp_balance(env: Env, user: Address, commodity: Symbol) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::BalanceLP(user, commodity))
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, Env};

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

    fn setup_env() -> (
        Env,
        Address,
        CommodityAmmClient<'static>,
        Address,
        Address,
        MockTokenClient<'static>,
        MockTokenClient<'static>,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let crop_id = env.register_contract(None, MockToken);
        let crop_client = MockTokenClient::new(&env, &crop_id);

        let usdc_id = env.register_contract(None, MockToken);
        let usdc_client = MockTokenClient::new(&env, &usdc_id);

        let amm_id = env.register_contract(None, CommodityAmm);
        let amm_client = CommodityAmmClient::new(&env, &amm_id);

        amm_client.initialize(&admin, &crop_id, &usdc_id);

        (env, admin, amm_client, amm_id, crop_id, usdc_client, crop_client)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let crop_id = env.register_contract(None, MockToken);
        let usdc_id = env.register_contract(None, MockToken);
        let amm_id = env.register_contract(None, CommodityAmm);
        let client = CommodityAmmClient::new(&env, &amm_id);

        client.initialize(&admin, &crop_id, &usdc_id);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin, &crop_id, &usdc_id);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_create_pool() {
        let (_env, admin, amm, _amm_id, _crop, _usdc, _crop_cl) = setup_env();
        amm.create_pool(&admin, &symbol_short!("MAIZE"));

        let pool = amm.get_pool(&symbol_short!("MAIZE"));
        assert_eq!(pool.commodity, symbol_short!("MAIZE"));
        assert_eq!(pool.reserve_crop, 0);
        assert_eq!(pool.reserve_usdc, 0);
    }

    #[test]
    fn test_create_duplicate_pool_fails() {
        let (_env, admin, amm, _amm_id, _crop, _usdc, _crop_cl) = setup_env();
        amm.create_pool(&admin, &symbol_short!("MAIZE"));

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            amm.create_pool(&admin, &symbol_short!("MAIZE"));
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_add_remove_liquidity() {
        let (_env, admin, amm, amm_id, _crop, usdc_client, crop_client) = setup_env();

        amm.create_pool(&admin, &symbol_short!("MAIZE"));

        crop_client.mint(&admin, &100_000i128);
        usdc_client.mint(&admin, &50_000i128);

        let (crop_deposited, usdc_deposited, lp_tokens) = amm.add_liquidity(
            &admin,
            &symbol_short!("MAIZE"),
            &100_000i128,
            &50_000i128,
        );

        assert_eq!(crop_deposited, 100_000);
        assert_eq!(usdc_deposited, 50_000);
        assert!(lp_tokens > 0);

        let pool = amm.get_pool(&symbol_short!("MAIZE"));
        assert_eq!(pool.reserve_crop, 100_000);
        assert_eq!(pool.reserve_usdc, 50_000);

        let balance = amm.get_lp_balance(&admin, &symbol_short!("MAIZE"));
        assert_eq!(balance, lp_tokens);

        assert_eq!(crop_client.balance(&admin), 0);
        assert_eq!(crop_client.balance(&amm_id), 100_000);
        assert_eq!(usdc_client.balance(&admin), 0);
        assert_eq!(usdc_client.balance(&amm_id), 50_000);

        let (crop_out, usdc_out) = amm.remove_liquidity(
            &admin,
            &symbol_short!("MAIZE"),
            &lp_tokens,
            &0i128,
            &0i128,
        );

        assert_eq!(crop_out, 100_000);
        assert_eq!(usdc_out, 50_000);

        let pool = amm.get_pool(&symbol_short!("MAIZE"));
        assert_eq!(pool.reserve_crop, 0);
        assert_eq!(pool.reserve_usdc, 0);

        assert_eq!(crop_client.balance(&admin), 100_000);
        assert_eq!(crop_client.balance(&amm_id), 0);
        assert_eq!(usdc_client.balance(&admin), 50_000);
        assert_eq!(usdc_client.balance(&amm_id), 0);
    }

    #[test]
    fn test_swap_basic() {
        let (env, admin, amm, _amm_id, _crop, usdc_client, crop_client) = setup_env();

        amm.create_pool(&admin, &symbol_short!("MAIZE"));

        crop_client.mint(&admin, &200_000i128);
        usdc_client.mint(&admin, &100_000i128);

        amm.add_liquidity(&admin, &symbol_short!("MAIZE"), &200_000i128, &100_000i128);

        let user = Address::generate(&env);
        crop_client.mint(&user, &10_000i128);

        let amount_out = amm.swap(
            &user,
            &symbol_short!("MAIZE"),
            &10_000i128,
            &0i128,
            &true,
        );

        assert!(amount_out > 0);
        assert!(amount_out < 10_000);
    }

    #[test]
    fn test_swap_price_impact() {
        let (env, admin, amm, _amm_id, _crop, usdc_client, crop_client) = setup_env();

        amm.create_pool(&admin, &symbol_short!("MAIZE"));

        crop_client.mint(&admin, &1_000_000i128);
        usdc_client.mint(&admin, &500_000i128);

        amm.add_liquidity(&admin, &symbol_short!("MAIZE"), &1_000_000i128, &500_000i128);

        let user = Address::generate(&env);
        crop_client.mint(&user, &100_000i128);

        let amount_out = amm.swap(
            &user,
            &symbol_short!("MAIZE"),
            &100_000i128,
            &0i128,
            &true,
        );

        let expected_spot = 500_000i128 * 100_000i128 / 1_000_000i128;
        let expected_fee = expected_spot * 9970 / 10000;
        assert!(amount_out < expected_fee, "Price impact should reduce output");
    }

    #[test]
    fn test_seasonal_curve_variance() {
        let env = Env::default();

        let jan_timestamp = 0u64;
        let dec_timestamp = 86400 * 364;

        let jan_out = crate::curve::calculate_swap(
            &env, 10_000, 1_000_000, 500_000, true, jan_timestamp,
        );
        let dec_out = crate::curve::calculate_swap(
            &env, 10_000, 1_000_000, 500_000, true, dec_timestamp,
        );

        assert!(
            dec_out > jan_out,
            "Post-harvest (Dec) should give more output than pre-harvest (Jan)"
        );
    }

    #[test]
    fn test_swap_slippage_protection() {
        let (env, admin, amm, _amm_id, _crop, usdc_client, crop_client) = setup_env();

        amm.create_pool(&admin, &symbol_short!("MAIZE"));

        crop_client.mint(&admin, &200_000i128);
        usdc_client.mint(&admin, &100_000i128);

        amm.add_liquidity(&admin, &symbol_short!("MAIZE"), &200_000i128, &100_000i128);

        let user = Address::generate(&env);
        crop_client.mint(&user, &10_000i128);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            amm.swap(&user, &symbol_short!("MAIZE"), &10_000i128, &10_000i128, &true);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_swap_reverse_direction() {
        let (env, admin, amm, _amm_id, _crop, usdc_client, crop_client) = setup_env();

        amm.create_pool(&admin, &symbol_short!("MAIZE"));

        crop_client.mint(&admin, &200_000i128);
        usdc_client.mint(&admin, &100_000i128);

        amm.add_liquidity(&admin, &symbol_short!("MAIZE"), &200_000i128, &100_000i128);

        let user = Address::generate(&env);
        usdc_client.mint(&user, &5_000i128);

        let amount_out = amm.swap(
            &user,
            &symbol_short!("MAIZE"),
            &5_000i128,
            &0i128,
            &false,
        );

        assert!(amount_out > 0);
        assert!(amount_out < 10_000);
    }

    #[test]
    fn test_multiple_pools() {
        let (_env, admin, amm, _amm_id, _crop, usdc_client, crop_client) = setup_env();

        amm.create_pool(&admin, &symbol_short!("MAIZE"));
        amm.create_pool(&admin, &symbol_short!("SOYA"));

        crop_client.mint(&admin, &300_000i128);
        usdc_client.mint(&admin, &150_000i128);

        amm.add_liquidity(&admin, &symbol_short!("MAIZE"), &200_000i128, &100_000i128);
        amm.add_liquidity(&admin, &symbol_short!("SOYA"), &100_000i128, &50_000i128);

        let pool1 = amm.get_pool(&symbol_short!("MAIZE"));
        let pool2 = amm.get_pool(&symbol_short!("SOYA"));

        assert_eq!(pool1.reserve_crop, 200_000);
        assert_eq!(pool2.reserve_crop, 100_000);
        assert_eq!(pool1.reserve_usdc, 100_000);
        assert_eq!(pool2.reserve_usdc, 50_000);
    }
}
