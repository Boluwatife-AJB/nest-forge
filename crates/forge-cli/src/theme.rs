//! Visual styling for CLI output

use console::{style, StyledObject};

// Status indicators
pub fn icon_success() -> StyledObject<&'static str> {
  style("✓").green().bold()
}

pub fn _icon_skip() -> StyledObject<&'static str> {
  style("~").yellow()
}

pub fn icon_dry_run() -> StyledObject<&'static str> {
  style("◆").cyan().bold()
}

pub fn icon_warning() -> StyledObject<&'static str> {
  style("▲").yellow().bold()
}

pub fn _icon_error() -> StyledObject<&'static str> {
  style("✕").red().bold()
}

pub fn icon_info() -> StyledObject<&'static str> {
  style("◆").cyan()
}

// Action Labels

pub fn label_create() -> StyledObject<&'static str> {
  style("CREATE").green().bold()
}

pub fn label_create_dry() -> StyledObject<&'static str> {
  style("CREATE").cyan()
}

pub fn label_skip() -> StyledObject<&'static str> {
  style("SKIP ").yellow()
}

// Tree drawing characters

pub fn tree_branch() -> &'static str {
  if cfg!(windows) {"+--"} else {"├──"}
}

pub fn tree_last() -> &'static str {
  if cfg!(windows) {"+--"} else {"└──"}
}

// Typography

pub fn _heading(s: &str) -> String {
  style(s).bold().underlined().to_string()
}

pub fn _dim(s: &str) -> String {
  style(s).dim().to_string()
}

pub fn _highlight(s: &str) -> String {
  style(s).cyan().to_string()
}

pub fn _success(s: &str) -> String {
  style(s).green().to_string()
}

pub fn muted(s: &str) -> String {
  style(s).dim().to_string()
}