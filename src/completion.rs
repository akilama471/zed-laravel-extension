use std::fs;
use std::path::Path;
use zed_extension_api::{
    lsp::{Completion, CompletionKind},
    CodeLabel, CodeLabelSpan,
};

/// ─── Built-in Laravel Helper Functions ─────────────────────────────────────
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

/// Returns true if a name is a known Laravel helper
pub fn is_laravel_helper(name: &str) -> bool {
    LARAVEL_HELPERS.binary_search(&name).is_ok()
}

/// ─── Built-in Laravel Facades ─────────────────────────────────────────────
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

pub fn is_laravel_facade(name: &str) -> bool {
    LARAVEL_FACADES.binary_search(&name).is_ok()
}

/// ─── Blade Directives ─────────────────────────────────────────────────────
pub static BLADE_DIRECTIVES: &[&str] = &[
    "@csrf", "@method", "@yield", "@section", "@show", "@extends", "@include", "@foreach", "@for",
    "@while", "@if", "@elseif", "@else", "@endif", "@isset", "@empty", "@auth", "@guest",
    "@switch", "@case", "@break", "@default",
];

pub fn is_blade_directive(name: &str) -> bool {
    BLADE_DIRECTIVES.binary_search(&name).is_ok()
}

/// ─── Project Helpers Scanner ─────────────────────────────────────────────
pub fn scan_project_helpers(worktree_root: &Path) -> Vec<String> {
    let mut project_helpers = Vec::new();

    let helpers_paths = [
        worktree_root.join("app/helpers.php"),
        worktree_root.join("app/helpers"),
    ];

    for path in helpers_paths {
        if path.exists() {
            if path.is_file() {
                if let Ok(content) = fs::read_to_string(&path) {
                    for line in content.lines() {
                        if let Some(name) = line.trim().strip_prefix("function ") {
                            if let Some(name) = name.split('(').next() {
                                project_helpers.push(name.to_string());
                            }
                        }
                    }
                }
            } else if path.is_dir() {
                if let Ok(entries) = fs::read_dir(&path) {
                    for entry in entries.flatten() {
                        if let Ok(content) = fs::read_to_string(entry.path()) {
                            for line in content.lines() {
                                if let Some(name) = line.trim().strip_prefix("function ") {
                                    if let Some(name) = name.split('(').next() {
                                        project_helpers.push(name.to_string());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    project_helpers
}

/// ─── Label Builder ─────────────────────────────────────────────────────────
pub fn build_label(completion: &Completion) -> Option<CodeLabel> {
    let label = &completion.label;
    let kind = completion.kind?;

    match kind {
        CompletionKind::Function | CompletionKind::Method => {
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
        CompletionKind::Class | CompletionKind::Interface | CompletionKind::Struct => {
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
        CompletionKind::Constant => {
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
        CompletionKind::Variable | CompletionKind::Field | CompletionKind::Property => {
            let code = format!("${}", label);
            let len = code.len() as u32;
            Some(CodeLabel {
                code,
                spans: vec![CodeLabelSpan::code_range(0..len)],
                filter_range: (1u32..len).into(),
            })
        }
        CompletionKind::Keyword => {
            let len = label.len() as u32;
            Some(CodeLabel {
                code: label.clone(),
                spans: vec![CodeLabelSpan::literal(label, Some("keyword".to_string()))],
                filter_range: (0u32..len).into(),
            })
        }
        CompletionKind::Snippet => {
            let len = label.len() as u32;
            let style = if is_blade_directive(label) {
                Some("string.special".to_string())
            } else {
                Some("snippet".to_string())
            };
            Some(CodeLabel {
                code: label.clone(),
                spans: vec![CodeLabelSpan::literal(label, style)],
                filter_range: (0u32..len).into(),
            })
        }
        CompletionKind::Module => {
            let len = label.len() as u32;
            Some(CodeLabel {
                code: label.clone(),
                spans: vec![CodeLabelSpan::literal(label, Some("namespace".to_string()))],
                filter_range: (0u32..len).into(),
            })
        }
        _ => None,
    }
}

/// ─── Tests ────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;
    use zed_extension_api::lsp::CompletionKind;

    #[test]
    fn helpers_facades_blade() {
        assert!(is_laravel_helper("route"));
        assert!(is_laravel_helper("view"));
        assert!(is_laravel_facade("Auth"));
        assert!(is_laravel_facade("Route"));
        assert!(is_blade_directive("@csrf"));
        assert!(!is_blade_directive("@banana"));
    }

    #[test]
    fn function_label() {
        let c = Completion {
            label: "route".to_string(),
            label_details: None,
            detail: None,
            kind: Some(CompletionKind::Function),
            insert_text_format: None,
        };
        let label = build_label(&c).unwrap();
        assert!(label.code.starts_with("fn "));
    }

    #[test]
    fn class_label() {
        let c = Completion {
            label: "User".to_string(),
            label_details: None,
            detail: None,
            kind: Some(CompletionKind::Class),
            insert_text_format: None,
        };
        let label = build_label(&c).unwrap();
        assert!(label.code.starts_with("class "));
    }

    #[test]
    fn snippet_label() {
        let c = Completion {
            label: "@csrf".to_string(),
            label_details: None,
            detail: None,
            kind: Some(CompletionKind::Snippet),
            insert_text_format: None,
        };
        let label = build_label(&c).unwrap();
        assert!(label.code.starts_with("@csrf"));
    }
}
