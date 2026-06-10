#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

mod attestation;
mod price_feed;

#[contracttype]
pub enum DataKey {
    Admin,
    OraclePubkey,
    InspectorSet,
    LotCounter,
    Lot(u64),
    LotLookup(Symbol, Symbol),
    Price(Symbol),
}

#[derive(Clone)]
#[contracttype]
pub struct InspectorSet {
    pub inspectors: Vec<Address>,
    pub threshold: u32,
}

#[derive(Clone)]
#[contracttype]
pub struct LotState {
    pub warehouse_id: Symbol,
    pub lot_id: Symbol,
    pub commodity: Symbol,
    pub quantity_kg: u64,
    pub approved: bool,
    pub approved_at: u64,
}

#[derive(Clone)]
#[contracttype]
pub struct PriceData {
    pub price: u64,
    pub timestamp: u64,
}

#[contract]
pub struct WarehouseOracle;

#[contractimpl]
impl WarehouseOracle {
    pub fn initialize(env: Env, admin: Address, oracle_pubkey: Address, inspectors: InspectorSet) {
        admin.require_auth();
        env.storage()
            .instance()
            .set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::OraclePubkey, &oracle_pubkey);
        env.storage()
            .instance()
            .set(&DataKey::InspectorSet, &inspectors);
    }

    pub fn submit_lot(
        env: Env,
        warehouse_id: Symbol,
        lot_id: Symbol,
        commodity: Symbol,
        quantity_kg: u64,
        inspector_sigs: Vec<Address>,
    ) -> u64 {
        attestation::execute_submit_lot(env, warehouse_id, lot_id, commodity, quantity_kg, inspector_sigs)
    }

    pub fn push_price(env: Env, commodity: Symbol, price_usdc: u64, timestamp: u64) {
        price_feed::execute_push_price(env, commodity, price_usdc, timestamp)
    }

    pub fn get_price(env: Env, commodity: Symbol) -> PriceData {
        price_feed::execute_get_price(env, commodity)
    }

    pub fn verify_lot(env: Env, warehouse_id: Symbol, lot_id: Symbol) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::LotLookup(warehouse_id, lot_id))
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, Env, Vec};

    fn setup_env() -> (Env, Address, WarehouseOracleClient<'static>, Address, Vec<Address>) {
        let env = Env::default();
        let admin = Address::generate(&env);
        let oracle = Address::generate(&env);

        let inspector1 = Address::generate(&env);
        let inspector2 = Address::generate(&env);
        let inspector3 = Address::generate(&env);

        let inspectors = Vec::from_array(&env, [
            inspector1.clone(),
            inspector2.clone(),
            inspector3.clone(),
        ]);

        let inspector_set = InspectorSet {
            inspectors: inspectors.clone(),
            threshold: 2,
        };

        let contract_id = env.register_contract(None, WarehouseOracle);
        let client = WarehouseOracleClient::new(&env, &contract_id);

        env.mock_all_auths();
        client.initialize(&admin, &oracle, &inspector_set);

        (env, admin, client, oracle, inspectors)
    }

    #[test]
    fn test_submit_lot() {
        let (_env, _admin, client, _oracle, inspectors) = setup_env();

        let warehouse_id = symbol_short!("WH001");
        let lot_id = symbol_short!("LOT001");
        let commodity = symbol_short!("MAIZE");

        let lot_num = client.submit_lot(&warehouse_id, &lot_id, &commodity, &10_000u64, &inspectors);
        assert_eq!(lot_num, 1u64);
    }

    #[test]
    fn test_submit_lot_requires_multisig() {
        let (env, _admin, client, _oracle, inspectors) = setup_env();

        let single = Vec::from_array(&env, [inspectors.get(0).unwrap()]);

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            client.submit_lot(
                &symbol_short!("WH001"),
                &symbol_short!("LOT001"),
                &symbol_short!("MAIZE"),
                &10_000u64,
                &single,
            );
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_push_price_oracle_sig() {
        let (_env, _admin, client, _oracle, _inspectors) = setup_env();

        let commodity = symbol_short!("MAIZE");
        client.push_price(&commodity, &200_000_000u64, &1_000_000u64);

        let price_data = client.get_price(&commodity);
        assert_eq!(price_data.price, 200_000_000);
        assert_eq!(price_data.timestamp, 1_000_000);
    }

    #[test]
    fn test_get_price() {
        let (_env, _admin, client, _oracle, _inspectors) = setup_env();

        let commodity = symbol_short!("MAIZE");
        let price_data = client.get_price(&commodity);
        assert_eq!(price_data.price, 0);
        assert_eq!(price_data.timestamp, 0);
    }
}
