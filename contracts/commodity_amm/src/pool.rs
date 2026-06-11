use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

use crate::{DataKey, PoolInfo};

pub fn execute_create_pool(env: &Env, commodity: Symbol) {
    assert!(
        !env.storage().instance().has(&DataKey::PoolInfo(commodity.clone())),
        "Pool already exists"
    );
    let pool = PoolInfo {
        commodity: commodity.clone(),
        reserve_crop: 0,
        reserve_usdc: 0,
        total_lp_supply: 0,
        created_at: env.ledger().timestamp(),
    };
    env.storage()
        .instance()
        .set(&DataKey::PoolInfo(commodity), &pool);
}

pub fn execute_swap(
    env: &Env,
    user: Address,
    commodity: Symbol,
    amount_in: i128,
    min_amount_out: i128,
    sell_crop: bool,
) -> i128 {
    user.require_auth();
    assert!(amount_in > 0, "Amount must be positive");
    assert!(min_amount_out >= 0, "Min amount out must be non-negative");

    let mut pool: PoolInfo = env
        .storage()
        .instance()
        .get(&DataKey::PoolInfo(commodity.clone()))
        .expect("Pool does not exist");

    let (token_in, reserve_in, reserve_out) = if sell_crop {
        let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).unwrap();
        (crop_token, pool.reserve_crop, pool.reserve_usdc)
    } else {
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        (usdc_token, pool.reserve_usdc, pool.reserve_crop)
    };

    let amount_out = crate::curve::calculate_swap(
        env,
        amount_in,
        reserve_in,
        reserve_out,
        sell_crop,
        env.ledger().timestamp(),
    );
    assert!(amount_out >= min_amount_out, "Slippage: amount_out below min");

    let token_out = if sell_crop {
        let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();
        usdc_token
    } else {
        let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).unwrap();
        crop_token
    };

    let transfer_args_in: Vec<Val> =
        (user.clone(), env.current_contract_address(), amount_in).into_val(env);
    let _: () = env.invoke_contract(&token_in, &Symbol::new(env, "transfer"), transfer_args_in);

    let transfer_args_out: Vec<Val> =
        (env.current_contract_address(), user.clone(), amount_out).into_val(env);
    let _: () = env.invoke_contract(&token_out, &Symbol::new(env, "transfer"), transfer_args_out);

    if sell_crop {
        pool.reserve_crop += amount_in;
        pool.reserve_usdc -= amount_out;
    } else {
        pool.reserve_usdc += amount_in;
        pool.reserve_crop -= amount_out;
    }
    env.storage()
        .instance()
        .set(&DataKey::PoolInfo(commodity.clone()), &pool);

    env.events().publish(
        (Symbol::new(env, "SwapExecuted"), commodity, user),
        (amount_in, amount_out),
    );

    amount_out
}

