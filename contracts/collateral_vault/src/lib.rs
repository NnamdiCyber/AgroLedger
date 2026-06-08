#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol};

mod ltv;
mod vault;

pub use crate::ltv::PriceData;

#[contracttype]
pub enum DataKey {
    Admin,
    ComplianceRegistry,
    UsdcToken,
    WarehouseOracle,
    VaultCounter,
    Vault(u64),
}

#[derive(Clone)]
#[contracttype]
pub struct VaultState {
    pub owner: Address,
    pub crop_token: Address,
    pub collateral_amount: i128,
    pub debt_amount: i128,
    pub commodity: Symbol,
    pub opened_at: u64,
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

    pub fn open(
        env: Env,
        user: Address,
        crop_token: Address,
        commodity: Symbol,
        passport_id: u64,
        jurisdiction: Symbol,
        collateral_amount: i128,
        borrow_amount_usdc: i128,
    ) -> u64 {
        vault::execute_open(
            env,
            user,
            crop_token,
            commodity,
            passport_id,
            jurisdiction,
            collateral_amount,
            borrow_amount_usdc,
        )
    }

    pub fn repay(env: Env, user: Address, vault_id: u64, amount: i128) {
        vault::execute_repay(env, user, vault_id, amount)
    }

    pub fn liquidate(env: Env, liquidator: Address, vault_id: u64) {
        vault::execute_liquidate(env, liquidator, vault_id)
    }

    pub fn get_vault(env: Env, vault_id: u64) -> VaultState {
        env.storage()
            .instance()
            .get(&DataKey::Vault(vault_id))
            .unwrap()
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, BytesN, Env, Vec};

    use compliance_registry::{ComplianceRegistry, ComplianceRegistryClient};
    use privacy_passport::{PrivacyPassport, PrivacyPassportClient};
    use warehouse_oracle::{WarehouseOracle, WarehouseOracleClient, InspectorSet};

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
        CollateralVaultClient<'static>,
        Address,
        Address,
        Address,
        ComplianceRegistryClient<'static>,
        WarehouseOracleClient<'static>,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        // Setup PrivacyPassport
        let passport_id = env.register_contract(None, PrivacyPassport);
        let passport_client = PrivacyPassportClient::new(&env, &passport_id);
        passport_client.initialize(&admin);

        // Setup ComplianceRegistry
        let registry_id = env.register_contract(None, ComplianceRegistry);
        let registry_client = ComplianceRegistryClient::new(&env, &registry_id);
        registry_client.initialize(&admin, &passport_id);
        registry_client.add_jurisdiction(&symbol_short!("NG"));

        let nullifier = BytesN::from_array(&env, &[1u8; 32]);
        let proof = BytesN::from_array(&env, &[2u8; 32]);
        passport_client.register(&nullifier, &proof, &symbol_short!("NG"));

        // Setup WarehouseOracle
        let oracle_id = env.register_contract(None, WarehouseOracle);
        let oracle_client = WarehouseOracleClient::new(&env, &oracle_id);
        let oracle_pubkey = Address::generate(&env);
        let inspectors = InspectorSet {
            inspectors: Vec::new(&env),
            threshold: 0,
        };
        oracle_client.initialize(&admin, &oracle_pubkey, &inspectors);

        // Push a price
        oracle_client.push_price(&symbol_short!("MAIZE"), &10_000_000u64, &1_000_000u64);

        // Setup Mock USDC
        let usdc_id = env.register_contract(None, MockToken);
        let usdc_client = MockTokenClient::new(&env, &usdc_id);

        // Setup Mock CropToken
        let crop_id = env.register_contract(None, MockToken);
        let _crop_client = MockTokenClient::new(&env, &crop_id);

        // Deploy vault
        let vault_id = env.register_contract(None, CollateralVault);
        let vault_client = CollateralVaultClient::new(&env, &vault_id);
        vault_client.initialize(&admin, &registry_id, &usdc_id, &oracle_id);

        // Fund vault with USDC for lending
        usdc_client.mint(&vault_id, &1_000_000_000_000i128);

