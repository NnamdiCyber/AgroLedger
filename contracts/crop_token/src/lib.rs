#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Bytes, BytesN, Val, Vec, IntoVal};

mod metadata;
mod transfer;

pub use crate::metadata::LotMeta;

#[contracttype]
pub enum DataKey {
    Admin,
    WarehouseOracle,
    ComplianceRegistry,
    LotMeta,
    Balance(Address),
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
        warehouse_id: Address,
        lot_id: Symbol,
        commodity: Symbol,
        quantity_kg: u64,
        oracle_sig: Bytes,
    ) -> Address {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let _oracle: Address = env.storage().instance().get(&DataKey::WarehouseOracle).unwrap();
        
        let lot_meta = LotMeta {
            warehouse_id: warehouse_id.clone(),
            commodity: commodity.clone(),
            quantity_kg,
            oracle_attestation: oracle_sig.clone(),
            expiry: env.ledger().timestamp() + 31536000, // 1 year expiry
            price: 0, // Will be updated by oracle later
        };

        metadata::set_lot_metadata(&env, &lot_meta);

        // Minting tokens 1:1 with quantity_kg
        let amount = quantity_kg as i128;
        let admin_balance_key = DataKey::Balance(admin.clone());
        let current_balance: i128 = env.storage().persistent().get(&admin_balance_key).unwrap_or(0);
        env.storage().persistent().set(&admin_balance_key, &(current_balance + amount));

        env.events().publish(
            (Symbol::new(&env, "CropTokenIssued"), lot_id),
            (warehouse_id, commodity, quantity_kg),
        );

        env.current_contract_address()
    }

    pub fn transfer(env: Env, from: Address, to: Address, amount: i128) {
        transfer::execute_transfer(env, from, to, amount);
    }

    pub fn burn(env: Env, amount: i128) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        let admin_balance_key = DataKey::Balance(admin.clone());
        let current_balance: i128 = env.storage().persistent().get(&admin_balance_key).unwrap_or(0);
        assert!(current_balance >= amount, "Insufficient balance to burn");
        
        env.storage().persistent().set(&admin_balance_key, &(current_balance - amount));

        env.events().publish(
            (Symbol::new(&env, "CropTokenBurned"),),
            amount,
        );
    }

    pub fn get_lot_metadata(env: Env) -> LotMeta {
        metadata::get_lot_metadata(&env)
    }

    pub fn balance(env: Env, id: Address) -> i128 {
        env.storage().persistent().get(&DataKey::Balance(id)).unwrap_or(0)
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, Bytes, Env, BytesN};
    
    use warehouse_oracle::{WarehouseOracle, WarehouseOracleClient, InspectorSet};
    use compliance_registry::{ComplianceRegistry, ComplianceRegistryClient};
    use privacy_passport::{PrivacyPassport, PrivacyPassportClient};

    fn setup_env() -> (Env, Address, CropTokenClient<'static>, WarehouseOracleClient<'static>, ComplianceRegistryClient<'static>) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        
        // Setup Oracle
        let oracle_id = env.register_contract(None, WarehouseOracle);
        let oracle_client = WarehouseOracleClient::new(&env, &oracle_id);
        let oracle_pubkey = Address::generate(&env);
        let inspectors = InspectorSet { inspectors: Vec::new(&env), threshold: 0 };
        oracle_client.initialize(&admin, &oracle_pubkey, &inspectors);

        // Setup Passport and Registry
        let passport_id = env.register_contract(None, PrivacyPassport);
        let passport_client = PrivacyPassportClient::new(&env, &passport_id);
        passport_client.initialize(&admin);
        
        let registry_id = env.register_contract(None, ComplianceRegistry);
        let registry_client = ComplianceRegistryClient::new(&env, &registry_id);
        registry_client.initialize(&admin, &passport_id);
        registry_client.add_jurisdiction(&symbol_short!("NG"));

        // Register a passport for id 1
        let nullifier = BytesN::from_array(&env, &[0u8; 32]);
        let proof = BytesN::from_array(&env, &[0u8; 32]);
        passport_client.register(&nullifier, &proof, &symbol_short!("NG"));

        // Setup CropToken
        let contract_id = env.register_contract(None, CropToken);
        let client = CropTokenClient::new(&env, &contract_id);
        
        client.initialize(&admin, &oracle_id, &registry_id);

        (env, admin, client, oracle_client, registry_client)
    }

    #[test]
    fn test_initialize() {
        let (_env, admin, client, oracle, registry) = setup_env();
        // Should panic if initialized again
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.initialize(&admin, &oracle.address, &registry.address);
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_issue_valid() {
        let (_env, admin, client, _oracle, _registry) = setup_env();
        let warehouse = Address::generate(&_env);
        let lot_id = symbol_short!("LOT1");
        let commodity = symbol_short!("MAIZE");
        let quantity = 1000u64;
        let sig = Bytes::from_array(&_env, &[0u8; 64]);

        client.issue(&warehouse, &lot_id, &commodity, &quantity, &sig);

        assert_eq!(client.balance(&admin), 1000i128);
        
        let meta = client.get_lot_metadata();
        assert_eq!(meta.commodity, commodity);
        assert_eq!(meta.quantity_kg, quantity);
    }

    #[test]
    fn test_transfer_compliance_gated() {
        let (_env, admin, client, _oracle, _registry) = setup_env();
        let warehouse = Address::generate(&_env);
        let user = Address::generate(&_env);
        
        client.issue(&warehouse, &symbol_short!("LOT1"), &symbol_short!("MAIZE"), &1000, &Bytes::from_array(&_env, &[0u8; 64]));

        client.transfer(&admin, &user, &500);

        assert_eq!(client.balance(&admin), 500);
        assert_eq!(client.balance(&user), 500);
    }

    #[test]
    fn test_burn() {
        let (_env, admin, client, _oracle, _registry) = setup_env();
        client.issue(&Address::generate(&_env), &symbol_short!("LOT1"), &symbol_short!("MAIZE"), &1000, &Bytes::from_array(&_env, &[0u8; 64]));

        client.burn(&400);

        assert_eq!(client.balance(&admin), 600);
    }
}
