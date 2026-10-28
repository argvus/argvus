//! Animations pages: categories, leaves, custom curves and the shared draft.

use super::*;

#[test]
fn animations_root_lists_the_switch_options_and_the_five_categories() {
  let application = app(AppearancePage::Animations);
  let rows = application.rows();
  assert_eq!(rows[0].kind(), RowKind::Toggle { on: true }, "Enable");
  assert_eq!(rows[1].kind(), RowKind::Separator, "Options heading");
  assert_eq!(rows[2].label(), "Custom curves");
  assert_eq!(rows[3].label(), "Windows");
  assert_eq!(rows[4].label(), "Workspaces");
  assert_eq!(rows[5].label(), "Layers & popups");
  assert_eq!(rows[6].label(), "Borders & effects");
  assert_eq!(rows[7].label(), "Other animations");
  assert_eq!(
    rows.last().unwrap().id(),
    Some(&Item::Apply),
    "shared Apply row"
  );
}

#[test]
fn leaf_pages_show_only_the_rows_of_their_kind() {
  let opening = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Opening,
  })
  .page_rows();
  let labels: Vec<String> = opening.iter().map(|row| row.label().to_owned()).collect();
  assert_eq!(labels, ["Style", "Speed", "Curve", "Pop-in percentage"]);

  let moving = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Moving,
  })
  .page_rows();
  assert_eq!(moving.len(), 3, "Enable, Speed and Curve only");
  assert_eq!(moving[0].kind(), RowKind::Toggle { on: true });
}

#[test]
fn custom_curves_list_their_parameters_and_fixed_points() {
  let bezier = app(AppearancePage::AnimationsCurve {
    curve: AnimationCurve::Bezier,
  })
  .page_rows();
  assert_eq!(bezier.len(), 4, "x1, y1, x2, y2");

  let spring = app(AppearancePage::AnimationsCurve {
    curve: AnimationCurve::Spring,
  })
  .page_rows();
  assert_eq!(spring.len(), 3, "mass, stiffness, dampening");

  let linear = app(AppearancePage::AnimationsCurve {
    curve: AnimationCurve::Linear,
  })
  .page_rows();
  assert_eq!(linear.len(), 1, "fixed curve: points only");
  assert_eq!(linear[0].label(), "Control points");
}

#[test]
fn every_animations_page_ends_with_the_shared_apply_row() {
  let pages = [
    AppearancePage::Animations,
    AppearancePage::AnimationsCustomCurves,
    AppearancePage::AnimationsCurve {
      curve: AnimationCurve::Bezier,
    },
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::Windows,
    },
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Opening,
    },
  ];
  for page in pages {
    let application = app(page);
    let rows = application.rows();
    let apply = rows.last().unwrap();
    assert_eq!(apply.id(), Some(&Item::Apply), "{page:?}");
    assert!(
      !apply.is_selectable(),
      "clean draft: Apply is skipped ({page:?})"
    );
  }
}

#[test]
fn animation_edits_stay_in_the_shared_draft_until_apply() {
  let mut animations = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Opening,
  });
  select(
    &mut animations,
    Item::AnimationLeafStyle(AnimationLeaf::Opening),
  );
  animations.handle(KeyCode::Enter);
  assert_eq!(
    animations
      .animations_editor()
      .values
      .leaf(AnimationLeaf::Opening)
      .style,
    AnimationStyle::Gnomed,
    "Enter cycles the style forward"
  );
  assert_eq!(
    animations
      .state
      .animation_values
      .leaf(AnimationLeaf::Opening)
      .style,
    AnimationStyle::Popin,
    "stored value unchanged"
  );
  assert!(animations.has_pending_changes(), "Apply is enabled");
  assert!(animations.action.is_none(), "nothing applied");
}

#[test]
fn animation_switch_is_part_of_the_draft() {
  let mut animations = app(AppearancePage::Animations);
  select(&mut animations, Item::Animations);
  animations.handle(KeyCode::Enter);
  assert!(!animations.animations_editor().enabled);
  assert!(animations.state.animations, "stored switch unchanged");
  assert!(animations.has_pending_changes());
  assert!(animations.action.is_none());
}

#[test]
fn typed_animation_values_are_bounded_and_stay_in_the_draft() {
  let mut speed = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Opening,
  });
  select(
    &mut speed,
    Item::AnimationField(AnimationField::Speed(AnimationLeaf::Opening)),
  );
  speed.handle(KeyCode::Enter);
  assert!(matches!(
    speed.page,
    AppearancePage::Prompt {
      goal: PromptGoal::AnimationValue(_)
    }
  ));
  // Speed accepts 1–10: 11 keeps the prompt open.
  speed.handle(KeyCode::Char('1'));
  speed.handle(KeyCode::Char('1'));
  speed.handle(KeyCode::Enter);
  assert!(speed.prompt_error.is_some());
  assert!(matches!(speed.page, AppearancePage::Prompt { .. }));

  speed.handle(KeyCode::Backspace);
  speed.handle(KeyCode::Backspace);
  speed.handle(KeyCode::Char('7'));
  speed.handle(KeyCode::Enter);
  assert_eq!(
    speed.page,
    AppearancePage::AnimationsLeaf {
      leaf: AnimationLeaf::Opening
    }
  );
  assert_eq!(
    speed
      .animations_editor()
      .values
      .leaf(AnimationLeaf::Opening)
      .speed,
    7.0
  );
  assert_eq!(
    speed
      .state
      .animation_values
      .leaf(AnimationLeaf::Opening)
      .speed,
    4.1,
    "stored value unchanged"
  );
  assert!(speed.action.is_none(), "nothing applied");
}

#[test]
fn leaving_a_leaf_keeps_the_draft_and_leaving_the_root_asks() {
  let mut animations = app(AppearancePage::AnimationsLeaf {
    leaf: AnimationLeaf::Fade,
  });
  select(
    &mut animations,
    Item::AnimationLeafEnabled(AnimationLeaf::Fade),
  );
  animations.handle(KeyCode::Enter);
  assert!(
    !animations.back_discards_draft(),
    "sub-pages keep the draft"
  );

  animations.handle(KeyCode::Esc);
  assert_eq!(
    animations.page,
    AppearancePage::AnimationsGroup {
      group: AnimationGroup::LayersPopups
    }
  );
  assert!(animations.animations_draft.is_some(), "draft survives");

  animations.handle(KeyCode::Esc);
  assert_eq!(animations.page, AppearancePage::Animations);
  assert!(animations.confirm.is_none(), "root asks only when leaving");
  assert!(
    !animations.handle(KeyCode::Esc),
    "pending draft: Esc asks first"
  );
  assert!(animations.confirm.is_some(), "leaving the root asks first");
}
