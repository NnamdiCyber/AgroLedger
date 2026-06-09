use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

use crate::{DataKey, PathQuote, PathResult, TravelRuleData};

const FEE_BPS: i128 = 15;

fn calculate_fee(amount: i128) -> i128 {
    if amount <= 0 {
        return 0;
    }
    (amount * FEE_BPS) / 10000
}

pub fn execute_route(
    env: Env,
    from: Address,
    to: Address,
    send_asset: Address,
    recv_asset: Address,
    amount: i128,
    travel_rule_data: TravelRuleData,
) -> PathResult {
    from.require_auth();

    assert!(amount > 0, "Amount must be positive");

    let compliance_registry: Address = env
        .storage()
        .instance()
        .get(&DataKey::ComplianceRegistry)
        .unwrap();

    let args: Vec<Val> = (
        travel_rule_data.passport_id,
        travel_rule_data.jurisdiction.clone(),
    )
        .into_val(&env);
    let compliant: bool = env.invoke_contract(
        &compliance_registry,
        &Symbol::new(&env, "verify"),
        args,
    );
    assert!(compliant, "Compliance check failed");

    let args: Vec<Val> = (amount, travel_rule_data.jurisdiction.clone()).into_val(&env);
    let travel_valid: bool = env.invoke_contract(
        &compliance_registry,
        &Symbol::new(&env, "validate_travel_rule"),
        args,
    );
    assert!(travel_valid, "Travel rule check failed");

    let fee = calculate_fee(amount);
    let amount_after_fee = amount - fee;

    let args: Vec<Val> = (from.clone(), env.current_contract_address(), amount).into_val(&env);
    let _: () = env.invoke_contract(&send_asset, &Symbol::new(&env, "transfer"), args);

    let args: Vec<Val> = (
        env.current_contract_address(),
        to.clone(),
        amount_after_fee,
    )
        .into_val(&env);
    let _: () = env.invoke_contract(&recv_asset, &Symbol::new(&env, "transfer"), args);

    let counter: u64 = env
        .storage()
        .instance()
        .get(&DataKey::RouteCounter)
        .unwrap_or(0);
    let route_id = counter + 1;
    env.storage()
        .instance()
        .set(&DataKey::RouteCounter, &route_id);

    env.events().publish(
        (Symbol::new(&env, "RouteExecuted"), route_id, from.clone()),
        (amount, fee),
    );

    PathResult {
        from,
        to,
        send_asset,
        recv_asset,
        amount_sent: amount,
        amount_received: amount_after_fee,
        fee,
    }
}

pub fn compute_estimate(
    env: &Env,
    send_asset: Address,
    recv_asset: Address,
    amount: i128,
) -> Vec<PathQuote> {
    assert!(amount > 0, "Amount must be positive");

    let fee = calculate_fee(amount);
    let amount_after_fee = amount - fee;

    let mut quotes: Vec<PathQuote> = Vec::new(env);
    quotes.push_back(PathQuote {
        send_asset,
        recv_asset,
        amount_out: amount_after_fee,
        fee,
    });
    quotes
}
