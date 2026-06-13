#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env};

mod vault;

#[contracttype]
pub enum DataKey {
    Admin,
    CropToken,
    CommodityAmm,
    UsdcToken,
    TotalCropDeposited,
    TotalHctSupply,
    TotalYieldUsdc,
    BalanceHct(Address),
    LastAccrual,
}

#[contract]
pub struct HarvestVault;

#[contractimpl]
impl HarvestVault {
    pub fn initialize(
        env: Env,
        admin: Address,
        crop_token: Address,
        commodity_amm: Address,
        usdc_token: Address,
    ) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(&DataKey::CropToken, &crop_token);
        env.storage()
            .instance()
            .set(&DataKey::CommodityAmm, &commodity_amm);
        env.storage().instance().set(&DataKey::UsdcToken, &usdc_token);
        env.storage()
            .instance()
            .set(&DataKey::TotalCropDeposited, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalHctSupply, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::TotalYieldUsdc, &0i128);
        env.storage()
            .instance()
            .set(&DataKey::LastAccrual, &env.ledger().timestamp());
    }

    pub fn deposit(env: Env, user: Address, amount: i128) -> i128 {
        vault::execute_deposit(env, user, amount)
    }

    pub fn withdraw(env: Env, user: Address, hct_amount: i128) -> (i128, i128) {
        vault::execute_withdraw(env, user, hct_amount)
    }

    pub fn get_hct_balance(env: Env, user: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&DataKey::BalanceHct(user))
            .unwrap_or(0)
    }

    pub fn get_total_crop_deposited(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalCropDeposited)
            .unwrap_or(0)
    }

    pub fn get_total_hct_supply(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalHctSupply)
            .unwrap_or(0)
    }

    pub fn get_total_yield(env: Env) -> i128 {
        env.storage()
            .instance()
            .get(&DataKey::TotalYieldUsdc)
            .unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};
    use soroban_sdk::Env;

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

    fn setup_env() -> (Env, Address, HarvestVaultClient<'static>, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);

        let admin = Address::generate(&env);

        let crop_id = env.register_contract(None, MockToken);
        let _crop_client = MockTokenClient::new(&env, &crop_id);

        let usdc_id = env.register_contract(None, MockToken);
        let usdc_client = MockTokenClient::new(&env, &usdc_id);

        let amm_id = Address::generate(&env);

        let vault_id = env.register_contract(None, HarvestVault);
        let vault_client = HarvestVaultClient::new(&env, &vault_id);

        vault_client.initialize(&admin, &crop_id, &amm_id, &usdc_id);

        // Fund vault with USDC for yield payouts
        usdc_client.mint(&vault_id, &10_000_000i128);

        (env, admin, vault_client, vault_id, crop_id, usdc_id)
    }

    #[test]
    fn test_initialize() {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_000_000);
        let admin = Address::generate(&env);
        let crop_id = Address::generate(&env);
        let amm_id = Address::generate(&env);
        let usdc_id = Address::generate(&env);
        let vault_id = env.register_contract(None, HarvestVault);
        let client = HarvestVaultClient::new(&env, &vault_id);

        client.initialize(&admin, &crop_id, &amm_id, &usdc_id);

        assert_eq!(client.get_total_crop_deposited(), 0);
        assert_eq!(client.get_total_hct_supply(), 0);
        assert_eq!(client.get_total_yield(), 0);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin, &crop_id, &amm_id, &usdc_id);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_deposit_withdraw() {
        let (env, admin, vault, _vault_id, crop, _usdc) = setup_env();
        let crop_client = MockTokenClient::new(&env, &crop);

        crop_client.mint(&admin, &100_000i128);

        let hct_received = vault.deposit(&admin, &50_000i128);
        assert_eq!(hct_received, 50_000);
        assert_eq!(vault.get_hct_balance(&admin), 50_000);
        assert_eq!(vault.get_total_crop_deposited(), 50_000);
        assert_eq!(vault.get_total_hct_supply(), 50_000);

        let (crop_out, yield_out) = vault.withdraw(&admin, &50_000i128);
        assert_eq!(crop_out, 50_000);
        assert_eq!(yield_out, 0);
        assert_eq!(vault.get_hct_balance(&admin), 0);
        assert_eq!(vault.get_total_crop_deposited(), 0);
        assert_eq!(vault.get_total_hct_supply(), 0);
    }

    #[test]
    fn test_multiple_depositors() {
        let (env, admin, vault, _vault_id, crop, _usdc) = setup_env();
        let crop_client = MockTokenClient::new(&env, &crop);

        let user1 = Address::generate(&env);
        let user2 = Address::generate(&env);

        crop_client.mint(&admin, &200_000i128);

        // First user deposits, establishing 1:1 ratio
        crop_client.transfer(&admin, &user1, &100_000i128);
        crop_client.transfer(&admin, &user2, &100_000i128);

        vault.deposit(&user1, &60_000i128);
        vault.deposit(&user2, &40_000i128);

        assert_eq!(vault.get_hct_balance(&user1), 60_000);
        assert_eq!(vault.get_hct_balance(&user2), 40_000);
        assert_eq!(vault.get_total_crop_deposited(), 100_000);
        assert_eq!(vault.get_total_hct_supply(), 100_000);

        // User1 withdraws all
        let (crop_out, yield_out) = vault.withdraw(&user1, &60_000i128);
        assert_eq!(crop_out, 60_000);
        assert_eq!(yield_out, 0);

        // User2 still has their share
        assert_eq!(vault.get_hct_balance(&user2), 40_000);
        assert_eq!(vault.get_total_crop_deposited(), 40_000);
        assert_eq!(vault.get_total_hct_supply(), 40_000);

        // User2 withdraws all
        let (crop_out2, _) = vault.withdraw(&user2, &40_000i128);
        assert_eq!(crop_out2, 40_000);
        assert_eq!(vault.get_total_crop_deposited(), 0);
    }

    #[test]
    fn test_deposit_zero_fails() {
        let (env, admin, vault, _vault_id, crop, _usdc) = setup_env();
        let crop_client = MockTokenClient::new(&env, &crop);

        crop_client.mint(&admin, &100_000i128);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            vault.deposit(&admin, &0i128);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_withdraw_insufficient_hct_fails() {
        let (env, admin, vault, _vault_id, crop, _usdc) = setup_env();
        let crop_client = MockTokenClient::new(&env, &crop);

        crop_client.mint(&admin, &100_000i128);
        vault.deposit(&admin, &50_000i128);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            vault.withdraw(&admin, &100_000i128);
        }));
        assert!(result.is_err());
    }
}
