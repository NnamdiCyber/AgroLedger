mod common;
use common::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address};

#[test]
fn test_open_vault_and_repay() {
    let env = setup_env();

    // Issue CropTokens to admin (≤50_000 to avoid 3-sig requirement)
    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        50_000,
    );

    let vault_id = env.vault_client.open(
        &env.admin,
        &env.crop_id,
        &symbol_short!("MAIZE"),
        &1u64,
        &symbol_short!("NG"),
        &30_000i128,
        &200_000i128,
    );

    // Check vault state
    let state = env.vault_client.get_vault(&vault_id);
    assert_eq!(state.owner, env.admin);
    assert_eq!(state.collateral_amount, 30_000);
    assert_eq!(state.debt_amount, 200_000);
    assert_eq!(state.commodity, symbol_short!("MAIZE"));

    // Admin should have received USDC
    let usdc_client = MockTokenClient::new(&env.env, &env.usdc_id);
    assert_eq!(usdc_client.balance(&env.admin), 200_000);

    // Collateral should be locked in vault
    assert_eq!(
        env.crop_client.balance(&env.admin),
        50_000 - 30_000
    );
    assert_eq!(
        env.crop_client.balance(&env.vault_id),
        30_000
    );

    // ── Repay ─────────────────────────────────────────────────────────────────
    env.vault_client
        .repay(&env.admin, &vault_id, &200_000i128);

    let state = env.vault_client.get_vault(&vault_id);
    assert_eq!(state.debt_amount, 0);
    assert_eq!(state.collateral_amount, 0);

    // Collateral returned to admin
    assert_eq!(env.crop_client.balance(&env.admin), 50_000);
    assert_eq!(env.crop_client.balance(&env.vault_id), 0);
}

#[test]
fn test_partial_repay() {
    let env = setup_env();

    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        50_000,
    );

    let vault_id = env.vault_client.open(
        &env.admin,
        &env.crop_id,
        &symbol_short!("MAIZE"),
        &1u64,
        &symbol_short!("NG"),
        &30_000i128,
        &200_000i128,
    );

    // Partial repay
    env.vault_client
        .repay(&env.admin, &vault_id, &80_000i128);

    let state = env.vault_client.get_vault(&vault_id);
    assert_eq!(state.debt_amount, 120_000);
    // Collateral still locked since debt > 0
    assert_eq!(state.collateral_amount, 30_000);
}

#[test]
fn test_liquidate_unhealthy_vault() {
    let env = setup_env();

    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        100,
    );

    // Open vault with very high LTV
    // Price of MAIZE = 10_000_000 per unit
    // Deposit 2 CropTokens = worth 20_000_000
    // Borrow 18_000_000 -> LTV = 90% (above 85%)
    let vault_id = env.vault_client.open(
        &env.admin,
        &env.crop_id,
        &symbol_short!("MAIZE"),
        &1u64,
        &symbol_short!("NG"),
        &2i128,
        &18_000_000i128,
    );

    // Liquidator needs USDC to pay off debt
    let liquidator = Address::generate(&env.env);
    let usdc_client = MockTokenClient::new(&env.env, &env.usdc_id);
    usdc_client.mint(&liquidator, &100_000_000i128);

    // Link liquidator's passport for the collateral transfer
    env.crop_client
        .link_passport(&liquidator, &1u64, &symbol_short!("NG"));

    // Liquidate
    env.vault_client.liquidate(&liquidator, &vault_id);

    let state = env.vault_client.get_vault(&vault_id);
    assert_eq!(state.debt_amount, 0);
    assert_eq!(state.collateral_amount, 0);

    // Liquidator received the collateral
    assert_eq!(env.crop_client.balance(&liquidator), 2);
}
