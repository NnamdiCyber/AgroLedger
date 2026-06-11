use soroban_sdk::Env;

const FEE_BPS: i128 = 30;
const SEASONAL_MIN_BPS: i128 = 9500;
const SEASONAL_MAX_BPS: i128 = 10500;
const SECONDS_PER_DAY: u64 = 86400;
const DAYS_PER_YEAR: u64 = 365;

fn day_of_year(timestamp: u64) -> u64 {
    (timestamp / SECONDS_PER_DAY) % DAYS_PER_YEAR
}

fn seasonal_factor_bps(timestamp: u64) -> i128 {
    let day = day_of_year(timestamp);
    let half_year = DAYS_PER_YEAR / 2;
    if day < half_year {
        let progress = day as i128;
        let last_step = (half_year - 1) as i128;
        SEASONAL_MIN_BPS + progress * (10000 - SEASONAL_MIN_BPS) / last_step
    } else {
        let progress = (day - half_year) as i128;
        let last_step = (DAYS_PER_YEAR - 1 - half_year) as i128;
        10000 + progress * (SEASONAL_MAX_BPS - 10000) / last_step
    }
}

pub fn calculate_swap(
    _env: &Env,
    amount_in: i128,
    reserve_in: i128,
    reserve_out: i128,
    sell_crop: bool,
    timestamp: u64,
) -> i128 {
    assert!(amount_in > 0, "Amount must be positive");
    assert!(reserve_in > 0 && reserve_out > 0, "Insufficient liquidity");

    let amount_in_with_fee = amount_in * (10000 - FEE_BPS) / 10000;
    let numerator = amount_in_with_fee * reserve_out;
    let denominator = reserve_in + amount_in_with_fee;
    let base_amount = numerator / denominator;

    let factor = seasonal_factor_bps(timestamp);
    if sell_crop {
        base_amount * factor / 10000
    } else {
        base_amount * 10000 / factor
    }
}

pub fn calculate_lp_tokens(
    crop_amount: i128,
    usdc_amount: i128,
    reserve_crop: i128,
    reserve_usdc: i128,
    total_lp_supply: i128,
) -> i128 {
    if total_lp_supply == 0 {
        let product = crop_amount * usdc_amount;
        let sqrt = integer_sqrt(product);
        if sqrt < 1000 {
            1000
        } else {
            sqrt
        }
    } else {
        let crop_share = crop_amount * total_lp_supply / reserve_crop;
        let usdc_share = usdc_amount * total_lp_supply / reserve_usdc;
        if crop_share < usdc_share {
            crop_share
        } else {
            usdc_share
        }
    }
}

fn integer_sqrt(n: i128) -> i128 {
    if n < 2 {
        return n;
    }
    let mut x = n;
    let mut y = (x + 1) / 2;
    while y < x {
        x = y;
        y = (x + n / x) / 2;
    }
    x
}

#[cfg(test)]
mod test {
    extern crate std;
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_day_of_year() {
        assert_eq!(day_of_year(0), 0);
        assert_eq!(day_of_year(86400), 1);
        assert_eq!(day_of_year(86400 * 364), 364);
        assert_eq!(day_of_year(86400 * 365), 0);
        assert_eq!(day_of_year(86400 * 366), 1);
    }

    #[test]
    fn test_seasonal_factor_range() {
        let jan = 0u64;
        let jul = 86400 * 182;
        let dec = 86400 * 364;

        let jan_factor = seasonal_factor_bps(jan);
        let jul_factor = seasonal_factor_bps(jul);
        let dec_factor = seasonal_factor_bps(dec);

        assert_eq!(jan_factor, SEASONAL_MIN_BPS);
        assert!(jul_factor >= 10000 && jul_factor <= SEASONAL_MAX_BPS);
        assert_eq!(dec_factor, SEASONAL_MAX_BPS);
    }

    #[test]
    fn test_calculate_swap_basic() {
        let env = Env::default();
        let ts = 86400 * 182;
        let out = calculate_swap(&env, 1000, 100000, 50000, true, ts);
        assert!(out > 0);
        assert!(out < 50000);
    }

    #[test]
    fn test_seasonal_variance() {
        let env = Env::default();
        let pre_harvest = 0u64;
        let post_harvest = 86400 * 364;

        let pre_out = calculate_swap(&env, 1000, 100000, 50000, true, pre_harvest);
        let post_out = calculate_swap(&env, 1000, 100000, 50000, true, post_harvest);

        assert!(post_out > pre_out, "Post-harvest should give more output");
    }

    #[test]
    fn test_price_impact() {
        let env = Env::default();
        let ts = 86400 * 182;

        let small = calculate_swap(&env, 10000, 1_000_000, 500_000, true, ts);
        let large = calculate_swap(&env, 500_000, 1_000_000, 500_000, true, ts);

        assert!(small > 0);
        assert!(large > 0);

        let per_unit_small = small * 10000 / 10000;
        let per_unit_large = large * 10000 / 500_000;

        assert!(per_unit_small > per_unit_large, "Large swap should have worse per-unit price");
    }

    #[test]
    fn test_sell_crop_vs_buy_crop() {
        let env = Env::default();
        let ts = 86400 * 182;
        let amount = 10000i128;
        let r_crop = 1_000_000i128;
        let r_usdc = 500_000i128;

        let crop_for_usdc = calculate_swap(&env, amount, r_crop, r_usdc, true, ts);
        let usdc_for_crop = calculate_swap(&env, crop_for_usdc, r_usdc - crop_for_usdc, r_crop + amount, false, ts);

        assert!(usdc_for_crop < amount, "Round-trip should lose value to fees");
    }

    #[test]
    fn test_integer_sqrt() {
        assert_eq!(integer_sqrt(0), 0);
        assert_eq!(integer_sqrt(1), 1);
        assert_eq!(integer_sqrt(4), 2);
        assert_eq!(integer_sqrt(9), 3);
        assert_eq!(integer_sqrt(100), 10);
        assert_eq!(integer_sqrt(10000), 100);
    }
}
