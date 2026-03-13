use regex::Regex;
use std::{fs, path::Path};

// ─── Blade Components ─────────────────────────────────────────────────────────

/// Scans `resources/views/components/` recursively and returns a list of
/// `<x-component-name />` strings ready for completion.
pub fn get_blade_components(project_root: &Path) -> Vec<String> {
    let components_dir = project_root.join("resources/views/components");
    let mut components = Vec::new();
    collect_blade_components(&components_dir, "", &mut components);
    components
}

fn collect_blade_components(dir: &Path, prefix: &str, out: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let file_name = entry.file_name();
        let name = file_name.to_string_lossy();

        if path.is_dir() {
            // Recurse: sub-folder becomes a dotted prefix, e.g. `forms.input`
            let sub_prefix = if prefix.is_empty() {
                name.to_string()
            } else {
                format!("{}.{}", prefix, name)
            };
            collect_blade_components(&path, &sub_prefix, out);
        } else if name.ends_with(".blade.php") {
            let base = name.trim_end_matches(".blade.php");
            let component_name = if prefix.is_empty() {
                base.to_string()
            } else {
                format!("{}.{}", prefix, base)
            };
            // Convert dot-notation to kebab-case for the x-tag
            let tag = component_name.replace('.', "-");
            out.push(format!("<x-{} />", tag));
        }
    }
}

// ─── Controllers ──────────────────────────────────────────────────────────────

/// Scans `app/Http/Controllers/` recursively and returns class names.
pub fn get_controllers(project_root: &Path) -> Vec<String> {
    let controllers_dir = project_root.join("app/Http/Controllers");
    let mut controllers = Vec::new();

    if !controllers_dir.exists() {
        return controllers;
    }

    collect_controllers(&controllers_dir, &mut controllers);
    controllers
}

fn collect_controllers(dir: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_controllers(&path, out);
        } else if path.extension().and_then(|s| s.to_str()) == Some("php") {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Some(name) = extract_class_name(&content) {
                    out.push(name);
                }
            }
        }
    }
}

// ─── Model Properties ─────────────────────────────────────────────────────────

