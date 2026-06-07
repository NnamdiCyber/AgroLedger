use soroban_sdk::{Address, Env, Symbol, Val, Vec, IntoVal};

pub fn execute_transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();

    // Compliance check
    let compliance_registry: Address = env.storage().instance().get(&super::DataKey::ComplianceRegistry).unwrap();
    
    // For now, we assume passport_id is 1 for testing purposes or retrieved from somewhere
    // In a real implementation, we'd need a mapping from Address to passport_id
    // For this sprint, we'll use a placeholder or assume the user has a passport registered.
    // Let's assume passport_id 1 and jurisdiction "NG" for the sake of the compliance gate demonstration.
    let passport_id = 1u64; // Placeholder
    let jurisdiction = Symbol::new(&env, "NG"); // Placeholder
    
    let args: Vec<Val> = (passport_id, jurisdiction).into_val(&env);
    let compliant: bool = env.invoke_contract(
        &compliance_registry,
        &Symbol::new(&env, "verify"),
        args,
    );
    assert!(compliant, "Compliance check failed");

    // Perform transfer
    let from_key = super::DataKey::Balance(from.clone());
    let to_key = super::DataKey::Balance(to.clone());

    let from_balance: i128 = env.storage().persistent().get(&from_key).unwrap_or(0);
    let to_balance: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);

    assert!(from_balance >= amount, "Insufficient balance");

    env.storage().persistent().set(&from_key, &(from_balance - amount));
    env.storage().persistent().set(&to_key, &(to_balance + amount));

    env.events().publish(
        (Symbol::new(&env, "Transfer"), from, to),
        amount,
    );
}
