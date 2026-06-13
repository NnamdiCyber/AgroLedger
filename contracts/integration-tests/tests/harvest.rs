mod common;
use common::*;
use soroban_sdk::{symbol_short, testutils::{Address as _, Ledger}, Address};

#[test]
fn test_harvest_full_flow() {
    let env = setup_env();

    // Fund harvest vault with USDC for yield payouts
    let usdc_client = MockTokenClient::new(&env.env, &env.usdc_id);
    usdc_client.mint(&env.harvest_id, &10_000_000i128);

    // Issue CropTokens to admin (≤50_000 per lot without multi-sig)
    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        50_000,
    );
    issue_crop_token(
        &env,
        symbol_short!("LOT2"),
        symbol_short!("MAIZE"),
        50_000,
    );

    // ── Deposit ───────────────────────────────────────────────────────────────
    let hct_received = env.harvest_client.deposit(&env.admin, &100_000i128);
    assert_eq!(hct_received, 100_000);
    assert_eq!(env.harvest_client.get_hct_balance(&env.admin), 100_000);
    assert_eq!(env.harvest_client.get_total_crop_deposited(), 100_000);
    assert_eq!(env.harvest_client.get_total_hct_supply(), 100_000);

    // Crop tokens transferred to vault
    assert_eq!(env.crop_client.balance(&env.admin), 0);
    assert_eq!(env.crop_client.balance(&env.harvest_id), 100_000);

    // ── Accrue yield ──────────────────────────────────────────────────────────
    env.env.ledger().set_timestamp(1_000_000 + 31_536_000);
    let yield_amount = env.harvest_client.accrue_yield(&env.admin);
    assert!(
        yield_amount >= 7_990,
        "Yield should be ~8,000 for 1 year at 8% APY"
    );
    assert!(yield_amount <= 8_010);
    assert_eq!(env.harvest_client.get_total_yield(), yield_amount);
    assert_eq!(
        env.harvest_client.get_apy(),
        800u32,
        "APY should be 800 bps"
    );

    // ── Withdraw ──────────────────────────────────────────────────────────────
    let (crop_out, yield_out) = env.harvest_client.withdraw(&env.admin, &100_000i128);
    assert_eq!(crop_out, 100_000);
    assert_eq!(yield_out, yield_amount);

    assert_eq!(env.crop_client.balance(&env.admin), 100_000);
    assert_eq!(env.crop_client.balance(&env.harvest_id), 0);
    assert_eq!(env.harvest_client.get_hct_balance(&env.admin), 0);
    assert_eq!(env.harvest_client.get_total_hct_supply(), 0);
    assert_eq!(env.harvest_client.get_total_crop_deposited(), 0);
}

#[test]
fn test_harvest_multiple_depositors() {
    let env = setup_env();

    let usdc_client = MockTokenClient::new(&env.env, &env.usdc_id);
    usdc_client.mint(&env.harvest_id, &10_000_000i128);

    let user1 = env.admin.clone();
    let user2 = Address::generate(&env.env);

    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        50_000,
    );
    issue_crop_token(
        &env,
        symbol_short!("LOT2"),
        symbol_short!("MAIZE"),
        50_000,
    );
    issue_crop_token(
        &env,
        symbol_short!("LOT3"),
        symbol_short!("MAIZE"),
        50_000,
    );
    env.crop_client
        .link_passport(&user2, &1u64, &symbol_short!("NG"));
    env.crop_client.transfer(&env.admin, &user2, &50_000i128);

    // Deposit from user1
    let hct1 = env.harvest_client.deposit(&user1, &100_000i128);
    assert_eq!(hct1, 100_000);

    // Deposit from user2
    let hct2 = env.harvest_client.deposit(&user2, &50_000i128);
    assert_eq!(hct2, 50_000);

    assert_eq!(env.harvest_client.get_total_crop_deposited(), 150_000);
    assert_eq!(env.harvest_client.get_total_hct_supply(), 150_000);

    // Accrue yield
    env.env.ledger().set_timestamp(1_000_000 + 31_536_000);
    let total_yield = env.harvest_client.accrue_yield(&env.admin);
    assert!(total_yield > 0);

    // User1 withdraws proportional share
    let (crop1, yield1) = env.harvest_client.withdraw(&user1, &100_000i128);
    assert_eq!(crop1, 100_000);
    let expected_yield1 = total_yield * 100_000 / 150_000;
    assert!((yield1 - expected_yield1).abs() <= 1);

    // User2 withdraws remaining
    let (crop2, yield2) = env.harvest_client.withdraw(&user2, &50_000i128);
    assert_eq!(crop2, 50_000);
    let expected_yield2 = total_yield - yield1;
    assert!((yield2 - expected_yield2).abs() <= 1);
}
