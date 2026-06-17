use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

use crate::DataKey;

pub fn execute_deposit(env: Env, user: Address, amount: i128) -> i128 {
    user.require_auth();
    assert!(amount > 0, "Deposit amount must be positive");

    let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).expect("CropToken not set");

    let args: Vec<Val> = (user.clone(), env.current_contract_address(), amount).into_val(&env);
    let _: () = env.invoke_contract(&crop_token, &Symbol::new(&env, "transfer"), args);

    let total_crop: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalCropDeposited)
        .unwrap_or(0);
    let total_hct: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalHctSupply)
        .unwrap_or(0);

    let hct_minted = if total_hct == 0 || total_crop == 0 {
        amount
    } else {
        amount * total_hct / total_crop
    };

    env.storage()
        .instance()
        .set(&DataKey::TotalCropDeposited, &(total_crop + amount));
    env.storage()
        .instance()
        .set(&DataKey::TotalHctSupply, &(total_hct + hct_minted));

    let user_hct_key = DataKey::BalanceHct(user.clone());
    let user_hct: i128 = env.storage().persistent().get(&user_hct_key).unwrap_or(0);
    env.storage()
        .persistent()
        .set(&user_hct_key, &(user_hct + hct_minted));

    env.events().publish(
        (Symbol::new(&env, "Deposited"), user),
        (amount, hct_minted),
    );

    hct_minted
}

pub fn execute_withdraw(env: Env, user: Address, hct_amount: i128) -> (i128, i128) {
    user.require_auth();
    assert!(hct_amount > 0, "Withdraw amount must be positive");

    let user_hct_key = DataKey::BalanceHct(user.clone());
    let user_hct: i128 = env.storage().persistent().get(&user_hct_key).unwrap_or(0);
    assert!(user_hct >= hct_amount, "Insufficient hCT balance");

    let total_hct: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalHctSupply)
        .expect("TotalHctSupply not set");
    let total_crop: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalCropDeposited)
        .expect("TotalCropDeposited not set");
    let total_yield: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalYieldUsdc)
        .unwrap_or(0);

    let crop_out = hct_amount * total_crop / total_hct;
    let yield_out = hct_amount * total_yield / total_hct;

    env.storage()
        .persistent()
        .set(&user_hct_key, &(user_hct - hct_amount));
    env.storage()
        .instance()
        .set(&DataKey::TotalHctSupply, &(total_hct - hct_amount));
    env.storage()
        .instance()
        .set(&DataKey::TotalCropDeposited, &(total_crop - crop_out));
    env.storage()
        .instance()
        .set(&DataKey::TotalYieldUsdc, &(total_yield - yield_out));

    let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).expect("CropToken not set");
    let crop_args: Vec<Val> =
        (env.current_contract_address(), user.clone(), crop_out).into_val(&env);
    let _: () = env.invoke_contract(&crop_token, &Symbol::new(&env, "transfer"), crop_args);

    if yield_out > 0 {
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).expect("USDC token not set");
        let usdc_args: Vec<Val> =
            (env.current_contract_address(), user.clone(), yield_out).into_val(&env);
        let _: () = env.invoke_contract(&usdc_token, &Symbol::new(&env, "transfer"), usdc_args);
    }

    env.events().publish(
        (Symbol::new(&env, "Withdrawn"), user),
        (crop_out, yield_out),
    );

    (crop_out, yield_out)
}
