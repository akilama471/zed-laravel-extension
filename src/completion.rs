use zed_extension_api::{
    lsp::{Completion, CompletionKind},
    CodeLabel, CodeLabelSpan,
};

// ─── Laravel Helper Functions ─────────────────────────────────────────────────

/// Every first-party Laravel global helper function.
pub static LARAVEL_HELPERS: &[&str] = &[
    "abort",
    "abort_if",
    "abort_unless",
    "action",
    "app",
    "app_path",
    "asset",
    "auth",
    "back",
    "base_path",
    "bcrypt",
    "blank",
    "broadcast",
    "cache",
    "config",
    "config_path",
    "cookie",
    "csrf_field",
    "csrf_token",
    "database_path",
    "dd",
    "dispatch",
    "dispatch_sync",
    "dump",
    "encrypt",
    "env",
    "event",
    "fake",
    "filled",
    "info",
    "lang_path",
    "logger",
    "method_field",
    "mix",
    "now",
    "old",
    "optional",
    "policy",
    "public_path",
    "redirect",
    "report",
    "report_if",
    "report_unless",
    "request",
    "rescue",
    "resolve",
    "resource",
    "resource_path",
    "response",
    "retry",
    "route",
    "secure_asset",
    "secure_url",
    "session",
    "storage_path",
    "str",
    "tap",
    "throw_if",
    "throw_unless",
    "today",
    "trans",
    "trans_choice",
    "to_route",
    "url",
    "validator",
    "value",
    "view",
    "with",
];

/// Returns `true` when `name` matches a known Laravel global helper.
///
/// Uses a binary search on the sorted `LARAVEL_HELPERS` slice, so it is O(log n).
pub fn is_laravel_helper(name: &str) -> bool {
    LARAVEL_HELPERS.binary_search(&name).is_ok()
}

// ─── Facade list ──────────────────────────────────────────────────────────────

/// Short names of the most common Laravel facades.
pub static LARAVEL_FACADES: &[&str] = &[
    "App",
    "Artisan",
    "Auth",
    "Blade",
    "Broadcast",
    "Bus",
    "Cache",
    "Config",
    "Cookie",
    "Crypt",
    "DB",
    "Event",
    "File",
    "Gate",
    "Hash",
    "Http",
    "Lang",
    "Log",
    "Mail",
    "Notification",
    "Password",
    "Queue",
    "Redirect",
    "Request",
    "Response",
    "Route",
    "Schema",
    "Session",
    "Storage",
    "URL",
    "Validator",
    "View",
];

/// Returns `true` when `name` matches a known Laravel facade class name.
pub fn is_laravel_facade(name: &str) -> bool {
    LARAVEL_FACADES.binary_search(&name).is_ok()
}

// ─── Label builder ────────────────────────────────────────────────────────────

