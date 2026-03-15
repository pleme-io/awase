use serde::{Deserialize, Serialize};

/// Conditions for when a binding is active.
///
/// All fields are optional — `None` means "match any". Multiple fields are
/// AND-combined: all specified conditions must match for the binding to be
/// active.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Condition {
    /// Active only when focused app bundle_id matches (substring or regex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app: Option<String>,
    /// Active only when focused app does NOT match (substring or regex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub app_exclude: Option<String>,
    /// Active only when window title matches (substring or regex).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// Active only on specified display index.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub display: Option<u32>,
}

/// Context for evaluating conditions against the current system state.
#[derive(Debug, Clone, Default)]
pub struct MatchContext {
    /// Bundle ID of the currently focused application.
    pub focused_app_bundle_id: Option<String>,
    /// Title of the currently focused window.
    pub focused_window_title: Option<String>,
    /// Index of the display containing the focused window.
    pub display_index: u32,
}

impl Condition {
    /// Returns `true` if all specified conditions match the given context.
    #[must_use]
    pub fn matches(&self, ctx: &MatchContext) -> bool {
        if let Some(ref app_pattern) = self.app {
            let Some(ref bundle_id) = ctx.focused_app_bundle_id else {
                return false;
            };
            if !pattern_matches(app_pattern, bundle_id) {
                return false;
            }
        }

        if let Some(ref app_exclude_pattern) = self.app_exclude {
            if let Some(ref bundle_id) = ctx.focused_app_bundle_id {
                if pattern_matches(app_exclude_pattern, bundle_id) {
                    return false;
                }
            }
        }

        if let Some(ref title_pattern) = self.title {
            let Some(ref title) = ctx.focused_window_title else {
                return false;
            };
            if !pattern_matches(title_pattern, title) {
                return false;
            }
        }

        if let Some(display) = self.display {
            if ctx.display_index != display {
                return false;
            }
        }

        true
    }
}

