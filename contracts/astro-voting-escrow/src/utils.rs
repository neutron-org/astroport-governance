use crate::contract::{MAX_LOCK_TIME, WEEK};
use crate::error::ContractError;
use astroport::asset::addr_validate_to_lower;
use cosmwasm_std::{
    Addr, Decimal, Deps, Fraction, Order, OverflowError, Pair, StdError, StdResult, Uint128,
    Uint256,
};
use cw_storage_plus::{Bound, U64Key};
use std::convert::TryInto;

use crate::state::{Point, BLACKLIST, CONFIG, HISTORY, SLOPE_CHANGES};

/// # Description
/// Checks the time is within limits
pub(crate) fn time_limits_check(time: u64) -> Result<(), ContractError> {
    if !(WEEK..=MAX_LOCK_TIME).contains(&time) {
        Err(ContractError::LockTimeLimitsError {})
    } else {
        Ok(())
    }
}

/// # Description
/// Calculates how many periods are withing specified time. Time should be in seconds.
pub(crate) fn get_period(time: u64) -> u64 {
    time / WEEK
}

/// # Description
/// Checks the sender is xASTRO token
pub(crate) fn xastro_token_check(deps: Deps, sender: Addr) -> Result<(), ContractError> {
    let config = CONFIG.load(deps.storage)?;
    if sender != config.deposit_token_addr {
        Err(ContractError::Unauthorized {})
    } else {
        Ok(())
    }
}

pub(crate) fn blacklist_check(deps: Deps, addr: &Addr) -> Result<(), ContractError> {
    let blacklist = BLACKLIST.load(deps.storage)?;
    if blacklist.contains(addr) {
        Err(ContractError::AddressBlacklisted(addr.to_string()))
    } else {
        Ok(())
    }
}

/// # Description
/// Trait is intended for Decimal rounding problem elimination
trait DecimalRoundedCheckedMul {
    fn checked_mul(self, other: Uint128) -> Result<Uint128, OverflowError>;
}

impl DecimalRoundedCheckedMul for Decimal {
    fn checked_mul(self, other: Uint128) -> Result<Uint128, OverflowError> {
        if self.is_zero() || other.is_zero() {
            return Ok(Uint128::zero());
        }
        let numerator = other.full_mul(self.numerator());
        let multiply_ratio = numerator / Uint256::from(self.denominator());
        if multiply_ratio > Uint256::from(Uint128::MAX) {
            Err(OverflowError::new(
                cosmwasm_std::OverflowOperation::Mul,
                self,
                other,
            ))
        } else {
            let mut result: Uint128 = multiply_ratio.try_into().unwrap();
            let rem: Uint128 = numerator
                .checked_rem(Uint256::from(self.denominator()))
                .unwrap()
                .try_into()
                .unwrap();
            // 0.5 in Decimal
            if rem.u128() >= 500000000000000000_u128 {
                result += Uint128::from(1_u128);
            }
            Ok(result)
        }
    }
}

/// # Description
/// Main calculation function by formula: previous_power - slope*(x - previous_x)
pub(crate) fn calc_voting_power(point: &Point, period: u64) -> Uint128 {
    let shift = point
        .slope
        .checked_mul(Uint128::from(period - point.start))
        .unwrap_or_else(|_| Uint128::zero());
    point
        .power
        .checked_sub(shift)
        .unwrap_or_else(|_| Uint128::zero())
}

/// # Description
/// Coefficient calculation where 0 [`WEEK`] equals to 1 and [`MAX_LOCK_TIME`] equals to 2.5.
pub(crate) fn calc_coefficient(interval: u64) -> Decimal {
    // coefficient = 1 + 1.5 * (end - start) / MAX_LOCK_TIME
    Decimal::one() + Decimal::from_ratio(15_u64 * interval, get_period(MAX_LOCK_TIME) * 10)
}

/// # Description
/// Fetches last checkpoint in [`HISTORY`] for given address.
pub(crate) fn fetch_last_checkpoint(
    deps: Deps,
    addr: &Addr,
    period_key: &U64Key,
) -> StdResult<Option<Pair<Point>>> {
    HISTORY
        .prefix(addr.clone())
        .range(
            deps.storage,
            None,
            Some(Bound::Inclusive(period_key.wrapped.clone())),
            Order::Ascending,
        )
        .last()
        .transpose()
}

/// # Description
/// Helper function for deserialization
pub(crate) fn deserialize_pair(pair: StdResult<Pair<Decimal>>) -> StdResult<(u64, Decimal)> {
    let (period_serialized, change) = pair?;
    let period_bytes: [u8; 8] = period_serialized
        .try_into()
        .map_err(|_| StdError::generic_err("Deserialization error"))?;
    Ok((u64::from_be_bytes(period_bytes), change))
}

/// # Description
/// Fetches all slope changes between last_slope_change and period.
pub(crate) fn fetch_slope_changes(
    deps: Deps,
    last_slope_change: u64,
    period: u64,
) -> StdResult<Vec<(u64, Decimal)>> {
    SLOPE_CHANGES
        .range(
            deps.storage,
            Some(Bound::Exclusive(U64Key::new(last_slope_change).wrapped)),
            Some(Bound::Inclusive(U64Key::new(period).wrapped)),
            Order::Ascending,
        )
        .map(deserialize_pair)
        .collect()
}

/// # Description
/// Bulk validation and converting [`String`] -> [`Addr`] of array with addresses.
/// If any address is invalid returns [`StdError`].
pub(crate) fn validate_addresses(deps: Deps, addresses: &[String]) -> StdResult<Vec<Addr>> {
    addresses
        .iter()
        .map(|addr| addr_validate_to_lower(deps.api, addr))
        .collect()
}
