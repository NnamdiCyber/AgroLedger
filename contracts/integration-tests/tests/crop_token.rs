mod common;
use common::*;
use soroban_sdk::{symbol_short, testutils::Address as _, Address};

#[test]
fn test_issue_crop_token() {
    let env = setup_env();

    let lot_id = symbol_short!("LOT1");
    let commodity = symbol_short!("MAIZE");
    let quantity = 10_000u64;

    issue_crop_token(&env, lot_id.clone(), commodity.clone(), quantity);

    assert_eq!(env.crop_client.balance(&env.admin), quantity as i128);

    let meta = env.crop_client.get_lot_metadata(&lot_id);
    assert_eq!(meta.commodity, commodity);
    assert_eq!(meta.quantity_kg, quantity);
    assert_eq!(meta.lot_id, lot_id);
}

#[test]
fn test_transfer_tokens() {
    let env = setup_env();
    let user = Address::generate(&env.env);
    let recipient = Address::generate(&env.env);

    // Link passports for compliance
    env.crop_client
        .link_passport(&user, &1u64, &symbol_short!("NG"));
    env.crop_client
        .link_passport(&recipient, &1u64, &symbol_short!("NG"));

    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        5_000,
    );

    env.crop_client
        .transfer(&env.admin, &user, &2_000);
    assert_eq!(env.crop_client.balance(&user), 2_000);

    env.crop_client
        .transfer(&user, &recipient, &1_000);
    assert_eq!(env.crop_client.balance(&user), 1_000);
    assert_eq!(env.crop_client.balance(&recipient), 1_000);
}

#[test]
fn test_burn_tokens() {
    let env = setup_env();

    issue_crop_token(
        &env,
        symbol_short!("LOT1"),
        symbol_short!("MAIZE"),
        1_000,
    );

    env.crop_client.burn(&symbol_short!("LOT1"));
    assert_eq!(env.crop_client.balance(&env.admin), 0);
}
