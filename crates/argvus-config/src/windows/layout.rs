//! Floating geometry uses the compositor's effective gaps and reserved areas.
use super::*;

pub(super) fn gaps(css: &str) -> ConfigResult<[i64; 4]> {
  let values = css
    .split_whitespace()
    .map(str::parse::<i64>)
    .collect::<Result<Vec<_>, _>>()
    .map_err(|error| invalid(format!("invalid compositor gaps: {error}")))?;
  let edges = match values.as_slice() {
    [all] => [*all; 4],
    [vertical, horizontal] => [*vertical, *horizontal, *vertical, *horizontal],
    [top, horizontal, bottom] => [*top, *horizontal, *bottom, *horizontal],
    [top, right, bottom, left] => [*top, *right, *bottom, *left],
    _ => return Err(invalid("invalid compositor gap edges")),
  };
  Ok(edges.map(|edge| edge.max(0)))
}

/// The compositor gaps already compensate the primary bar's inner margin.
/// The opposite bar reserves its inner margin too; do not count it twice.
pub(super) fn compensate_secondary(mut gaps: [i64; 4], bar: &Value) -> [i64; 4] {
  let (edge, margin) = match bar["position"].as_str() {
    Some("top") => (0, "margin-bottom"),
    Some("bottom") => (2, "margin-top"),
    _ => return gaps,
  };
  let inner = bar[margin].as_i64().unwrap_or(0).max(0);
  gaps[edge] = gaps[edge].saturating_sub(inner).max(0);
  gaps
}

pub(super) fn geometry(
  window: &Value,
  output: &Value,
  gaps: [i64; 4],
  border: i64,
  title: i64,
  maximized: bool,
) -> ([i64; 2], [i64; 2]) {
  let scale = output["scale"]
    .as_f64()
    .filter(|scale| *scale > 0.0)
    .unwrap_or(1.0);
  // Hyprland exposes transformed physical dimensions; quarter-turn outputs
  // exchange axes before conversion to logical coordinates.
  let (mut width, mut height) = (
    output["width"].as_f64().unwrap_or(1.0),
    output["height"].as_f64().unwrap_or(1.0),
  );
  if output["transform"].as_i64().unwrap_or(0) % 2 != 0 {
    std::mem::swap(&mut width, &mut height);
  }
  let reserve = |index: usize| output["reserved"][index].as_i64().unwrap_or(0);
  let x = output["x"].as_i64().unwrap_or(0) + reserve(0) + gaps[3] + border;
  let y = output["y"].as_i64().unwrap_or(0) + reserve(1) + gaps[0] + border + title;
  let available = [
    ((width / scale) as i64 - reserve(0) - reserve(2) - gaps[3] - gaps[1] - 2 * border).max(1),
    ((height / scale) as i64 - reserve(1) - reserve(3) - gaps[0] - gaps[2] - 2 * border - title)
      .max(1),
  ];
  if maximized {
    return ([x, y], available);
  }
  let size = [
    window["size"][0]
      .as_i64()
      .unwrap_or(1)
      .clamp(1, available[0]),
    window["size"][1]
      .as_i64()
      .unwrap_or(1)
      .clamp(1, available[1]),
  ];
  let at = [
    window["at"][0]
      .as_i64()
      .unwrap_or(x)
      .clamp(x, x + available[0] - size[0]),
    window["at"][1]
      .as_i64()
      .unwrap_or(y)
      .clamp(y, y + available[1] - size[1]),
  ];
  (at, size)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn gaps_expand_css_edges() {
    assert_eq!(gaps("18").unwrap(), [18; 4]);
    assert_eq!(gaps("10 20").unwrap(), [10, 20, 10, 20]);
    assert_eq!(gaps("10 20 30").unwrap(), [10, 20, 30, 20]);
    assert_eq!(gaps("0 18 18 18").unwrap(), [0, 18, 18, 18]);
    assert!(gaps("oops").is_err());
  }

  #[test]
  fn opposite_bar_margin_is_not_counted_twice() {
    let bottom = json!({"position":"bottom", "margin-top":18,"margin-bottom":18});
    assert_eq!(
      compensate_secondary([0, 18, 18, 18], &bottom),
      [0, 18, 0, 18]
    );
    let top = json!({"position":"top", "margin-top":18,"margin-bottom":18});
    assert_eq!(compensate_secondary([18, 18, 0, 18], &top), [0, 18, 0, 18]);
    assert_eq!(
      compensate_secondary([0, 18, 4, 18], &bottom),
      [0, 18, 0, 18]
    );
    assert_eq!(
      compensate_secondary([0, 18, 18, 18], &json!({"position":"bottom"})),
      [0, 18, 18, 18]
    );
  }

  #[test]
  fn window_has_equal_visible_spacing_above_and_below() {
    let output = json!({"x":0,"y":0,"width":1920,"height":1080,"scale":1,"reserved":[0,64,0,64]});
    for (gaps, bar) in [
      (
        [0, 18, 18, 18],
        json!({"position":"bottom","margin-top":18}),
      ),
      (
        [18, 18, 0, 18],
        json!({"position":"top","margin-bottom":18}),
      ),
    ] {
      let (at, size) = geometry(
        &json!({}),
        &output,
        compensate_secondary(gaps, &bar),
        1,
        28,
        true,
      );
      // Primary/secondary visible edges: top ends at 46, bottom starts at 1034.
      let above = at[1] - 28 - 1 - 46;
      let below = 1034 - (at[1] + size[1] + 1);
      assert_eq!(above, 18);
      assert_eq!(below, above);
    }
  }

  #[test]
  fn spaced_bounds_include_both_bars_title_and_border() {
    let output = json!({"x":0,"y":0,"width":1920,"height":1080,"scale":1,"reserved":[0,46,0,46]});
    let window = json!({"at":[0,0],"size":[1920,1080]});
    let expected = ([19, 93], [1882, 922]);
    assert_eq!(geometry(&window, &output, [18; 4], 1, 28, false), expected);
    assert_eq!(geometry(&window, &output, [18; 4], 1, 28, true), expected);
    let small = json!({"at":[200,200],"size":[600,400]});
    assert_eq!(
      geometry(&small, &output, [18; 4], 1, 28, false),
      ([200, 200], [600, 400])
    );
  }

  #[test]
  fn rotated_scaled_monitor_uses_logical_bounds() {
    let output = json!({"x":-900,"y":20,"width":1920,"height":1080,"scale":1.5,"transform":1,"reserved":[0,30,0,30]});
    assert_eq!(
      geometry(&json!({}), &output, [10; 4], 2, 28, true),
      ([-888, 90], [696, 1168])
    );
  }
}
