//! Period-consistent financial calculations; rates are percentages per period.
use crate::engine::Error;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operation {
  Simple,
  Compound,
  #[default]
  Payment,
  Present,
  Future,
  Change,
}
impl Operation {
  pub const ALL: [Self; 6] = [
    Self::Simple,
    Self::Compound,
    Self::Payment,
    Self::Present,
    Self::Future,
    Self::Change,
  ];
  pub fn key(self) -> &'static str {
    match self {
      Self::Simple => "finance.simple",
      Self::Compound => "finance.compound",
      Self::Payment => "finance.payment",
      Self::Present => "finance.present",
      Self::Future => "finance.future",
      Self::Change => "finance.change",
    }
  }
}
/// `principal` is the current amount (future amount for Present); payments are at period end.
pub fn calculate(
  operation: Operation,
  principal: f64,
  rate: f64,
  periods: f64,
  payment: f64,
) -> Result<f64, Error> {
  if ![principal, rate, periods, payment]
    .into_iter()
    .all(f64::is_finite)
  {
    return Err(Error::Domain);
  }
  if operation == Operation::Change {
    return if principal == 0.0 {
      Err(Error::Domain)
    } else {
      finite((rate - principal) / principal * 100.0)
    };
  }
  if periods <= 0.0 || rate <= -100.0 {
    return Err(Error::Domain);
  }
  let rate = rate / 100.0;
  let growth = (periods * rate.ln_1p()).exp();
  let annuity = if rate == 0.0 {
    periods
  } else {
    (periods * rate.ln_1p()).exp_m1() / rate
  };
  let result = match operation {
    Operation::Simple => principal * (1.0 + rate * periods),
    Operation::Compound => principal * growth,
    Operation::Payment => principal * growth / annuity,
    Operation::Present => (principal + payment * annuity) / growth,
    Operation::Future => principal * growth + payment * annuity,
    Operation::Change => return Err(Error::Domain),
  };
  finite(result)
}
fn finite(value: f64) -> Result<f64, Error> {
  if value.is_finite() {
    Ok(value)
  } else {
    Err(Error::Domain)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn known_cases() {
    for (operation, principal, rate, periods, payment, expected) in [
      (Operation::Simple, 1000.0, 10.0, 2.0, 0.0, 1200.0),
      (Operation::Compound, 1000.0, 10.0, 2.0, 0.0, 1210.0),
      (Operation::Payment, 1200.0, 0.0, 12.0, 0.0, 100.0),
      (Operation::Present, 1210.0, 10.0, 2.0, 0.0, 1000.0),
      (Operation::Future, 0.0, 0.0, 12.0, 100.0, 1200.0),
      (Operation::Change, 100.0, 125.0, 0.0, 0.0, 25.0),
    ] {
      assert!(
        (calculate(operation, principal, rate, periods, payment).unwrap() - expected).abs() < 1e-8
      );
    }
  }
  #[test]
  fn payment_and_present_value_agree_for_positive_negative_and_zero_rates() {
    for rate in [-1.0, 0.0, 0.00000001, 1.0, 10.0] {
      let payment = calculate(Operation::Payment, 1000.0, rate, 12.0, 0.0).unwrap();
      let principal = calculate(Operation::Present, 0.0, rate, 12.0, payment).unwrap();
      assert!((principal - 1000.0).abs() < 1e-8);
    }
    assert!(
      (calculate(Operation::Payment, 1000.0, 1.0, 12.0, 0.0).unwrap() - 88.8487886783416).abs()
        < 1e-10
    );
  }
  #[test]
  fn invalid_cases() {
    for (rate, periods) in [
      (-100.0, 12.0),
      (10.0, 0.0),
      (f64::NAN, 12.0),
      (1000.0, 100000.0),
    ] {
      assert!(calculate(Operation::Payment, 1000.0, rate, periods, 0.0).is_err());
    }
    assert!(calculate(Operation::Change, 0.0, 5.0, 0.0, 0.0).is_err());
  }
}
