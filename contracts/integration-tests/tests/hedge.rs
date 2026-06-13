mod common;
use common::*;
use soroban_sdk::{symbol_short, testutils::{Address as _, Ledger}, Address, Symbol};

fn setup_hedge_env() -> (TestEnv, Address, Address) {
    let env = setup_env();

    let buyer = Address::generate(&env.env);
    let farmer = env.admin.clone();

    // Issue CropTokens to farmer
    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        10_000,
    );

    // Link buyer's passport for compliance on token transfers
    env.crop_client
        .link_passport(&buyer, &1u64, &symbol_short!("NG"));

    (env, buyer, farmer)
}

fn place_and_accept(env: &TestEnv, buyer: &Address, farmer: &Address) -> u64 {
    let price: i128 = 500_000_000;
    let salt: i128 = 12345;
    let commitment = make_commitment(&env.env, price, salt);

    let hedge_id = env.hedge_client.place_hedge(
        buyer,
        &symbol_short!("MAIZE"),
        &1000i128,
        &commitment,
        &2_000_000u64,
    );

    env.hedge_client.accept_hedge(&hedge_id, farmer);
    env.hedge_client.reveal(&hedge_id, &price, &salt);

    hedge_id
}

#[test]
fn test_hedge_physical_settlement() {
    let (env, buyer, farmer) = setup_hedge_env();

    let hedge_id = place_and_accept(&env, &buyer, &farmer);

    // Advance past expiry
    env.env.ledger().set_timestamp(2_000_001);

    let farmer_bal_before = env.crop_client.balance(&farmer);
    let buyer_bal_before = env.crop_client.balance(&buyer);

    // Settle physically
    env.hedge_client
        .settle(&hedge_id, &Symbol::new(&env.env, "Physical"), &farmer);

    let farmer_bal_after = env.crop_client.balance(&farmer);
    let buyer_bal_after = env.crop_client.balance(&buyer);

    assert_eq!(farmer_bal_before - farmer_bal_after, 1000);
    assert_eq!(buyer_bal_after - buyer_bal_before, 1000);

    let state = env.hedge_client.get_hedge(&hedge_id);
    assert_eq!(state.status, Symbol::new(&env.env, "SettledPhysical"));
}

#[test]
fn test_hedge_cash_settlement() {
    let (env, buyer, farmer) = setup_hedge_env();

    let hedge_id = place_and_accept(&env, &buyer, &farmer);

    // Farm buyer with CropTokens so they can pay cash settlement
    issue_crop_token(
        &env,
        symbol_short!("LOT2"),
        symbol_short!("MAIZE"),
        5_000,
    );
    env.crop_client.transfer(&env.admin, &buyer, &5_000i128);

    // Advance past expiry
    env.env.ledger().set_timestamp(2_000_001);

    let buyer_bal_before = env.crop_client.balance(&buyer);
    let farmer_bal_before = env.crop_client.balance(&farmer);

    // Settle in cash (buyer pays farmer)
    env.hedge_client
        .settle(&hedge_id, &Symbol::new(&env.env, "Cash"), &buyer);

    let expected = 1000 * 500_000_000 / 1_000_000_000i128;

    let buyer_bal_after = env.crop_client.balance(&buyer);
    let farmer_bal_after = env.crop_client.balance(&farmer);

    assert_eq!(buyer_bal_before - buyer_bal_after, expected);
    assert_eq!(farmer_bal_after - farmer_bal_before, expected);

    let state = env.hedge_client.get_hedge(&hedge_id);
    assert_eq!(state.status, Symbol::new(&env.env, "SettledCash"));
}

#[test]
fn test_hedge_cancel_before_expiry() {
    let (env, buyer, _farmer) = setup_hedge_env();

    let price: i128 = 500_000_000;
    let salt: i128 = 12345;
    let commitment = make_commitment(&env.env, price, salt);

    let hedge_id = env.hedge_client.place_hedge(
        &buyer,
        &symbol_short!("MAIZE"),
        &1000i128,
        &commitment,
        &2_000_000u64,
    );

    // Cancel before expiry
    env.hedge_client.cancel(&hedge_id, &buyer);

    let state = env.hedge_client.get_hedge(&hedge_id);
    assert_eq!(state.status, Symbol::new(&env.env, "Cancelled"));
}

#[test]
fn test_hedge_reveal_mismatch_panics() {
    let (env, buyer, farmer) = setup_hedge_env();

    let price: i128 = 500_000_000;
    let salt: i128 = 12345;
    let commitment = make_commitment(&env.env, price, salt);

    let hedge_id = env.hedge_client.place_hedge(
        &buyer,
        &symbol_short!("MAIZE"),
        &1000i128,
        &commitment,
        &2_000_000u64,
    );

    env.hedge_client.accept_hedge(&hedge_id, &farmer);

    let wrong_price: i128 = 600_000_000;
    let wrong_salt: i128 = 99999;

    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        env.hedge_client
            .reveal(&hedge_id, &wrong_price, &wrong_salt);
    }));
    assert!(result.is_err(), "Reveal with mismatched price should panic");
}
