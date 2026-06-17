use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

use crate::{DataKey, HedgeState};

pub fn execute_settle(
    env: Env,
    hedge_id: u64,
    settlement_type: Symbol,
    caller: Address,
) {
    caller.require_auth();

    let mut hedge: HedgeState = env
        .storage()
        .instance()
        .get(&DataKey::Hedge(hedge_id))
        .expect("Hedge not found");

    assert_eq!(
        hedge.status,
        Symbol::new(&env, "Accepted"),
        "Hedge must be in Accepted status"
    );
    assert!(
        env.ledger().timestamp() >= hedge.expiry,
        "Hedge has not expired yet"
    );

    let revealed_price: i128 = env
        .storage()
        .instance()
        .get(&DataKey::RevealedPrice(hedge_id))
        .expect("Price must be revealed before settlement");

    let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).expect("CropToken not set");

    if settlement_type == Symbol::new(&env, "Physical") {
        let args: Vec<Val> = (
            hedge.farmer.clone(),
            hedge.buyer.clone(),
            hedge.quantity,
        )
            .into_val(&env);
        let _: () = env.invoke_contract(&crop_token, &Symbol::new(&env, "transfer"), args);

        hedge.status = Symbol::new(&env, "SettledPhysical");
    } else if settlement_type == Symbol::new(&env, "Cash") {
        let settlement_amount = hedge.quantity * revealed_price / 1_000_000_000i128;

        let args: Vec<Val> = (
            hedge.buyer.clone(),
            hedge.farmer.clone(),
            settlement_amount,
        )
            .into_val(&env);
        let _: () = env.invoke_contract(&crop_token, &Symbol::new(&env, "transfer"), args);

        hedge.status = Symbol::new(&env, "SettledCash");
    } else {
        panic!("Invalid settlement type");
    }

    env.storage()
        .instance()
        .set(&DataKey::Hedge(hedge_id), &hedge);

    env.events().publish(
        (Symbol::new(&env, "HedgeSettled"), hedge_id),
        settlement_type,
    );
}

pub fn execute_cancel(env: Env, hedge_id: u64, caller: Address) {
    caller.require_auth();

    let mut hedge: HedgeState = env
        .storage()
        .instance()
        .get(&DataKey::Hedge(hedge_id))
        .expect("Hedge not found");

    assert!(
        hedge.status == Symbol::new(&env, "Placed")
            || hedge.status == Symbol::new(&env, "Accepted"),
        "Hedge already settled or cancelled"
    );
    assert!(
        env.ledger().timestamp() < hedge.expiry,
        "Cannot cancel after expiry"
    );

    let penalty_bps: i128 = 1000;
    let penalty_amount = hedge.quantity * penalty_bps / 10000;

    if hedge.status == Symbol::new(&env, "Accepted") && penalty_amount > 0 {
        let crop_token: Address = env.storage().instance().get(&DataKey::CropToken).expect("CropToken not set");
        let args: Vec<Val> = (
            hedge.farmer.clone(),
            hedge.buyer.clone(),
            penalty_amount,
        )
            .into_val(&env);
        let _: () = env.invoke_contract(&crop_token, &Symbol::new(&env, "transfer"), args);
    }

    hedge.status = Symbol::new(&env, "Cancelled");
    env.storage()
        .instance()
        .set(&DataKey::Hedge(hedge_id), &hedge);

    env.events().publish(
        (Symbol::new(&env, "HedgeCancelled"), hedge_id),
        caller,
    );
}
