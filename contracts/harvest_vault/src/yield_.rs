use soroban_sdk::{Address, Env, Symbol};

use crate::DataKey;

const BASE_APY_BPS: u32 = 800;
const SECONDS_PER_YEAR: u64 = 31536000;

pub fn execute_accrue_yield(env: Env, admin: Address) -> i128 {
    admin.require_auth();
    accrue_yield_internal(env)
}

fn accrue_yield_internal(env: Env) -> i128 {
    let last_accrual: u64 = env
        .storage()
        .instance()
        .get(&DataKey::LastAccrual)
        .unwrap_or(env.ledger().timestamp());
    let now = env.ledger().timestamp();

    if now <= last_accrual {
        return 0;
    }

    let total_crop: i128 = env
        .storage()
        .instance()
        .get(&DataKey::TotalCropDeposited)
        .unwrap_or(0);
    if total_crop == 0 {
        env.storage().instance().set(&DataKey::LastAccrual, &now);
        return 0;
    }

    let elapsed = now - last_accrual;

    let yield_amount = (total_crop * (BASE_APY_BPS as i128) * (elapsed as i128))
        / (SECONDS_PER_YEAR as i128 * 10000);

    if yield_amount > 0 {
        let total_yield: i128 = env
            .storage()
            .instance()
            .get(&DataKey::TotalYieldUsdc)
            .unwrap_or(0);
        env.storage()
            .instance()
            .set(&DataKey::TotalYieldUsdc, &(total_yield + yield_amount));

        env.events().publish(
            (Symbol::new(&env, "YieldAccrued"),),
            yield_amount,
        );
    }

    env.storage().instance().set(&DataKey::LastAccrual, &now);

    yield_amount
}

pub fn execute_get_apy(_env: &Env) -> u32 {
    BASE_APY_BPS
}

pub fn execute_rebalance(env: Env, admin: Address) {
    admin.require_auth();
    accrue_yield_internal(env);
}