/// Attempts to build a richer [`CodeLabel`] for the given LSP [`Completion`].
///
/// Returns `None` when no special treatment is needed (Zed will fall back to
/// its default rendering).
///
/// ## Strategy
///
/// | `CompletionKind`                        | Synthetic code snippet         |
/// |-----------------------------------------|--------------------------------|
/// | `Function` / `Method`                   | `fn label()`                   |
/// | `Class` / `Interface` / `Struct`        | `class Label`                  |
/// | `Constant`                               | `const LABEL`                  |
/// | `Variable` / `Field` / `Property`       | `$label`                       |
/// | `Keyword`                                | literal span, no code          |
/// | *(everything else)*                      | plain label literal            |
///
/// For all code-backed variants the `filter_range` covers only the label text
/// so that fuzzy matching is based on the identifier alone, not the keyword
/// prefix (e.g. `"fn "` or `"class "`).
pub fn build_label(completion: &Completion) -> Option<CodeLabel> {
    let label = &completion.label;
    let kind = completion.kind?;

    match kind {
        // ── Functions / Methods ──────────────────────────────────────────────
        CompletionKind::Function | CompletionKind::Method => {
            // Produce:  fn label_name()
            //           ^^^           prefix we skip in the span
            let prefix = "fn ";
            let code = format!("{}{}()", prefix, label);
            let start = prefix.len() as u32;
            let end = start + label.len() as u32;

            Some(CodeLabel {
                code,
                spans: vec![CodeLabelSpan::code_range(start..end)],
                filter_range: (0u32..label.len() as u32).into(),
            })
        }

        // ── Classes / Interfaces / Traits ────────────────────────────────────
        CompletionKind::Class | CompletionKind::Interface | CompletionKind::Struct => {
            // Produce:  class LabelName
            //           ^^^^^^ prefix we skip
            let prefix = "class ";
            let code = format!("{}{}", prefix, label);
            let start = prefix.len() as u32;
            let end = start + label.len() as u32;

            Some(CodeLabel {
                code,
                spans: vec![CodeLabelSpan::code_range(start..end)],
                filter_range: (0u32..label.len() as u32).into(),
            })
        }

        // ── Constants ────────────────────────────────────────────────────────
        CompletionKind::Constant => {
            // Produce:  const LABEL_NAME
            let prefix = "const ";
            let code = format!("{}{}", prefix, label);
            let start = prefix.len() as u32;
            let end = start + label.len() as u32;

            Some(CodeLabel {
                code,
                spans: vec![CodeLabelSpan::code_range(start..end)],
                filter_range: (0u32..label.len() as u32).into(),
            })
        }

        // ── Variables / Fields / Properties ──────────────────────────────────
        CompletionKind::Variable | CompletionKind::Field | CompletionKind::Property => {
            // Produce:  $label_name
            // The `$` sigil is part of the rendered span but excluded from
            // filter_range so the user can type without the sigil.
            let code = format!("${}", label);
            let len = code.len() as u32;

            Some(CodeLabel {
                code,
                spans: vec![CodeLabelSpan::code_range(0..len)],
                // Filter on the bare name, without the leading '$'
                filter_range: (1u32..len).into(),
            })
        }

        // ── Keywords ─────────────────────────────────────────────────────────
        CompletionKind::Keyword => {
            let len = label.len() as u32;
            Some(CodeLabel {
                // No synthetic Rust code — use a plain literal span so the
                // PHP keyword is highlighted as a keyword token.
                code: label.clone(),
                spans: vec![CodeLabelSpan::literal(
                    label.as_str(),
                    Some("keyword".to_string()),
                )],
                filter_range: (0u32..len).into(),
            })
        }

        // ── Snippets (e.g. Blade directives) ─────────────────────────────────
        CompletionKind::Snippet => {
            let len = label.len() as u32;
            Some(CodeLabel {
                code: label.clone(),
                spans: vec![CodeLabelSpan::literal(
                    label.as_str(),
                    Some("string.special".to_string()),
                )],
                filter_range: (0u32..len).into(),
            })
        }

        // ── Modules / Namespaces ──────────────────────────────────────────────
        CompletionKind::Module => {
            let len = label.len() as u32;
            Some(CodeLabel {
                code: label.clone(),
                spans: vec![CodeLabelSpan::literal(
                    label.as_str(),
                    Some("namespace".to_string()),
                )],
                filter_range: (0u32..len).into(),
            })
        }

        // ── Everything else: let Zed render its default ───────────────────────
        _ => None,
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_helpers_are_recognised() {
        assert!(is_laravel_helper("route"));
        assert!(is_laravel_helper("view"));
        assert!(is_laravel_helper("config"));
        assert!(is_laravel_helper("abort_if"));
        assert!(is_laravel_helper("now"));
    }

    #[test]
    fn unknown_names_are_rejected() {
        assert!(!is_laravel_helper("banana"));
        assert!(!is_laravel_helper("Route")); // facade, not helper
        assert!(!is_laravel_helper(""));
    }

    #[test]
    fn known_facades_are_recognised() {
        assert!(is_laravel_facade("Auth"));
        assert!(is_laravel_facade("DB"));
        assert!(is_laravel_facade("Route"));
        assert!(is_laravel_facade("Storage"));
    }

    #[test]
    fn unknown_facade_is_rejected() {
        assert!(!is_laravel_facade("auth")); // helper, not facade
        assert!(!is_laravel_facade("Banana"));
    }

    fn make_completion(label: &str, kind: CompletionKind) -> Completion {
        Completion {
            label: label.to_string(),
            label_details: None,
            detail: None,
            kind: Some(kind),
            insert_text_format: None,
        }
    }

    #[test]
    fn function_label_uses_fn_prefix() {
        let c = make_completion("route", CompletionKind::Function);
        let label = build_label(&c).expect("should produce a label");
        assert!(label.code.starts_with("fn "));
        assert!(label.code.contains("route"));
        // filter_range starts at 0 (the bare name, no prefix)
        assert_eq!(label.filter_range, 0..5);
    }

    #[test]
    fn class_label_uses_class_prefix() {
        let c = make_completion("User", CompletionKind::Class);
        let label = build_label(&c).expect("should produce a label");
        assert!(label.code.starts_with("class "));
        assert!(label.code.contains("User"));
        assert_eq!(label.filter_range, 0..4);
    }

    #[test]
    fn constant_label_uses_const_prefix() {
        let c = make_completion("APP_ENV", CompletionKind::Constant);
        let label = build_label(&c).expect("should produce a label");
        assert!(label.code.starts_with("const "));
        assert!(label.code.contains("APP_ENV"));
    }

    #[test]
    fn variable_label_has_dollar_sigil() {
        let c = make_completion("request", CompletionKind::Variable);
        let label = build_label(&c).expect("should produce a label");
        assert!(label.code.starts_with('$'));
        // filter_range skips the '$'
        assert_eq!(label.filter_range.start, 1);
    }

    #[test]
    fn no_kind_returns_none() {
        let c = Completion {
            label: "something".to_string(),
            label_details: None,
            detail: None,
            kind: None,
            insert_text_format: None,
        };
        assert!(build_label(&c).is_none());
    }
}
