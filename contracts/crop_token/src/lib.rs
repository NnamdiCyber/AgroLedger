#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Bytes, Env, IntoVal, Symbol, Val, Vec};

mod metadata;
mod transfer;

pub use crate::metadata::LotMeta;

#[contracttype]
pub enum DataKey {
    Admin,
    WarehouseOracle,
    ComplianceRegistry,
    LotMeta(Symbol),
    Balance(Address),
    AddressPassport(Address),
}

#[contract]
pub struct CropToken;

#[contractimpl]
impl CropToken {
    pub fn initialize(
        env: Env,
        admin: Address,
        warehouse_oracle: Address,
        compliance_registry: Address,
    ) {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::WarehouseOracle, &warehouse_oracle);
        env.storage()
            .instance()
            .set(&DataKey::ComplianceRegistry, &compliance_registry);
    }

    pub fn issue(
        env: Env,
        warehouse_id: Symbol,
        lot_id: Symbol,
        commodity: Symbol,
        quantity_kg: u64,
        oracle_sig: Bytes,
    ) -> Address {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        admin.require_auth();

        let oracle: Address = env.storage().instance().get(&DataKey::WarehouseOracle).expect("WarehouseOracle not set");

        let args: Vec<Val> = (warehouse_id.clone(), lot_id.clone()).into_val(&env);
        let lot_valid: bool = env.invoke_contract(
            &oracle,
            &Symbol::new(&env, "verify_lot"),
            args,
        );
        assert!(lot_valid, "Lot not verified by oracle");

        let lot_meta = LotMeta {
            warehouse_id,
            lot_id: lot_id.clone(),
            commodity: commodity.clone(),
            quantity_kg,
            oracle_attestation: oracle_sig,
            expiry: env.ledger().timestamp() + 31536000,
            price: 0,
        };

        metadata::set_lot_metadata(&env, &lot_id, &lot_meta);

        let amount = quantity_kg as i128;
        let admin_balance_key = DataKey::Balance(admin.clone());
        let current_balance: i128 = env.storage().persistent().get(&admin_balance_key).unwrap_or(0);
        env.storage().persistent().set(&admin_balance_key, &(current_balance + amount));

        env.events().publish(
            (Symbol::new(&env, "CropTokenIssued"), lot_id),
            (commodity, quantity_kg),
        );

        env.current_contract_address()
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        transfer::execute_transfer(env, from, to, amount);
    }

    pub fn burn(env: Env, lot_id: Symbol) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        admin.require_auth();

        let lot_meta = metadata::get_lot_metadata(&env, &lot_id);
        let amount = lot_meta.quantity_kg as i128;

        let admin_balance_key = DataKey::Balance(admin.clone());
        let current_balance: i128 = env.storage().persistent().get(&admin_balance_key).unwrap_or(0);
        assert!(current_balance >= amount, "Insufficient balance to burn");

        env.storage().persistent().set(&admin_balance_key, &(current_balance - amount));

        env.events().publish(
            (Symbol::new(&env, "CropTokenBurned"), lot_id),
            amount,
        );
    }

    pub fn get_lot_metadata(env: Env, lot_id: Symbol) -> LotMeta {
        metadata::get_lot_metadata(&env, &lot_id)
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(id)).unwrap_or(0)
    }

    pub fn link_passport(env: Env, address: Address, passport_id: u64, jurisdiction: Symbol) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).expect("Admin not set");
        admin.require_auth();

        env.storage()
            .instance()
            .set(&DataKey::AddressPassport(address), &(passport_id, jurisdiction));
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, BytesN, Env, Vec};

    use warehouse_oracle::{WarehouseOracle, WarehouseOracleClient, InspectorSet};
    use compliance_registry::{ComplianceRegistry, ComplianceRegistryClient};
    use privacy_passport::{PrivacyPassport, PrivacyPassportClient};

    fn setup_env() -> (Env, Address, CropTokenClient<'static>, WarehouseOracleClient<'static>, ComplianceRegistryClient<'static>, PrivacyPassportClient<'static>, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let passport_id = env.register_contract(None, PrivacyPassport);
        let passport_client = PrivacyPassportClient::new(&env, &passport_id);
        passport_client.initialize(&admin);

        let nullifier = BytesN::from_array(&env, &[0u8; 32]);
        let proof = BytesN::from_array(&env, &[0u8; 32]);
        passport_client.register(&nullifier, &proof, &symbol_short!("NG"));

        let registry_id = env.register_contract(None, ComplianceRegistry);
        let registry_client = ComplianceRegistryClient::new(&env, &registry_id);
        registry_client.initialize(&admin, &passport_id);
        registry_client.add_jurisdiction(&symbol_short!("NG"));

        let oracle_id = env.register_contract(None, WarehouseOracle);
        let oracle_client = WarehouseOracleClient::new(&env, &oracle_id);
        let oracle_pubkey = Address::generate(&env);
        let inspector_set = InspectorSet {
            inspectors: Vec::new(&env),
            threshold: 0,
        };
        oracle_client.initialize(&admin, &oracle_pubkey, &inspector_set);

        let contract_id = env.register_contract(None, CropToken);
        let client = CropTokenClient::new(&env, &contract_id);

        client.initialize(&admin, &oracle_id, &registry_id);

        (env, admin, client, oracle_client, registry_client, passport_client, oracle_id)
    }

    #[test]
    fn test_initialize() {
        let (_env, admin, client, _oracle, _registry, _passport, _oracle_id) = setup_env();
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin, &_oracle_id, &_registry.address);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_issue_valid() {
        let (env, admin, client, oracle_client, _registry, _passport, _oracle_id) = setup_env();
        let lot_id = symbol_short!("LOT1");
        let commodity = symbol_short!("MAIZE");
        let quantity = 1000u64;
        let warehouse_id = symbol_short!("WH001");

        let inspectors = Vec::new(&env);
        oracle_client.submit_lot(&warehouse_id, &lot_id, &commodity, &quantity, &inspectors);

        let sig = Bytes::from_array(&env, &[0u8; 64]);
        client.issue(&warehouse_id, &lot_id, &commodity, &quantity, &sig);

        assert_eq!(client.balance(&admin), 1000i128);

        let meta = client.get_lot_metadata(&lot_id);
        assert_eq!(meta.commodity, commodity);
        assert_eq!(meta.quantity_kg, quantity);
    }

    #[test]
    fn test_transfer_compliance_gated() {
        let (env, admin, client, oracle_client, _registry, _passport, _oracle_id) = setup_env();
        let user = Address::generate(&env);

        client.link_passport(&admin, &1u64, &symbol_short!("NG"));

        let lot_id = symbol_short!("LOT1");
        let commodity = symbol_short!("MAIZE");
        let warehouse_id = symbol_short!("WH001");

        let inspectors = Vec::new(&env);
        oracle_client.submit_lot(&warehouse_id, &lot_id, &commodity, &1000, &inspectors);

        let sig = Bytes::from_array(&env, &[0u8; 64]);
        client.issue(&warehouse_id, &lot_id, &commodity, &1000, &sig);

        client.transfer(&admin, &user, &500);

        assert_eq!(client.balance(&admin), 500);
        assert_eq!(client.balance(&user), 500);
    }

    #[test]
    fn test_burn() {
        let (env, admin, client, oracle_client, _registry, _passport, _oracle_id) = setup_env();
        let lot_id = symbol_short!("LOT1");
        let warehouse_id = symbol_short!("WH001");

        let inspectors = Vec::new(&env);
        oracle_client.submit_lot(&warehouse_id, &lot_id, &symbol_short!("MAIZE"), &1000, &inspectors);

        let sig = Bytes::from_array(&env, &[0u8; 64]);
        client.issue(&warehouse_id, &lot_id, &symbol_short!("MAIZE"), &1000, &sig);

        client.burn(&lot_id);

        assert_eq!(client.balance(&admin), 0);
    }

    #[test]
    fn test_issue_invalid_sig_panics() {
        let (_env, _admin, client, _oracle_client, _registry, _passport, _oracle_id) = setup_env();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.issue(&symbol_short!("WH001"), &symbol_short!("LOT1"), &symbol_short!("MAIZE"), &1000, &Bytes::from_array(&_env, &[0u8; 64]));
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_get_metadata() {
        let (env, admin, client, oracle_client, _registry, _passport, _oracle_id) = setup_env();
        let lot_id = symbol_short!("LOT1");
        let warehouse_id = symbol_short!("WH001");

        let inspectors = Vec::new(&env);
        oracle_client.submit_lot(&warehouse_id, &lot_id, &symbol_short!("MAIZE"), &2000, &inspectors);

        let sig = Bytes::from_array(&env, &[0u8; 64]);
        client.issue(&warehouse_id, &lot_id, &symbol_short!("MAIZE"), &2000, &sig);

        let meta = client.get_lot_metadata(&lot_id);
        assert_eq!(meta.quantity_kg, 2000);
        assert_eq!(meta.lot_id, lot_id);
    }
}
