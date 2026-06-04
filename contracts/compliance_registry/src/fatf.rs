

const TRAVEL_RULE_THRESHOLD: i128 = 10_000_000_000;

pub fn validate_travel_rule(amount: i128) -> bool {
    amount <= TRAVEL_RULE_THRESHOLD
}