pub fn execute_add_liquidity(
    env: &Env,
    user: Address,
    commodity: Symbol,
    amount_crop: i128,
    amount_usdc: i128,
) -> (i128, i128, i128) {
    user.require_auth();
    assert!(amount_crop > 0 && amount_usdc > 0, "Amounts must be positive");

    let mut pool: PoolInfo = env
        .storage()
        .instance()
        .get(&DataKey::PoolInfo(commodity.clone()))
        .expect("Pool does not exist");

    let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).unwrap();
    let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

    let (actual_crop, actual_usdc) = if pool.total_lp_supply > 0 {
        let ideal_usdc = amount_crop * pool.reserve_usdc / pool.reserve_crop;
        if ideal_usdc <= amount_usdc {
            (amount_crop, ideal_usdc)
        } else {
            let ideal_crop = amount_usdc * pool.reserve_crop / pool.reserve_usdc;
            (ideal_crop, amount_usdc)
        }
    } else {
        (amount_crop, amount_usdc)
    };

    let lp_tokens = crate::curve::calculate_lp_tokens(
        actual_crop,
        actual_usdc,
        pool.reserve_crop,
        pool.reserve_usdc,
        pool.total_lp_supply,
    );
    assert!(lp_tokens > 0, "No LP tokens to mint");

    let transfer_crop_args: Vec<Val> =
        (user.clone(), env.current_contract_address(), actual_crop).into_val(env);
    let _: () =
        env.invoke_contract(&crop_token, &Symbol::new(env, "transfer"), transfer_crop_args);

    let transfer_usdc_args: Vec<Val> =
        (user.clone(), env.current_contract_address(), actual_usdc).into_val(env);
    let _: () =
        env.invoke_contract(&usdc_token, &Symbol::new(env, "transfer"), transfer_usdc_args);

    pool.reserve_crop += actual_crop;
    pool.reserve_usdc += actual_usdc;
    pool.total_lp_supply += lp_tokens;
    env.storage()
        .instance()
        .set(&DataKey::PoolInfo(commodity.clone()), &pool);

    let lp_balance_key = DataKey::BalanceLP(user.clone(), commodity.clone());
    let current_lp: i128 = env
        .storage()
        .persistent()
        .get(&lp_balance_key)
        .unwrap_or(0);
    env.storage()
        .persistent()
        .set(&lp_balance_key, &(current_lp + lp_tokens));

    let total_lp_key = DataKey::TotalLpSupply(commodity.clone());
    let total_lp: i128 = env.storage().instance().get(&total_lp_key).unwrap_or(0);
    env.storage()
        .instance()
        .set(&total_lp_key, &(total_lp + lp_tokens));

    env.events().publish(
        (Symbol::new(env, "LiquidityAdded"), commodity, user),
        (actual_crop, actual_usdc, lp_tokens),
    );

    (actual_crop, actual_usdc, lp_tokens)
}

pub fn execute_remove_liquidity(
    env: &Env,
    user: Address,
    commodity: Symbol,
    lp_tokens: i128,
    min_crop: i128,
    min_usdc: i128,
) -> (i128, i128) {
    user.require_auth();
    assert!(lp_tokens > 0, "LP tokens must be positive");

    let pool: PoolInfo = env
        .storage()
        .instance()
        .get(&DataKey::PoolInfo(commodity.clone()))
        .expect("Pool does not exist");

    let lp_balance_key = DataKey::BalanceLP(user.clone(), commodity.clone());
    let current_lp: i128 = env
        .storage()
        .persistent()
        .get(&lp_balance_key)
        .unwrap_or(0);
    assert!(
        current_lp >= lp_tokens,
        "Insufficient LP tokens"
    );

    let crop_out = lp_tokens * pool.reserve_crop / pool.total_lp_supply;
    let usdc_out = lp_tokens * pool.reserve_usdc / pool.total_lp_supply;

    assert!(crop_out >= min_crop, "Crop output below minimum");
    assert!(usdc_out >= min_usdc, "USDC output below minimum");

    let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).unwrap();
    let usdc_token: Address = env.storage().instance().get(&DataKey::UsdcToken).unwrap();

    let transfer_crop_args: Vec<Val> =
        (env.current_contract_address(), user.clone(), crop_out).into_val(env);
    let _: () =
        env.invoke_contract(&crop_token, &Symbol::new(env, "transfer"), transfer_crop_args);

    let transfer_usdc_args: Vec<Val> =
        (env.current_contract_address(), user.clone(), usdc_out).into_val(env);
    let _: () =
        env.invoke_contract(&usdc_token, &Symbol::new(env, "transfer"), transfer_usdc_args);

    env.storage()
        .persistent()
        .set(&lp_balance_key, &(current_lp - lp_tokens));

    let mut new_pool = pool.clone();
    new_pool.reserve_crop -= crop_out;
    new_pool.reserve_usdc -= usdc_out;
    new_pool.total_lp_supply -= lp_tokens;
    env.storage()
        .instance()
        .set(&DataKey::PoolInfo(commodity), &new_pool);

    let total_lp_key = DataKey::TotalLpSupply(new_pool.commodity.clone());
    let total_lp: i128 = env.storage().instance().get(&total_lp_key).unwrap_or(0);
    env.storage()
        .instance()
        .set(&total_lp_key, &(total_lp - lp_tokens));

    env.events().publish(
        (Symbol::new(env, "LiquidityRemoved"), new_pool.commodity, user),
        (crop_out, usdc_out),
    );

    (crop_out, usdc_out)
}
