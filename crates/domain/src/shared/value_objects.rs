use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::errors::DomainError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UserId(Uuid);

impl UserId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn into_inner(self) -> Uuid {
        self.0
    }
}

impl From<Uuid> for UserId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl std::fmt::Display for UserId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    amount: Decimal,
    currency: String,
}

impl Money {
    pub fn new(amount: Decimal, currency: impl Into<String>) -> Result<Self, DomainError> {
        if amount < Decimal::ZERO {
            return Err(DomainError::NegativeMoney);
        }

        Ok(Self {
            amount,
            currency: currency.into(),
        })
    }

    pub fn zero(currency: impl Into<String>) -> Self {
        Self {
            amount: Decimal::ZERO,
            currency: currency.into(),
        }
    }

    pub fn amount(&self) -> Decimal {
        self.amount
    }

    pub fn currency(&self) -> &str {
        &self.currency
    }

    pub fn add(&self, other: &Money) -> Result<Self, DomainError> {
        if self.currency != other.currency {
            return Err(DomainError::CurrencyMismatch);
        }

        Ok(Self {
            amount: self.amount + other.amount,
            currency: self.currency.clone(),
        })
    }

    pub fn subtract(&self, other: &Money) -> Result<Self, DomainError> {
        if self.currency != other.currency {
            return Err(DomainError::CurrencyMismatch);
        }

        let new_amount = self.amount - other.amount;
        if new_amount < Decimal::ZERO {
            return Err(DomainError::NegativeMoney);
        }

        Ok(Self {
            amount: new_amount,
            currency: self.currency.clone(),
        })
    }

    pub fn multiply(&self, multiplier: i32) -> Result<Self, DomainError> {
        if multiplier < 0 {
            return Err(DomainError::NegativeMoney);
        }

        Ok(Self {
            amount: self.amount * Decimal::from(multiplier),
            currency: self.currency.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal_macros::dec;

    #[test]
    fn money_creation_success() {
        let money = Money::new(dec!(100.50), "USD").unwrap();
        assert_eq!(money.amount(), dec!(100.50));
        assert_eq!(money.currency(), "USD");
    }

    #[test]
    fn money_creation_fails_on_negative_amount() {
        let money = Money::new(dec!(-10.0), "USD");
        assert!(matches!(money, Err(DomainError::NegativeMoney)));
    }

    #[test]
    fn money_add_success() {
        let m1 = Money::new(dec!(10.0), "USD").unwrap();
        let m2 = Money::new(dec!(20.0), "USD").unwrap();
        let m3 = m1.add(&m2).unwrap();
        assert_eq!(m3.amount(), dec!(30.0));
        assert_eq!(m3.currency(), "USD");
    }

    #[test]
    fn money_add_fails_on_currency_mismatch() {
        let m1 = Money::new(dec!(10.0), "USD").unwrap();
        let m2 = Money::new(dec!(20.0), "EUR").unwrap();
        let result = m1.add(&m2);
        assert!(matches!(result, Err(DomainError::CurrencyMismatch)));
    }

    #[test]
    fn money_subtract_success() {
        let m1 = Money::new(dec!(30.0), "USD").unwrap();
        let m2 = Money::new(dec!(10.0), "USD").unwrap();
        let m3 = m1.subtract(&m2).unwrap();
        assert_eq!(m3.amount(), dec!(20.0));
    }

    #[test]
    fn money_subtract_fails_on_negative_result() {
        let m1 = Money::new(dec!(10.0), "USD").unwrap();
        let m2 = Money::new(dec!(30.0), "USD").unwrap();
        let result = m1.subtract(&m2);
        assert!(matches!(result, Err(DomainError::NegativeMoney)));
    }
}
