use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, testutils::Address as _, Address, Bytes,
    BytesN, Env, Symbol, Vec,
};

use privacy_passport::{PrivacyPassport, PrivacyPassportClient};
use compliance_registry::{ComplianceRegistry, ComplianceRegistryClient};
use warehouse_oracle::{WarehouseOracle, WarehouseOracleClient, InspectorSet};
use crop_token::{CropToken, CropTokenClient};
use collateral_vault::{CollateralVault, CollateralVaultClient};
use cross_border_router::{CrossBorderRouter, CrossBorderRouterClient, TravelRuleData};

// ─── Mock Token (generic ERC-20-like for USDC) ───────────────────────────────

#[contracttype]
pub enum MockDataKey {
    Balance(Address),
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage()
            .persistent()
            .get(&MockDataKey::Balance(id))
            .unwrap_or(0)
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
}

// ─── TestEnv ─────────────────────────────────────────────────────────────────

pub struct TestEnv {
    pub env: Env,
    pub admin: Address,
    pub oracle_pubkey: Address,
    pub passport_client: PrivacyPassportClient<'static>,
    pub registry_client: ComplianceRegistryClient<'static>,
    pub oracle_client: WarehouseOracleClient<'static>,
    pub crop_client: CropTokenClient<'static>,
    pub vault_client: CollateralVaultClient<'static>,
    pub router_client: CrossBorderRouterClient<'static>,
    pub usdc_id: Address,
    pub vault_id: Address,
    pub router_id: Address,
    pub crop_id: Address,
}

pub fn setup_env() -> TestEnv {
    let env = Env::default();
    env.mock_all_auths();
    let admin = Address::generate(&env);
    let oracle_pubkey = Address::generate(&env);

    // ── 1. PrivacyPassport ──────────────────────────────────────────────────
    let passport_id = env.register_contract(None, PrivacyPassport);
    let passport_client = PrivacyPassportClient::new(&env, &passport_id);
    passport_client.initialize(&admin);

    // Register a passport for NG jurisdiction
    let nullifier = BytesN::from_array(&env, &[1u8; 32]);
    let proof = BytesN::from_array(&env, &[2u8; 32]);
    let passport_num = passport_client.register(&nullifier, &proof, &symbol_short!("NG"));
    assert!(passport_num == 1 || passport_num > 0);

    // ── 2. ComplianceRegistry ───────────────────────────────────────────────
    let registry_id = env.register_contract(None, ComplianceRegistry);
    let registry_client = ComplianceRegistryClient::new(&env, &registry_id);
    registry_client.initialize(&admin, &passport_id);
    registry_client.add_jurisdiction(&symbol_short!("NG"));

    // ── 3. WarehouseOracle ──────────────────────────────────────────────────
    let oracle_id = env.register_contract(None, WarehouseOracle);
    let oracle_client = WarehouseOracleClient::new(&env, &oracle_id);
    let inspector_set = InspectorSet {
        inspectors: Vec::new(&env),
        threshold: 0,
    };
    oracle_client.initialize(&admin, &oracle_pubkey, &inspector_set);

    // Push a price for MAIZE
    oracle_client.push_price(&symbol_short!("MAIZE"), &10_000_000u64, &1_000_000u64);

    // ── 4. CropToken ────────────────────────────────────────────────────────
    let crop_id = env.register_contract(None, CropToken);
    let crop_client = CropTokenClient::new(&env, &crop_id);
    crop_client.initialize(&admin, &oracle_id, &registry_id);

    // Link admin's address to passport 1 / NG so transfer compliance passes
    crop_client.link_passport(&admin, &1u64, &symbol_short!("NG"));

    // ── 5. Mock USDC Token ──────────────────────────────────────────────────
    let usdc_id = env.register_contract(None, MockToken);
    let usdc_client = MockTokenClient::new(&env, &usdc_id);

    // ── 6. CollateralVault ──────────────────────────────────────────────────
    let vault_contract_id = env.register_contract(None, CollateralVault);
    let vault_client = CollateralVaultClient::new(&env, &vault_contract_id);
    vault_client.initialize(&admin, &registry_id, &usdc_id, &oracle_id);

    // Fund vault with USDC for lending
    usdc_client.mint(&vault_contract_id, &1_000_000_000_000i128);

    // ── 7. CrossBorderRouter ────────────────────────────────────────────────
    let router_contract_id = env.register_contract(None, CrossBorderRouter);
    let router_client = CrossBorderRouterClient::new(&env, &router_contract_id);
    router_client.initialize(&admin, &registry_id);

    // Register USDC asset in router
    router_client.register_asset(&admin, &symbol_short!("USDC"), &usdc_id);

    TestEnv {
        env,
        admin,
        oracle_pubkey,
        passport_client,
        registry_client,
        oracle_client,
        crop_client,
        vault_client,
        router_client,
        usdc_id,
        vault_id: vault_contract_id,
        router_id: router_contract_id,
        crop_id,
    }
}

// Helper to issue CropTokens after submitting a lot to the oracle
pub fn issue_crop_token(env: &TestEnv, lot_id: Symbol, commodity: Symbol, quantity_kg: u64) {
    let warehouse_id = symbol_short!("WH001");
    let inspectors = Vec::new(&env.env);
    env.oracle_client
        .submit_lot(&warehouse_id, &lot_id, &commodity, &quantity_kg, &inspectors);

    let sig = Bytes::from_array(&env.env, &[0u8; 64]);
    env.crop_client
        .issue(&warehouse_id, &lot_id, &commodity, &quantity_kg, &sig);
}

// Create a TravelRuleData for testing
pub fn test_travel_rule() -> TravelRuleData {
    TravelRuleData {
        passport_id: 1u64,
        jurisdiction: symbol_short!("NG"),
    }
}
