#![no_std]
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Symbol, Vec};

mod router;
mod stablecoins;

#[contracttype]
pub enum DataKey {
    Admin,
    ComplianceRegistry,
    RouteCounter,
    Asset(Symbol),
}

#[derive(Clone)]
#[contracttype]
pub struct PathResult {
    pub from: Address,
    pub to: Address,
    pub send_asset: Address,
    pub recv_asset: Address,
    pub amount_sent: i128,
    pub amount_received: i128,
    pub fee: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct PathQuote {
    pub send_asset: Address,
    pub recv_asset: Address,
    pub amount_out: i128,
    pub fee: i128,
}

#[derive(Clone)]
#[contracttype]
pub struct TravelRuleData {
    pub passport_id: u64,
    pub jurisdiction: Symbol,
}

#[contract]
pub struct CrossBorderRouter;

#[contractimpl]
impl CrossBorderRouter {
    pub fn initialize(env: Env, admin: Address, compliance_registry: Address) {
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::ComplianceRegistry, &compliance_registry);
    }

    pub fn route(
        env: Env,
        from: Address,
        to: Address,
        send_asset: Address,
        recv_asset: Address,
        amount: i128,
        travel_rule_data: TravelRuleData,
    ) -> PathResult {
        router::execute_route(env, from, to, send_asset, recv_asset, amount, travel_rule_data)
    }

    pub fn estimate(
        env: Env,
        send_asset: Address,
        recv_asset: Address,
        amount: i128,
    ) -> Vec<PathQuote> {
        router::compute_estimate(&env, send_asset, recv_asset, amount)
    }

    pub fn register_asset(env: Env, admin: Address, symbol: Symbol, contract_id: Address) {
        admin.require_auth();
        stablecoins::register_asset(&env, &symbol, &contract_id);
    }

    pub fn get_asset(env: Env, symbol: Symbol) -> Address {
        stablecoins::get_asset(&env, &symbol)
    }
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::{symbol_short, BytesN, Env};

    use compliance_registry::{ComplianceRegistry, ComplianceRegistryClient};
    use privacy_passport::{PrivacyPassport, PrivacyPassportClient};

    #[contracttype]
    enum MockDataKey {
        Balance(Address),
        Admin,
    }

    #[contract]
    struct MockToken;

