use soroban_sdk::{Env, Symbol, Vec};

use crate::DataKey;

pub fn add_jurisdiction(env: &Env, code: &Symbol) {
    let mut jurisdictions: Vec<Symbol> = env
        .storage()
        .instance()
        .get(&DataKey::AllowedJurisdictions)
        .unwrap_or(Vec::new(env));

    if !jurisdictions.contains(code) {
        jurisdictions.push_back(code.clone());
        env.storage()
            .instance()
            .set(&DataKey::AllowedJurisdictions, &jurisdictions);
    }

    env.events().publish(
        (Symbol::new(env, "JurisdictionAdded"), code.clone()),
        (),
    );
}

pub fn remove_jurisdiction(env: &Env, code: &Symbol) {
    let jurisdictions: Vec<Symbol> = env
        .storage()
        .instance()
        .get(&DataKey::AllowedJurisdictions)
        .unwrap_or(Vec::new(env));

    let mut new_jurisdictions: Vec<Symbol> = Vec::new(env);
    for j in jurisdictions.iter() {
        if j != *code {
            new_jurisdictions.push_back(j);
        }
    }

    env.storage()
        .instance()
        .set(&DataKey::AllowedJurisdictions, &new_jurisdictions);

    env.events().publish(
        (Symbol::new(env, "JurisdictionRemoved"), code.clone()),
        (),
    );
}

pub fn is_allowed(env: &Env, code: &Symbol) -> bool {
    let jurisdictions: Vec<Symbol> = env
        .storage()
        .instance()
        .get(&DataKey::AllowedJurisdictions)
        .unwrap_or(Vec::new(env));

    jurisdictions.contains(code)
}
