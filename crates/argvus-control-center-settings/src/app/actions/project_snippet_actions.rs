//! Projects and snippets pages: project paths, snippet names and contents, and typing a snippet.

use super::*;

impl App {
  /// Opens the single-line editor for a new project path, or a new root.
  pub(super) fn open_project_editor(&mut self, root: bool) {
    let title = if root {
      tr(self.lang, "control_center.projects_add_root")
    } else {
      tr(self.lang, "control_center.projects_add_path")
    };
    self.admin.editor = Some(Editor::new(
      title.into(),
      String::new(),
      EditTarget::ProjectPath(root),
      false,
    ));
  }

  pub(super) fn open_new_snippet_name(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.snippets_name").into(),
      self.new_snippet_name.clone(),
      EditTarget::NewSnippetName,
      false,
    ));
  }

  pub(super) fn open_new_snippet_content(&mut self) {
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.snippets_content").into(),
      self.new_snippet_content.clone(),
      EditTarget::NewSnippetContent,
      false,
    ));
  }

  /// Opens the editor with a saved snippet's content; saving replaces it.
  pub(super) fn open_snippet_content(&mut self, index: usize) {
    let Some(entry) = self.snippets.entries.get(index) else {
      return;
    };
    self.admin.editor = Some(Editor::new(
      tr(self.lang, "control_center.snippets_content").into(),
      entry.content.clone(),
      EditTarget::SnippetContent(entry.name.clone()),
      false,
    ));
  }

  pub(super) fn add_snippet(&mut self) {
    let name = self.new_snippet_name.trim().to_owned();
    if name.is_empty() {
      self.fail(tr(self.lang, "control_center.snippets_name_required").to_string());
      return;
    }
    match snippets::add(&name, &self.new_snippet_content) {
      Ok(()) => {
        self.new_snippet_name.clear();
        self.new_snippet_content.clear();
        self.snippets = snippets::load();
        self.success(tr(self.lang, "control_center.snippets_saved").to_string());
      }
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn open_snippet_picker(&mut self) {
    match snippets::open_picker() {
      Ok(()) => self.success(tr(self.lang, "control_center.snippets_picker_opening").to_string()),
      Err(error) => self.fail(error),
    }
  }

  pub(super) fn type_snippet(&mut self, index: usize) {
    match snippets::type_position(index + 1) {
      Ok(()) => self.success(tr(self.lang, "control_center.snippets_typing").to_string()),
      Err(error) => self.fail(error),
    }
  }
}
