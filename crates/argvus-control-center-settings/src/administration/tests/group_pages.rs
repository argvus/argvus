//! Group pages: members, rename and delete.

use super::*;

#[test]
fn group_page_has_save_after_fields_and_delete_in_the_danger_zone() {
  let app = app(Page::Group);
  let rows = app.admin.rows(Page::Group, Lang::for_locale("pt-BR"), "");
  let ids: Vec<Option<Item>> = rows.iter().map(|row| row.id().copied()).collect();
  assert_eq!(
    ids,
    vec![
      None,
      Some(Item::GroupName),
      None,
      Some(Item::Members),
      None,
      Some(Item::SaveGroup),
      None,
      Some(Item::DeleteGroup),
    ]
  );
  assert_eq!(rows[5].label(), "Salvar alterações");
  assert_eq!(rows[7].kind(), RowKind::Destructive);
  assert!(rows[6].is_section());
}

#[test]
fn group_open_keeps_original_name_for_rename_and_delete() {
  let mut app = app(Page::GroupList);
  app.admin.accounts = json!({
    "users": [],
    "groups": [{"name":"dev","gid":1000,"members":["alice"]}],
    "group_details": [{"name":"dev","gid":1000,"members":["alice"]}]
  });
  select(&mut app, Item::GroupEntry(0));
  press(&mut app, KeyCode::Enter);
  assert_eq!(app.page(), Page::Group);
  assert_eq!(text(&app.admin.group, "original"), "dev");
}

#[test]
/// Without administration rights the group pages are read-only: no row can run.
fn group_writes_are_disabled_for_non_administrators() {
  let mut app = app(Page::Group);
  app.admin.accounts["actor_is_admin"] = json!(false);
  assert!(
    app.rows().iter().all(|row| !row.is_selectable()),
    "no group row is selectable"
  );
}