/// Simple pattern matching: pipe-separated alternatives, each checked
/// as a case-insensitive substring match.
///
/// Example: `"com.apple.Terminal|com.mitchellh.ghostty"` matches if the
/// value contains either substring.
fn pattern_matches(pattern: &str, value: &str) -> bool {
    let value_lower = value.to_ascii_lowercase();
    pattern
        .split('|')
        .any(|p| value_lower.contains(&p.trim().to_ascii_lowercase()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx(app: Option<&str>, title: Option<&str>, display: u32) -> MatchContext {
        MatchContext {
            focused_app_bundle_id: app.map(String::from),
            focused_window_title: title.map(String::from),
            display_index: display,
        }
    }

    #[test]
    fn empty_condition_matches_everything() {
        let c = Condition::default();
        assert!(c.matches(&ctx(Some("com.apple.Safari"), Some("Google"), 0)));
        assert!(c.matches(&ctx(None, None, 5)));
    }

    #[test]
    fn app_include_match() {
        let c = Condition {
            app: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        assert!(c.matches(&ctx(Some("com.apple.Safari"), None, 0)));
        assert!(!c.matches(&ctx(Some("com.mitchellh.ghostty"), None, 0)));
    }

    #[test]
    fn app_include_no_app_fails() {
        let c = Condition {
            app: Some("com.apple.Safari".to_string()),
            ..Default::default()
        };
        assert!(!c.matches(&ctx(None, None, 0)));
    }

    #[test]
    fn app_exclude_match() {
        let c = Condition {
            app_exclude: Some("com.apple.Terminal|com.mitchellh.ghostty".to_string()),
            ..Default::default()
        };
        assert!(!c.matches(&ctx(Some("com.apple.Terminal"), None, 0)));
        assert!(!c.matches(&ctx(Some("com.mitchellh.ghostty"), None, 0)));
        assert!(c.matches(&ctx(Some("com.apple.Safari"), None, 0)));
    }

    #[test]
    fn app_exclude_no_app_passes() {
        let c = Condition {
            app_exclude: Some("com.apple.Terminal".to_string()),
            ..Default::default()
        };
        assert!(c.matches(&ctx(None, None, 0)));
    }

    #[test]
    fn title_match() {
        let c = Condition {
            title: Some("Untitled".to_string()),
            ..Default::default()
        };
        assert!(c.matches(&ctx(None, Some("Untitled Document"), 0)));
        assert!(!c.matches(&ctx(None, Some("My File"), 0)));
        assert!(!c.matches(&ctx(None, None, 0)));
    }

    #[test]
    fn display_match() {
        let c = Condition {
            display: Some(1),
            ..Default::default()
        };
        assert!(c.matches(&ctx(None, None, 1)));
        assert!(!c.matches(&ctx(None, None, 0)));
    }

    #[test]
    fn combined_conditions() {
        let c = Condition {
            app: Some("Safari".to_string()),
            title: Some("Google".to_string()),
            display: Some(0),
            ..Default::default()
        };
        assert!(c.matches(&ctx(Some("com.apple.Safari"), Some("Google Search"), 0)));
        assert!(!c.matches(&ctx(Some("com.apple.Safari"), Some("Google Search"), 1)));
        assert!(!c.matches(&ctx(Some("com.apple.Safari"), Some("Yahoo"), 0)));
        assert!(!c.matches(&ctx(Some("com.mitchellh.ghostty"), Some("Google Search"), 0)));
    }

    #[test]
    fn pattern_pipe_alternatives() {
        let c = Condition {
            app: Some("Safari|Chrome|Firefox".to_string()),
            ..Default::default()
        };
        assert!(c.matches(&ctx(Some("com.apple.Safari"), None, 0)));
        assert!(c.matches(&ctx(Some("com.google.Chrome"), None, 0)));
        assert!(c.matches(&ctx(Some("org.mozilla.Firefox"), None, 0)));
        assert!(!c.matches(&ctx(Some("com.mitchellh.ghostty"), None, 0)));
    }

    #[test]
    fn case_insensitive_matching() {
        let c = Condition {
            app: Some("safari".to_string()),
            ..Default::default()
        };
        assert!(c.matches(&ctx(Some("com.apple.Safari"), None, 0)));
    }

    #[test]
    fn serde_roundtrip() {
        let c = Condition {
            app: Some("Safari".to_string()),
            app_exclude: None,
            title: Some("test".to_string()),
            display: Some(1),
        };
        let json = serde_json::to_string(&c).unwrap();
        let deserialized: Condition = serde_json::from_str(&json).unwrap();
        assert_eq!(c, deserialized);
    }

    #[test]
    fn serde_skips_none_fields() {
        let c = Condition {
            app: Some("test".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&c).unwrap();
        assert!(!json.contains("app_exclude"));
        assert!(!json.contains("title"));
        assert!(!json.contains("display"));
    }

    // ── Additional condition tests ──────────────────────────────────

    #[test]
    fn app_and_app_exclude_combined() {
        // Both app include and exclude present -- both must pass
        let c = Condition {
            app: Some("Safari".to_string()),
            app_exclude: Some("Private".to_string()),
            ..Default::default()
        };
        // App matches include, no exclude match
        assert!(c.matches(&ctx(Some("com.apple.Safari"), None, 0)));
        // App matches include but also matches exclude
        assert!(!c.matches(&ctx(Some("com.apple.Safari.Private"), None, 0)));
        // App doesn't match include
        assert!(!c.matches(&ctx(Some("com.apple.Terminal"), None, 0)));
    }

    #[test]
    fn title_case_insensitive() {
        let c = Condition {
            title: Some("google".to_string()),
            ..Default::default()
        };
        assert!(c.matches(&ctx(None, Some("GOOGLE Search"), 0)));
        assert!(c.matches(&ctx(None, Some("Google"), 0)));
    }

    #[test]
    fn app_exclude_with_spaces_in_pattern() {
        let c = Condition {
            app_exclude: Some(" Terminal | Ghostty ".to_string()),
            ..Default::default()
        };
        assert!(!c.matches(&ctx(Some("com.apple.Terminal"), None, 0)));
        assert!(!c.matches(&ctx(Some("com.mitchellh.Ghostty"), None, 0)));
        assert!(c.matches(&ctx(Some("com.apple.Safari"), None, 0)));
    }

    #[test]
    fn match_context_default_values() {
        let ctx = MatchContext::default();
        assert!(ctx.focused_app_bundle_id.is_none());
        assert!(ctx.focused_window_title.is_none());
        assert_eq!(ctx.display_index, 0);
    }

    #[test]
    fn empty_app_pattern_matches_everything() {
        // An empty string is a substring of every string
        let c = Condition {
            app: Some(String::new()),
            ..Default::default()
        };
        assert!(c.matches(&ctx(Some("anything"), None, 0)));
    }

    #[test]
    fn display_zero_matches() {
        let c = Condition {
            display: Some(0),
            ..Default::default()
        };
        assert!(c.matches(&ctx(None, None, 0)));
        assert!(!c.matches(&ctx(None, None, 1)));
    }

    #[test]
    fn all_conditions_present() {
        let c = Condition {
            app: Some("Safari".to_string()),
            app_exclude: Some("Private".to_string()),
            title: Some("Search".to_string()),
            display: Some(1),
        };
        // All conditions met
        assert!(c.matches(&ctx(Some("com.apple.Safari"), Some("Google Search"), 1)));
        // Wrong display
        assert!(!c.matches(&ctx(Some("com.apple.Safari"), Some("Google Search"), 0)));
        // Wrong title
        assert!(!c.matches(&ctx(Some("com.apple.Safari"), Some("Homepage"), 1)));
        // App excluded
        assert!(!c.matches(&ctx(Some("com.apple.Safari.Private"), Some("Google Search"), 1)));
    }
}
