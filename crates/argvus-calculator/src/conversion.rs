//! Offline physical-unit conversions through canonical SI values.
use crate::engine::Error;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Category {
  #[default]
  Length,
  Mass,
  Temperature,
  Area,
  Volume,
  Time,
  Speed,
  Data,
}
#[derive(Clone, Copy)]
pub struct Unit {
  pub symbol: &'static str,
  factor: f64,
  offset: f64,
}
const fn unit(symbol: &'static str, factor: f64) -> Unit {
  Unit {
    symbol,
    factor,
    offset: 0.0,
  }
}
impl Category {
  pub const ALL: [Self; 8] = [
    Self::Length,
    Self::Mass,
    Self::Temperature,
    Self::Area,
    Self::Volume,
    Self::Time,
    Self::Speed,
    Self::Data,
  ];
  pub fn key(self) -> &'static str {
    match self {
      Self::Length => "conversion.length",
      Self::Mass => "conversion.mass",
      Self::Temperature => "conversion.temperature",
      Self::Area => "conversion.area",
      Self::Volume => "conversion.volume",
      Self::Time => "conversion.time",
      Self::Speed => "conversion.speed",
      Self::Data => "conversion.data",
    }
  }
  pub fn units(self) -> &'static [Unit] {
    match self {
      Self::Length => {
        const UNITS: &[Unit] = &[
          unit("m", 1.0),
          unit("km", 1000.0),
          unit("cm", 0.01),
          unit("mm", 0.001),
          unit("in", 0.0254),
          unit("ft", 0.3048),
          unit("yd", 0.9144),
          unit("mi", 1609.344),
        ];
        UNITS
      }
      Self::Mass => {
        const UNITS: &[Unit] = &[
          unit("kg", 1.0),
          unit("g", 0.001),
          unit("mg", 0.000001),
          unit("t", 1000.0),
          unit("lb", 0.45359237),
          unit("oz", 0.028349523125),
        ];
        UNITS
      }
      Self::Temperature => {
        const UNITS: &[Unit] = &[
          Unit {
            symbol: "°C",
            factor: 1.0,
            offset: 273.15,
          },
          Unit {
            symbol: "°F",
            factor: 5.0 / 9.0,
            offset: 255.3722222222222,
          },
          unit("K", 1.0),
        ];
        UNITS
      }
      Self::Area => {
        const UNITS: &[Unit] = &[
          unit("m²", 1.0),
          unit("km²", 1000000.0),
          unit("cm²", 0.0001),
          unit("ha", 10000.0),
          unit("ft²", 0.09290304),
          unit("acre", 4046.8564224),
        ];
        UNITS
      }
      Self::Volume => {
        const UNITS: &[Unit] = &[
          unit("L", 1.0),
          unit("mL", 0.001),
          unit("m³", 1000.0),
          unit("US gal", 3.785411784),
          unit("US fl oz", 0.0295735295625),
        ];
        UNITS
      }
      Self::Time => {
        const UNITS: &[Unit] = &[
          unit("s", 1.0),
          unit("ms", 0.001),
          unit("min", 60.0),
          unit("h", 3600.0),
          unit("d", 86400.0),
        ];
        UNITS
      }
      Self::Speed => {
        const UNITS: &[Unit] = &[
          unit("m/s", 1.0),
          unit("km/h", 1.0 / 3.6),
          unit("mph", 0.44704),
          unit("kn", 1852.0 / 3600.0),
        ];
        UNITS
      }
      Self::Data => {
        const UNITS: &[Unit] = &[
          unit("bit", 1.0),
          unit("B", 8.0),
          unit("kbit", 1000.0),
          unit("kB", 8000.0),
          unit("MB", 8000000.0),
          unit("GB", 8000000000.0),
          unit("TB", 8000000000000.0),
          unit("Kibit", 1024.0),
          unit("KiB", 8192.0),
          unit("MiB", 8388608.0),
          unit("GiB", 8589934592.0),
          unit("TiB", 8796093022208.0),
        ];
        UNITS
      }
    }
  }
}
pub fn calculate(value: f64, category: Category, from: usize, to: usize) -> Result<f64, Error> {
  let source = category.units().get(from).ok_or(Error::Domain)?;
  let target = category.units().get(to).ok_or(Error::Domain)?;
  let canonical = value.mul_add(source.factor, source.offset);
  if category == Category::Temperature && canonical < -1e-10 {
    return Err(Error::Domain);
  }
  let result = (canonical - target.offset) / target.factor;
  if value.is_finite() && result.is_finite() {
    Ok(result)
  } else {
    Err(Error::Domain)
  }
}
#[cfg(test)]
mod tests {
  use super::*;
  #[test]
  fn round_trips_all_units() {
    for category in Category::ALL {
      for index in 0..category.units().len() {
        let result = calculate(42.0, category, 0, index).unwrap();
        assert!((calculate(result, category, index, 0).unwrap() - 42.0).abs() < 1e-8);
      }
    }
  }
  #[test]
  fn offsets_and_data() {
    assert!((calculate(32.0, Category::Temperature, 1, 0).unwrap()).abs() < 1e-10);
    assert!((calculate(100.0, Category::Temperature, 0, 1).unwrap() - 212.0).abs() < 1e-10);
    assert_eq!(calculate(1.0, Category::Data, 8, 1).unwrap(), 1024.0);
    assert_eq!(calculate(1.0, Category::Data, 3, 1).unwrap(), 1000.0);
    assert_eq!(calculate(1.0, Category::Data, 1, 0).unwrap(), 8.0);
  }
  #[test]
  fn rejects_invalid() {
    assert!(calculate(-274.0, Category::Temperature, 0, 1).is_err());
    assert!(calculate(1.0, Category::Length, 100, 0).is_err());
    assert!(calculate(f64::INFINITY, Category::Mass, 0, 1).is_err());
  }
}
