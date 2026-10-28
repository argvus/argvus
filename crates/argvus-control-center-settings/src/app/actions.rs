//! Menu navigation and row activation of the Settings pages.
//!
//! Keys reach the page's [`MenuState`]; the resulting [`MenuEvent`] carries
//! the row's [`Item`], and the handlers below keep each option's original
//! effect and timing (immediate, confirmed, or draft).
use crossterm::event::KeyCode;

use argvus_control_center_apps::catalog::Category;
use argvus_tui::hints::{HintContext, confirm_hints, hints};
use argvus_tui::menu::{MenuEvent, MenuState, Row, RowKind};

use super::keybinding_capture::{
  keybinding_category_order, keybinding_is_cheatsheet_entry, keybinding_label, keybinding_section,
  keybinding_section_order,
};
use super::keyboard_draft;
use super::labels::search_matches;
use super::rows::LANGUAGES;
use super::{App, PendingAction, Status, StatusKind};
use crate::administration::{EditTarget, Editor};
use crate::config::fonts::SettingKind;
use crate::i18n::{Lang, tr};
use crate::item::{Item, RatbagRow};
use crate::navigation::Page;
use crate::system::date::{CalendarDate, DateOrder};
use crate::system::gpg_keys;
use crate::system::snippets;
use crate::system::ssh_keys;
use crate::system::window_rules;
use crate::system::{keybindings, keyboard, locale, time};
use argvus_control_center_core::clipboard;

mod gpg_key_actions;
mod keybinding_actions;
mod menu;
mod preference_actions;
mod project_snippet_actions;
mod ssh_key_actions;
mod window_rule_actions;

pub(crate) use menu::setting_values;
