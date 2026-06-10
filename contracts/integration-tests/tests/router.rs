mod common;
use common::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address};

#[test]
fn test_register_assets_and_route() {
    let env = setup_env();

    let user = Address::generate(&env.env);
    let recipient = Address::generate(&env.env);

    // Deploy and register cNGN token
    let cngn_id = env.env.register_contract(None, MockToken);
    let cngn_client = MockTokenClient::new(&env.env, &cngn_id);
    cngn_client.mint(&env.router_id, &10_000_000_000i128);

    env.router_client.register_asset(
        &env.admin,
        &symbol_short!("cNGN"),
        &cngn_id,
    );

    // Fund user with USDC
    let usdc_client = MockTokenClient::new(&env.env, &env.usdc_id);
    usdc_client.mint(&user, &1_000_000i128);

    // ── Same-asset route ──────────────────────────────────────────────────────
    let travel_data = test_travel_rule();
    let result = env.router_client.route(
        &user,
        &recipient,
        &env.usdc_id,
        &env.usdc_id,
        &100_000i128,
        &travel_data,
    );

    assert_eq!(result.amount_sent, 100_000);
    // 0.15% fee = 150
    assert_eq!(result.fee, 150);
    assert_eq!(result.amount_received, 99_850);
    assert_eq!(result.from, user);
    assert_eq!(result.to, recipient);

    assert_eq!(usdc_client.balance(&user), 900_000);
    assert_eq!(usdc_client.balance(&recipient), 99_850);

    // ── Cross-border route ────────────────────────────────────────────────────
    let user2 = Address::generate(&env.env);
    let recipient2 = Address::generate(&env.env);
    usdc_client.mint(&user2, &500_000i128);

    let result2 = env.router_client.route(
        &user2,
        &recipient2,
        &env.usdc_id,
        &cngn_id,
        &200_000i128,
        &travel_data,
    );

    assert_eq!(result2.amount_sent, 200_000);
    // 0.15% of 200_000 = 300
    assert_eq!(result2.fee, 300);
    assert_eq!(result2.amount_received, 199_700);

    assert_eq!(usdc_client.balance(&user2), 300_000);
    assert_eq!(cngn_client.balance(&recipient2), 199_700);
}

#[test]
fn test_route_estimate() {
    let env = setup_env();

    let quotes = env.router_client.estimate(
        &env.usdc_id,
        &env.usdc_id,
        &50_000i128,
    );

    assert_eq!(quotes.len(), 1);
    let quote = quotes.get(0).unwrap();
    assert_eq!(quote.amount_out, 49_925); // 50_000 - 75 (0.15%)
    assert_eq!(quote.fee, 75);
}
