use soroban_sdk::{Address, Bytes, BytesN, Env, Symbol};

use crate::{DataKey, HedgeState};

pub fn execute_place_hedge(
    env: Env,
    buyer: Address,
    commodity: Symbol,
    quantity: i128,
    commitment: BytesN<32>,
    expiry: u64,
) -> u64 {
    buyer.require_auth();
    assert!(quantity > 0, "Quantity must be positive");
    assert!(expiry > env.ledger().timestamp(), "Expiry must be in the future");

    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::HedgeCounter)
        .unwrap_or(0);
    let hedge_id = counter + 1;

    let hedge = HedgeState {
        buyer: buyer.clone(),
        farmer: buyer.clone(),
        commodity: commodity.clone(),
        quantity,
        commitment: commitment.clone(),
        expiry,
        status: Symbol::new(&env, "Placed"),
        placed_at: env.ledger().timestamp(),
    };

    env.storage()
        .instance()
        .set(&DataKey::Hedge(hedge_id), &hedge);
    env.storage()
        .instance()
        .set(&DataKey::HedgeCounter, &hedge_id);

    env.events().publish(
        (Symbol::new(&env, "HedgePlaced"), hedge_id),
        (buyer, commodity, quantity, expiry),
    );

    hedge_id
}

pub fn execute_accept_hedge(env: Env, hedge_id: u64, farmer: Address) {
    farmer.require_auth();

    let mut hedge: HedgeState = env
        .storage()
        .instance()
        .get(&DataKey::Hedge(hedge_id))
        .unwrap();

    assert_eq!(
        hedge.status,
        Symbol::new(&env, "Placed"),
        "Hedge must be in Placed status"
    );
    assert!(
        env.ledger().timestamp() < hedge.expiry,
        "Hedge has expired"
    );

    hedge.farmer = farmer.clone();
    hedge.status = Symbol::new(&env, "Accepted");

    env.storage()
        .instance()
        .set(&DataKey::Hedge(hedge_id), &hedge);

    env.events().publish(
        (Symbol::new(&env, "HedgeAccepted"), hedge_id),
        farmer,
    );
}

pub fn execute_reveal(env: Env, hedge_id: u64, price: i128, salt: i128) {
    let hedge: HedgeState = env
        .storage()
        .instance()
        .get(&DataKey::Hedge(hedge_id))
        .unwrap();

    assert_eq!(
        hedge.status,
        Symbol::new(&env, "Accepted"),
        "Hedge must be in Accepted status"
    );

    let price_arr = price.to_be_bytes();
    let salt_arr = salt.to_be_bytes();
    let mut input = Bytes::new(&env);
    input.append(&Bytes::from_slice(&env, &price_arr));
    input.append(&Bytes::from_slice(&env, &salt_arr));
    let computed_hash = env.crypto().sha256(&input);

    let computed_hash_bytes: BytesN<32> = computed_hash.into();
    assert_eq!(
        computed_hash_bytes, hedge.commitment,
        "Revealed price and salt do not match commitment"
    );

    env.storage()
        .instance()
        .set(&DataKey::RevealedPrice(hedge_id), &price);
}
