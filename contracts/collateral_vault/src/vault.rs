use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

use crate::{DataKey, VaultState};

pub fn execute_open(
    env: Env,
    user: Address,
    crop_token: Address,
    commodity: Symbol,
    passport_id: u64,
    jurisdiction: Symbol,
    collateral_amount: i128,
    borrow_amount_usdc: i128,
) -> u64 {
    user.require_auth();

    assert!(collateral_amount > 0, "Collateral must be positive");
    assert!(borrow_amount_usdc > 0, "Borrow amount must be positive");

    // Compliance check
    let compliance_registry: Address = env
        .storage()
        .instance()
        .get(&DataKey::ComplianceRegistry)
        .unwrap();
    let args: Vec<Val> = (passport_id, jurisdiction).into_val(&env);
    let compliant: bool = env.invoke_contract(
        &compliance_registry,
        &Symbol::new(&env, "verify"),
        args,
    );
    assert!(compliant, "Compliance check failed");

    // Lock CropToken collateral (transfer from user to vault)
    let args: Vec<Val> = (
        user.clone(),
        env.current_contract_address(),
        collateral_amount,
    )
        .into_val(&env);
    let _: () = env.invoke_contract(&crop_token, &Symbol::new(&env, "transfer"), args);

    // Lend USDC to user (transfer from vault to user)
    let usdc: Address = env
        .storage()
        .instance()
        .get(&DataKey::UsdcToken)
        .unwrap();
    let args: Vec<Val> = (
        env.current_contract_address(),
        user.clone(),
        borrow_amount_usdc,
    )
        .into_val(&env);
    let _: () = env.invoke_contract(&usdc, &Symbol::new(&env, "transfer"), args);

    // Create vault
    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::VaultCounter)
        .unwrap();
    let vault_id = counter + 1;
    let vault = VaultState {
        owner: user,
        crop_token,
        collateral_amount,
        debt_amount: borrow_amount_usdc,
        commodity,
        opened_at: env.ledger().timestamp(),
    };
    env.storage()
        .instance()
        .set(&DataKey::Vault(vault_id), &vault);
    env.storage()
        .instance()
        .set(&DataKey::VaultCounter, &vault_id);

    env.events().publish(
        (Symbol::new(&env, "VaultOpened"), vault_id),
        borrow_amount_usdc,
    );

    vault_id
}

pub fn execute_repay(env: Env, user: Address, vault_id: u64, amount: i128) {
    user.require_auth();

    assert!(amount > 0, "Repay amount must be positive");

    let mut vault: VaultState = env
        .storage()
        .instance()
        .get(&DataKey::Vault(vault_id))
        .unwrap();
    assert!(vault.owner == user, "Not vault owner");
    assert!(
        amount <= vault.debt_amount,
        "Repay amount exceeds debt"
    );

    // Transfer USDC from user to vault
    let usdc: Address = env
        .storage()
        .instance()
        .get(&DataKey::UsdcToken)
        .unwrap();
    let args: Vec<Val> = (user.clone(), env.current_contract_address(), amount).into_val(&env);
    let _: () = env.invoke_contract(&usdc, &Symbol::new(&env, "transfer"), args);

    vault.debt_amount -= amount;

    // If fully repaid, unlock collateral
    if vault.debt_amount == 0 {
        let args: Vec<Val> = (
            env.current_contract_address(),
            vault.owner.clone(),
            vault.collateral_amount,
        )
            .into_val(&env);
        let _: () = env.invoke_contract(
            &vault.crop_token,
            &Symbol::new(&env, "transfer"),
            args,
        );
        vault.collateral_amount = 0;
    }

    env.storage()
        .instance()
        .set(&DataKey::Vault(vault_id), &vault);

    env.events().publish(
        (Symbol::new(&env, "VaultRepaid"), vault_id),
        amount,
    );
}

pub fn execute_liquidate(env: Env, liquidator: Address, vault_id: u64) {
    liquidator.require_auth();

    let vault: VaultState = env
        .storage()
        .instance()
        .get(&DataKey::Vault(vault_id))
        .unwrap();

    // Check LTV > 85% using oracle price
    let ltv =
        crate::ltv::compute_ltv(&env, vault.collateral_amount, &vault.commodity, vault.debt_amount);
    assert!(ltv > 85, "Vault is not liquidatable");

    // Liquidator pays off the debt
    let usdc: Address = env
        .storage()
        .instance()
        .get(&DataKey::UsdcToken)
        .unwrap();
    let args: Vec<Val> = (
        liquidator.clone(),
        env.current_contract_address(),
        vault.debt_amount,
    )
        .into_val(&env);
    let _: () = env.invoke_contract(&usdc, &Symbol::new(&env, "transfer"), args);

    // Liquidator receives the collateral
    let args: Vec<Val> = (
        env.current_contract_address(),
        liquidator.clone(),
        vault.collateral_amount,
    )
        .into_val(&env);
    let _: () = env.invoke_contract(
        &vault.crop_token,
        &Symbol::new(&env, "transfer"),
        args,
    );

    // Zero out vault
    let mut closed = vault.clone();
    closed.debt_amount = 0;
    closed.collateral_amount = 0;
    env.storage()
        .instance()
        .set(&DataKey::Vault(vault_id), &closed);

    env.events().publish(
        (Symbol::new(&env, "VaultLiquidated"), vault_id),
        vault.collateral_amount,
    );
}