        (
            env, admin, vault_client, vault_id, usdc_id, crop_id, registry_client, oracle_client,
        )
    }

    #[test]
    fn test_open_vault() {
        let (env, admin, vault, vault_addr, _usdc, crop, _registry, _oracle) = setup_env();

        let crop_client = MockTokenClient::new(&env, &crop);
        crop_client.mint(&admin, &100_000i128);

        let vault_id = vault.open(
            &admin,
            &crop,
            &symbol_short!("MAIZE"),
            &1u64,
            &symbol_short!("NG"),
            &50_000i128,
            &100_000i128,
        );

        let state = vault.get_vault(&vault_id);
        assert_eq!(state.owner, admin);
        assert_eq!(state.collateral_amount, 50_000);
        assert_eq!(state.debt_amount, 100_000);
        assert_eq!(state.commodity, symbol_short!("MAIZE"));

        // Collateral should be locked in vault
        assert_eq!(crop_client.balance(&admin), 50_000);
        assert_eq!(crop_client.balance(&vault_addr), 50_000);
    }

    #[test]
    fn test_compliance_reverts() {
        let (env, admin, vault, _vault_addr, _usdc, crop, _registry, _oracle) = setup_env();

        let crop_client = MockTokenClient::new(&env, &crop);
        crop_client.mint(&admin, &100_000i128);

        // Use a jurisdiction that's not allowed
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            vault.open(
                &admin,
                &crop,
                &symbol_short!("MAIZE"),
                &1u64,
                &symbol_short!("US"),
                &50_000i128,
                &100_000i128,
            );
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_repay_full() {
        let (env, admin, vault, vault_addr, usdc, crop, _registry, _oracle) = setup_env();

        let usdc_client = MockTokenClient::new(&env, &usdc);
        let crop_client = MockTokenClient::new(&env, &crop);
        crop_client.mint(&admin, &100_000i128);

        let vault_id = vault.open(
            &admin,
            &crop,
            &symbol_short!("MAIZE"),
            &1u64,
            &symbol_short!("NG"),
            &50_000i128,
            &100_000i128,
        );

        // User should have received USDC
        assert_eq!(usdc_client.balance(&admin), 100_000);

        // Repay in full - user needs USDC to repay (they already have it from borrowing)
        vault.repay(&admin, &vault_id, &100_000i128);

        let state = vault.get_vault(&vault_id);
        assert_eq!(state.debt_amount, 0);
        assert_eq!(state.collateral_amount, 0);

        // Collateral returned to user
        assert_eq!(crop_client.balance(&admin), 100_000);
        assert_eq!(crop_client.balance(&vault_addr), 0);
    }

    #[test]
    fn test_repay_partial() {
        let (env, admin, vault, _vault_addr, _usdc, crop, _registry, _oracle) = setup_env();

        let crop_client = MockTokenClient::new(&env, &crop);
        crop_client.mint(&admin, &100_000i128);

        let vault_id = vault.open(
            &admin,
            &crop,
            &symbol_short!("MAIZE"),
            &1u64,
            &symbol_short!("NG"),
            &50_000i128,
            &100_000i128,
        );

        // Partial repay
        vault.repay(&admin, &vault_id, &40_000i128);

        let state = vault.get_vault(&vault_id);
        assert_eq!(state.debt_amount, 60_000);
        // Collateral still locked since debt > 0
        assert_eq!(state.collateral_amount, 50_000);
    }

    #[test]
    fn test_liquidate_healthy_fails() {
        let (env, admin, vault, _vault_addr, _usdc, crop, _registry, _oracle) = setup_env();

        let crop_client = MockTokenClient::new(&env, &crop);
        crop_client.mint(&admin, &100_000i128);

        let vault_id = vault.open(
            &admin,
            &crop,
            &symbol_short!("MAIZE"),
            &1u64,
            &symbol_short!("NG"),
            &50_000i128,
            &100_000i128,
        );

        // LTV: (100_000 * 100) / (50_000 * 10_000_000) = 10_000_000 / 500_000_000_000 ~= 0%
        // Very healthy - should not be liquidatable
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            vault.liquidate(&admin, &vault_id);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_liquidate_unhealthy() {
        let (env, admin, vault, _vault_addr, usdc, crop, _registry, _oracle) = setup_env();

        let usdc_client = MockTokenClient::new(&env, &usdc);
        let crop_client = MockTokenClient::new(&env, &crop);
        crop_client.mint(&admin, &100_000i128);

        // Open vault with high LTV
        // Price of MAIZE = 10_000_000 per token
        // Deposit 2 CropTokens = worth 20_000_000
        // Borrow 18_000_000 -> LTV = 90% (above 85%)
        let vault_id = vault.open(
            &admin,
            &crop,
            &symbol_short!("MAIZE"),
            &1u64,
            &symbol_short!("NG"),
            &2i128,
            &18_000_000i128,
        );

        // Liquidator needs USDC to pay off debt
        let liquidator = Address::generate(&env);
        usdc_client.mint(&liquidator, &100_000_000i128);

        // Liquidate
        vault.liquidate(&liquidator, &vault_id);

        let state = vault.get_vault(&vault_id);
        assert_eq!(state.debt_amount, 0);
        assert_eq!(state.collateral_amount, 0);
    }

    #[test]
    fn test_multiple_vaults() {
        let (env, admin, vault, _vault_addr, _usdc, crop, _registry, _oracle) = setup_env();

        let crop_client = MockTokenClient::new(&env, &crop);
        crop_client.mint(&admin, &200_000i128);

        let id1 = vault.open(
            &admin,
            &crop,
            &symbol_short!("MAIZE"),
            &1u64,
            &symbol_short!("NG"),
            &50_000i128,
            &100_000i128,
        );
        let id2 = vault.open(
            &admin,
            &crop,
            &symbol_short!("MAIZE"),
            &1u64,
            &symbol_short!("NG"),
            &30_000i128,
            &50_000i128,
        );

        assert_eq!(id1, 1);
        assert_eq!(id2, 2);

        let s1 = vault.get_vault(&id1);
        let s2 = vault.get_vault(&id2);
        assert_eq!(s1.collateral_amount, 50_000);
        assert_eq!(s2.collateral_amount, 30_000);
    }
}