    #[contractimpl]
    impl MockToken {
        pub fn initialize(env: Env, admin: Address) {
            env.storage()
                .instance()
                .set(&MockDataKey::Admin, &admin);
        }

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
        CrossBorderRouterClient<'static>,
        Address,
        Address,
    ) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);

        let passport_id = env.register_contract(None, PrivacyPassport);
        let passport_client = PrivacyPassportClient::new(&env, &passport_id);
        passport_client.initialize(&admin);

        let registry_id = env.register_contract(None, ComplianceRegistry);
        let registry_client = ComplianceRegistryClient::new(&env, &registry_id);
        registry_client.initialize(&admin, &passport_id);
        registry_client.add_jurisdiction(&symbol_short!("NG"));

        let nullifier = BytesN::from_array(&env, &[1u8; 32]);
        let proof = BytesN::from_array(&env, &[2u8; 32]);
        passport_client.register(&nullifier, &proof, &symbol_short!("NG"));

        let router_id = env.register_contract(None, CrossBorderRouter);
        let router_client = CrossBorderRouterClient::new(&env, &router_id);
        router_client.initialize(&admin, &registry_id);

        let usdc_id = env.register_contract(None, MockToken);
        let usdc_client = MockTokenClient::new(&env, &usdc_id);
        usdc_client.initialize(&admin);

        let cngn_id = env.register_contract(None, MockToken);
        let cngn_client = MockTokenClient::new(&env, &cngn_id);
        cngn_client.initialize(&admin);

        (env, admin, router_client, usdc_id, cngn_id)
    }

    #[test]
    fn test_register_asset() {
        let (_env, admin, router, usdc_id, _cngn_id) = setup_env();

        let usdc_sym = symbol_short!("USDC");
        router.register_asset(&admin, &usdc_sym, &usdc_id);

        let stored = router.get_asset(&usdc_sym);
        assert_eq!(stored, usdc_id);
    }

    #[test]
    fn test_route_same_asset() {
        let (env, _admin, router, usdc_id, _cngn_id) = setup_env();
        let user = Address::generate(&env);
        let recipient = Address::generate(&env);

        let usdc_client = MockTokenClient::new(&env, &usdc_id);
        usdc_client.mint(&user, &1_000_000i128);

        let travel_data = TravelRuleData {
            passport_id: 1u64,
            jurisdiction: symbol_short!("NG"),
        };

        let result = router.route(
            &user,
            &recipient,
            &usdc_id,
            &usdc_id,
            &100_000i128,
            &travel_data,
        );

        assert_eq!(result.amount_sent, 100_000);
        assert_eq!(result.fee, 150);
        assert_eq!(result.amount_received, 99_850);
        assert_eq!(result.send_asset, usdc_id);
        assert_eq!(result.recv_asset, usdc_id);
        assert_eq!(result.from, user);
        assert_eq!(result.to, recipient);

        assert_eq!(usdc_client.balance(&user), 900_000);
        assert_eq!(usdc_client.balance(&recipient), 99_850);
    }

    #[test]
    fn test_route_cross_border() {
        let (env, _admin, router, usdc_id, cngn_id) = setup_env();
        let user = Address::generate(&env);
        let recipient = Address::generate(&env);

        let usdc_client = MockTokenClient::new(&env, &usdc_id);
        let cngn_client = MockTokenClient::new(&env, &cngn_id);

        usdc_client.mint(&user, &1_000_000i128);
        cngn_client.mint(&router.address, &1_000_000i128);

        let travel_data = TravelRuleData {
            passport_id: 1u64,
            jurisdiction: symbol_short!("NG"),
        };

        let result = router.route(
            &user,
            &recipient,
            &usdc_id,
            &cngn_id,
            &200_000i128,
            &travel_data,
        );

        assert_eq!(result.amount_sent, 200_000);
        assert_eq!(result.fee, 300);
        assert_eq!(result.amount_received, 199_700);
        assert_eq!(result.send_asset, usdc_id);
        assert_eq!(result.recv_asset, cngn_id);

        assert_eq!(usdc_client.balance(&user), 800_000);
        assert_eq!(cngn_client.balance(&recipient), 199_700);
    }

    #[test]
    fn test_route_blocked_jurisdiction() {
        let (env, _admin, router, usdc_id, _cngn_id) = setup_env();
        let user = Address::generate(&env);
        let recipient = Address::generate(&env);

        let usdc_client = MockTokenClient::new(&env, &usdc_id);
        usdc_client.mint(&user, &1_000_000i128);

        let travel_data = TravelRuleData {
            passport_id: 1u64,
            jurisdiction: symbol_short!("US"),
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            router.route(
                &user,
                &recipient,
                &usdc_id,
                &usdc_id,
                &100_000i128,
                &travel_data,
            );
        }));
        assert!(result.is_err());
    }

    #[test]
    fn test_route_travel_rule_threshold() {
        let (env, _admin, router, usdc_id, _cngn_id) = setup_env();
        let user = Address::generate(&env);
        let recipient = Address::generate(&env);

        let usdc_client = MockTokenClient::new(&env, &usdc_id);
        usdc_client.mint(&user, &100_000_000_000i128);

        let travel_data = TravelRuleData {
            passport_id: 1u64,
            jurisdiction: symbol_short!("NG"),
        };

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            router.route(
                &user,
                &recipient,
                &usdc_id,
                &usdc_id,
                &50_000_000_001i128,
                &travel_data,
            );
        }));
        assert!(result.is_err());

        let ok_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            router.route(
                &user,
                &recipient,
                &usdc_id,
                &usdc_id,
                &5_000_000_000i128,
                &travel_data,
            );
        }));
        assert!(ok_result.is_ok());
    }
}