/// Extracts entries from a `$fillable` array in a PHP model file.
///
/// ```php
/// protected $fillable = ['name', 'email', 'password'];
/// ```
pub fn extract_fillable(content: &str) -> Vec<String> {
    // Match both single and double quoted strings inside $fillable = [...]
    let array_re = Regex::new(r"\$fillable\s*=\s*\[([^\]]*)\]").unwrap();
    let string_re = Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();

    if let Some(cap) = array_re.captures(content) {
        let array_body = &cap[1];
        return string_re
            .captures_iter(array_body)
            .map(|c| c[1].to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    Vec::new()
}

/// Extracts entries from a `$guarded` array in a PHP model file.
pub fn extract_guarded(content: &str) -> Vec<String> {
    let array_re = Regex::new(r"\$guarded\s*=\s*\[([^\]]*)\]").unwrap();
    let string_re = Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();

    if let Some(cap) = array_re.captures(content) {
        let array_body = &cap[1];
        return string_re
            .captures_iter(array_body)
            .map(|c| c[1].to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    Vec::new()
}

/// Extracts entries from a `$casts` array in a PHP model file.
pub fn extract_cast_keys(content: &str) -> Vec<String> {
    let array_re = Regex::new(r"\$casts\s*=\s*\[([^\]]*)\]").unwrap();
    let key_re = Regex::new(r#"['"]([^'"]+)['"]\s*=>"#).unwrap();

    if let Some(cap) = array_re.captures(content) {
        let array_body = &cap[1];
        return key_re
            .captures_iter(array_body)
            .map(|c| c[1].to_string())
            .filter(|s| !s.is_empty())
            .collect();
    }

    Vec::new()
}

// ─── Routes ───────────────────────────────────────────────────────────────────

/// Extracts named route names from a PHP routes file.
///
/// Matches patterns like:
/// ```php
/// ->name('dashboard')
/// Route::get(...)->name("profile.edit")
/// ```
pub fn extract_routes(content: &str) -> Vec<String> {
    let re = Regex::new(r"->name\(\s*['\"]([^'\"]+)['\"]\s*\)").unwrap();
    re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Extracts route URIs from a PHP routes file.
///
/// Matches patterns like:
/// ```php
/// Route::get('/dashboard', ...)
/// Route::post('/users/{id}', ...)
/// ```
pub fn extract_route_uris(content: &str) -> Vec<String> {
    let re = Regex::new(
        r"Route::\w+\(\s*['\"]([^'\"]+)['\"]",
    )
    .unwrap();
    re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

// ─── PHP Class Utilities ──────────────────────────────────────────────────────

/// Extracts the declared class name from a PHP file's content.
///
/// ```php
/// class UserController extends Controller
/// ```
/// → `Some("UserController")`
pub fn extract_class_name(content: &str) -> Option<String> {
    let re = Regex::new(r"(?m)^\s*(?:abstract\s+|final\s+)?class\s+(\w+)").unwrap();
    re.captures(content).map(|cap| cap[1].to_string())
}

/// Extracts the declared namespace from a PHP file's content.
///
/// ```php
/// namespace App\Http\Controllers;
/// ```
/// → `Some("App\\Http\\Controllers")`
pub fn extract_namespace(content: &str) -> Option<String> {
    let re = Regex::new(r"(?m)^namespace\s+([\w\\]+)\s*;").unwrap();
    re.captures(content).map(|cap| cap[1].to_string())
}

/// Extracts the fully-qualified class name (namespace + class name).
pub fn extract_fqn(content: &str) -> Option<String> {
    let namespace = extract_namespace(content)?;
    let class = extract_class_name(content)?;
    Some(format!("{}\\{}", namespace, class))
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_fillable_single_quotes() {
        let php = "protected $fillable = ['name', 'email', 'password'];";
        let result = extract_fillable(php);
        assert_eq!(result, vec!["name", "email", "password"]);
    }

    #[test]
    fn extracts_fillable_double_quotes() {
        let php = r#"protected $fillable = ["title", "body"];"#;
        let result = extract_fillable(php);
        assert_eq!(result, vec!["title", "body"]);
    }

    #[test]
    fn extracts_fillable_multiline() {
        let php = "protected $fillable = [\n    'first_name',\n    'last_name',\n];";
        let result = extract_fillable(php);
        assert_eq!(result, vec!["first_name", "last_name"]);
    }

    #[test]
    fn extracts_named_routes() {
        let php = "Route::get('/dashboard', fn() => '')->name('dashboard');\n\
                   Route::get('/profile', fn() => '')->name(\"profile.show\");";
        let result = extract_routes(php);
        assert_eq!(result, vec!["dashboard", "profile.show"]);
    }

    #[test]
    fn extracts_route_uris() {
        let php = "Route::get('/users', [UserController::class, 'index']);\n\
                   Route::post('/users/{id}', [UserController::class, 'update']);";
        let result = extract_route_uris(php);
        assert_eq!(result, vec!["/users", "/users/{id}"]);
    }

    #[test]
    fn extracts_class_name() {
        let php = "<?php\n\nnamespace App\\Http\\Controllers;\n\nclass UserController extends Controller\n{";
        assert_eq!(extract_class_name(php), Some("UserController".to_string()));
    }

    #[test]
    fn extracts_namespace() {
        let php = "<?php\n\nnamespace App\\Models;\n\nclass User extends Model {}";
        assert_eq!(extract_namespace(php), Some("App\\Models".to_string()));
    }

    #[test]
    fn extracts_fqn() {
        let php = "<?php\n\nnamespace App\\Models;\n\nclass Post extends Model {}";
        assert_eq!(extract_fqn(php), Some("App\\Models\\Post".to_string()));
    }

    #[test]
    fn empty_fillable_returns_empty() {
        let php = "protected $fillable = [];";
        assert_eq!(extract_fillable(php), Vec::<String>::new());
    }

    #[test]
    fn no_fillable_returns_empty() {
        let php = "class User extends Model {}";
        assert_eq!(extract_fillable(php), Vec::<String>::new());
    }
}
