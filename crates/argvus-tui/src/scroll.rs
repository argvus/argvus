//! Scrollbar geometry shared by the Control Center pages.

/// Maps a scroll offset onto the position range of `ratatui::widgets::ScrollbarState`.
///
/// `ScrollbarState` draws the thumb at its end only when its position is
/// `content_length - 1`, but the last scroll offset of a view is
/// `rows - viewport`. Scaling the offset by that ratio puts the thumb at the
/// bottom exactly when the view shows its last row.
pub fn thumb_position(offset: usize, rows: usize, viewport: usize) -> usize {
  let max_offset = rows.saturating_sub(viewport);
  if max_offset == 0 {
    return 0;
  }
  offset.min(max_offset) * rows.saturating_sub(1) / max_offset
}

#[cfg(test)]
mod tests {
  use super::thumb_position;

  #[test]
  fn thumb_reaches_the_bottom_on_the_last_row() {
    // 30 rows in a 10-row viewport: offsets run 0..=20, the thumb spans 0..=29.
    assert_eq!(thumb_position(0, 30, 10), 0);
    assert_eq!(thumb_position(20, 30, 10), 29);
    assert!(thumb_position(10, 30, 10) < 29);
  }

  #[test]
  fn thumb_stays_at_the_top_when_rows_fit() {
    assert_eq!(thumb_position(0, 5, 10), 0);
  }
}
