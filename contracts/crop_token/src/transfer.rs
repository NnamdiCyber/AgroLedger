use soroban_sdk::{Address, Env, IntoVal, Symbol, Val, Vec};

pub fn execute_transfer(env: Env, from: Address, to: Address, amount: i128) {
    from.require_auth();

    let from_balance: i128 = env.storage().persistent().get(&super::DataKey::Balance(from.clone())).unwrap_or(0);
    assert!(from_balance >= amount, "Insufficient balance");

    // Compliance check: only if sender has a linked passport (user-initiated transfers)
    // Contract-to-contract transfers (e.g. vault liquidation) skip compliance
    if env.storage().instance().has(&super::DataKey::AddressPassport(from.clone())) {
        let compliance_registry: Address = env.storage().instance().get(&super::DataKey::ComplianceRegistry).expect("ComplianceRegistry not set");
        let passport_data: (u64, Symbol) = env.storage()
            .instance()
            .get(&super::DataKey::AddressPassport(from.clone()))
            .expect("Passport not linked to address");

        let args: Vec<Val> = (passport_data.0, passport_data.1.clone()).into_val(&env);
        let compliant: bool = env.invoke_contract(
            &compliance_registry,
            &Symbol::new(&env, "verify"),
            args,
        );
        assert!(compliant, "Compliance check failed");
    }

    let to_key = super::DataKey::Balance(to.clone());
    let to_balance: i128 = env.storage().persistent().get(&to_key).unwrap_or(0);

    env.storage().persistent().set(&super::DataKey::Balance(from.clone()), &(from_balance - amount));
    env.storage().persistent().set(&to_key, &(to_balance + amount));

    env.events().publish(
        (Symbol::new(&env, "Transfer"), from, to),
        amount,
    );
}
