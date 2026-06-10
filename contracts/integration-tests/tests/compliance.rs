mod common;
use common::*;
use soroban_sdk::{symbol_short, BytesN};

#[test]
fn test_register_verify_revoke() {
    let env = setup_env();

    // Passport 1 was registered in setup with jurisdiction NG
    assert!(env
        .passport_client
        .verify(&1u64, &symbol_short!("NG")));

    // Wrong jurisdiction fails
    assert!(!env
        .passport_client
        .verify(&1u64, &symbol_short!("US")));

    // Revoke
    env.passport_client.revoke(&1u64);
    assert!(!env
        .passport_client
        .verify(&1u64, &symbol_short!("NG")));
}

#[test]
fn test_register_new_passport() {
    let env = setup_env();

    let nullifier = BytesN::from_array(&env.env, &[3u8; 32]);
    let proof = BytesN::from_array(&env.env, &[4u8; 32]);

    let pid = env
        .passport_client
        .register(&nullifier, &proof, &symbol_short!("GH"));

    assert_eq!(pid, 2u64); // Second passport (first was created in setup)

    assert!(env
        .passport_client
        .verify(&pid, &symbol_short!("GH")));

    assert!(!env
        .passport_client
        .verify(&pid, &symbol_short!("NG")));
}

#[test]
fn test_compliance_registry_verify() {
    let env = setup_env();

    // Verify via ComplianceRegistry (checks passport + jurisdiction allowlist)
    assert!(env
        .registry_client
        .verify(&1u64, &symbol_short!("NG")));

    // Blocked jurisdiction
    assert!(!env
        .registry_client
        .verify(&1u64, &symbol_short!("US")));

    // Revoked passport fails via registry too
    env.passport_client.revoke(&1u64);
    assert!(!env
        .registry_client
        .verify(&1u64, &symbol_short!("NG")));
}

#[test]
fn test_travel_rule_threshold() {
    let env = setup_env();

    // Under threshold
    assert!(env
        .registry_client
        .validate_travel_rule(&5_000_000_000i128, &symbol_short!("NG")));

    // At threshold
    assert!(env
        .registry_client
        .validate_travel_rule(&10_000_000_000i128, &symbol_short!("NG")));

    // Over threshold
    assert!(!env
        .registry_client
        .validate_travel_rule(&10_000_000_001i128, &symbol_short!("NG")));
}
