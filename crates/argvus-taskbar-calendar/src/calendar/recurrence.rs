use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct Recurrence {
  pub rrule: Option<String>,
  pub raw_properties: Vec<String>,
}

impl Recurrence {
  #[allow(dead_code)]
  pub fn is_empty(&self) -> bool {
    self.rrule.is_none() && self.raw_properties.is_empty()
  }
}
