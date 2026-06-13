mod common;
use common::*;
use soroban_sdk::{symbol_short, testutils::{Address as _, Ledger}, Address, Symbol};

#[test]
fn test_farmer_to_buyer_flow() {
    let env = setup_env();

    let buyer = Address::generate(&env.env);
    let farmer = env.admin.clone();
    let usdc_client = MockTokenClient::new(&env.env, &env.usdc_id);

    // ── Step 1: Issue CropTokens (farmer deposits lot) ────────────────────────
    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        50_000,
    );
    assert_eq!(env.crop_client.balance(&farmer), 50_000);

    // ── Step 2: Open vault → borrow USDC ─────────────────────────────────────
    let vault_id = env.vault_client.open(
        &farmer,
        &env.crop_id,
        &symbol_short!("MAIZE"),
        &1u64,
        &symbol_short!("NG"),
        &30_000i128,  // collateral
        &200_000i128, // borrow
    );

    let vault_state = env.vault_client.get_vault(&vault_id);
    assert_eq!(vault_state.collateral_amount, 30_000);
    assert_eq!(vault_state.debt_amount, 200_000);
    assert_eq!(usdc_client.balance(&farmer), 200_000);

    // Farmer has 20_000 CropTokens remaining
    assert_eq!(env.crop_client.balance(&farmer), 20_000);

    // ── Step 3: Place forward hedge ───────────────────────────────────────────
    let price: i128 = 500_000_000;
    let salt: i128 = 12345;
    let commitment = make_commitment(&env.env, price, salt);

    env.crop_client
        .link_passport(&buyer, &1u64, &symbol_short!("NG"));

    let hedge_id = env.hedge_client.place_hedge(
        &buyer,
        &symbol_short!("MAIZE"),
        &5_000i128,
        &commitment,
        &2_000_000u64,
    );

    // ── Step 4: Farmer accepts hedge ─────────────────────────────────────────
    env.hedge_client.accept_hedge(&hedge_id, &farmer);
    env.hedge_client.reveal(&hedge_id, &price, &salt);

    let hedge_state = env.hedge_client.get_hedge(&hedge_id);
    assert_eq!(hedge_state.status, Symbol::new(&env.env, "Accepted"));

    // ── Step 5: Settle hedge ─────────────────────────────────────────────────
    env.env.ledger().set_timestamp(2_000_001);

    let farmer_bal_before = env.crop_client.balance(&farmer);
    let buyer_bal_before = env.crop_client.balance(&buyer);

    env.hedge_client
        .settle(&hedge_id, &Symbol::new(&env.env, "Physical"), &farmer);

    // Farmer delivers 5_000 CropTokens to buyer
    assert_eq!(
        env.crop_client.balance(&farmer),
        farmer_bal_before - 5_000
    );
    assert_eq!(env.crop_client.balance(&buyer), buyer_bal_before + 5_000);

    let hedge_state = env.hedge_client.get_hedge(&hedge_id);
    assert_eq!(hedge_state.status, Symbol::new(&env.env, "SettledPhysical"));

    // ── Step 6: Repay vault ──────────────────────────────────────────────────
    // Farmer uses borrowed USDC to repay
    let vault_before = env.vault_client.get_vault(&vault_id);
    assert_eq!(vault_before.debt_amount, 200_000);

    env.vault_client.repay(&farmer, &vault_id, &200_000i128);

    let vault_after = env.vault_client.get_vault(&vault_id);
    assert_eq!(vault_after.debt_amount, 0);
    assert_eq!(vault_after.collateral_amount, 0);

    // Collateral returned to farmer
    assert_eq!(
        env.crop_client.balance(&farmer),
        farmer_bal_before - 5_000 + 30_000 // had 20_000 - 5_000 delivered + 30_000 returned
    );
}
