mod common;
use common::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address};

#[test]
fn test_amm_full_flow() {
    let env = setup_env();

    let user = Address::generate(&env.env);
    let usdc_client = MockTokenClient::new(&env.env, &env.usdc_id);

    // Issue 2 lots of 50,000 each (max per lot is 50,000 without multi-sig)
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
    usdc_client.mint(&env.admin, &50_000i128);

    // ── Add liquidity ─────────────────────────────────────────────────────────
    let (crop_deposited, usdc_deposited, lp_tokens) = env.amm_client.add_liquidity(
        &env.admin,
        &symbol_short!("MAIZE"),
        &100_000i128,
        &50_000i128,
    );

    assert_eq!(crop_deposited, 100_000);
    assert_eq!(usdc_deposited, 50_000);
    assert!(lp_tokens > 0);

    let pool = env.amm_client.get_pool(&symbol_short!("MAIZE"));
    assert_eq!(pool.reserve_crop, 100_000);
    assert_eq!(pool.reserve_usdc, 50_000);

    let lp_balance = env.amm_client.get_lp_balance(&env.admin, &symbol_short!("MAIZE"));
    assert_eq!(lp_balance, lp_tokens);

    assert_eq!(env.crop_client.balance(&env.admin), 0);
    assert_eq!(env.crop_client.balance(&env.amm_id), 100_000);
    assert_eq!(usdc_client.balance(&env.admin), 0);
    assert_eq!(usdc_client.balance(&env.amm_id), 50_000);

    // ── User swaps CropTokens for USDC ────────────────────────────────────────
    issue_crop_token(
        &env,
        symbol_short!("LOT3"),
        symbol_short!("MAIZE"),
        10_000,
    );
    env.crop_client
        .link_passport(&user, &1u64, &symbol_short!("NG"));
    env.crop_client.transfer(&env.admin, &user, &5_000i128);

    let amount_out = env.amm_client.swap(
        &user,
        &symbol_short!("MAIZE"),
        &5_000i128,
        &0i128,
        &true,
    );

    assert!(amount_out > 0);
    assert!(amount_out < 5_000);
    assert_eq!(env.crop_client.balance(&user), 0);

    let pool_after_swap = env.amm_client.get_pool(&symbol_short!("MAIZE"));
    assert_eq!(pool_after_swap.reserve_crop, 105_000);

    // ── User swaps USDC back for CropTokens ───────────────────────────────────
    let usdc_balance = usdc_client.balance(&user);
    assert!(usdc_balance > 0, "User should have USDC from first swap");

    let crop_out = env.amm_client.swap(
        &user,
        &symbol_short!("MAIZE"),
        &usdc_balance,
        &0i128,
        &false,
    );

    assert!(crop_out > 0);
    assert_eq!(usdc_client.balance(&user), 0, "All USDC should be spent");

    // ── Remove liquidity ──────────────────────────────────────────────────────
    let (crop_removed, usdc_removed) = env.amm_client.remove_liquidity(
        &env.admin,
        &symbol_short!("MAIZE"),
        &lp_tokens,
        &0i128,
        &0i128,
    );

    assert!(crop_removed > 0);
    assert!(usdc_removed > 0);

    let pool_final = env.amm_client.get_pool(&symbol_short!("MAIZE"));
    assert_eq!(pool_final.reserve_crop, 0);
    assert_eq!(pool_final.reserve_usdc, 0);
}
