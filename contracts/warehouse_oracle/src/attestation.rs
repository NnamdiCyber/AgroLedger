use soroban_sdk::{Address, Env, Symbol, Vec};
use crate::{DataKey, InspectorSet, LotState};

pub fn execute_submit_lot(
    env: Env,
    warehouse_id: Symbol,
    lot_id: Symbol,
    commodity: Symbol,
    quantity_kg: u64,
    inspector_sigs: Vec<Address>,
) -> u64 {
    let inspectors: InspectorSet = env.storage().instance().get(&DataKey::InspectorSet).unwrap();

    let required = if quantity_kg > 50_000 {
        if inspectors.threshold >= 3 {
            inspectors.threshold
        } else {
            3
        }
    } else {
        inspectors.threshold
    };

    let count = inspector_sigs.len();
    if count < required {
        panic!("insufficient signatures");
    }

    for i in 0..count {
        inspector_sigs.get(i).unwrap().require_auth();
    }

    let counter = env.storage().instance().get(&DataKey::LotCounter).unwrap_or(0) + 1;
    let lot = LotState {
        warehouse_id: warehouse_id.clone(),
        lot_id: lot_id.clone(),
        commodity: commodity.clone(),
        quantity_kg,
        approved: true,
        approved_at: env.ledger().timestamp(),
    };
    env.storage().instance().set(&DataKey::Lot(counter), &lot);
    env.storage().instance().set(&DataKey::LotCounter, &counter);
    env.storage()
        .instance()
        .set(&DataKey::LotLookup(warehouse_id.clone(), lot_id.clone()), &true);

    env.events().publish(
        (Symbol::new(&env, "LotSubmitted"), counter),
        (warehouse_id, lot_id, commodity, quantity_kg),
    );

    counter
}
