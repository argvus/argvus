use super::config_validation::validate_animation_setting;
use super::*;

mod migration;
mod store;
mod validation;

impl ConfigStore {
  fn set_explicit_for_test(&self, pointer: &str, value: Value) -> ConfigResult<()> {
    self.update(pointer, value).map(|_| ())
  }
}
