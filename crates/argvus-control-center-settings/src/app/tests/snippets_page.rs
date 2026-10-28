//! Snippets page.

use super::*;

#[test]
fn snippets_page_offers_type_and_remove_for_each_entry() {
  let mut app = app(Page::Snippets);
  app.snippets = crate::system::snippets::Snippets {
    available: true,
    entries: vec![crate::system::snippets::SnippetEntry {
      name: "email".into(),
      content: "you@example.com".into(),
    }],
  };
  let rows = app.rows();
  assert!(row_of(&rows, Item::AddSnippet).is_selectable());
  assert!(row_of(&rows, Item::OpenSnippetPicker).is_selectable());
  assert!(row_of(&rows, Item::SnippetContent(0)).is_selectable());
  assert!(row_of(&rows, Item::TypeSnippet(0)).is_selectable());
  assert!(row_of(&rows, Item::RemoveSnippet(0)).is_selectable());
}

#[test]
fn snippets_page_without_the_launcher_only_informs() {
  let mut app = app(Page::Snippets);
  app.snippets = crate::system::snippets::Snippets::default();
  let rows = app.rows();
  assert!(rows.iter().all(|row| row.id().is_none()));
}
