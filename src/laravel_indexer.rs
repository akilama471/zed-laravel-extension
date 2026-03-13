use regex::Regex;
use std::path::Path;

/// Scans `resources/views/components/` and returns a list of `<x-component-name />` strings.
pub fn get_blade_components(project_root: &Path) -> Vec<String> {
    let mut components = Vec::new();
    let path = project_root.join("resources/views/components");

    if !path.exists() {
        return components;
    }

    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.flatten() {
            let file_name = entry.file_name();
            let name = file_name.to_string_lossy();
            if name.ends_with(".blade.php") {
                let component_name = name.replace(".blade.php", "");
                components.push(format!("<x-{} />", component_name));
            }
        }
    }

    // Also scan one level of sub-directories (e.g. components/forms/input.blade.php)
    if let Ok(entries) = std::fs::read_dir(&path) {
        for entry in entries.flatten() {
            let sub_path = entry.path();
            if sub_path.is_dir() {
                let dir_name = entry.file_name();
                let dir_str = dir_name.to_string_lossy();
                if let Ok(sub_entries) = std::fs::read_dir(&sub_path) {
                    for sub_entry in sub_entries.flatten() {
                        let file_name = sub_entry.file_name();
                        let name = file_name.to_string_lossy();
                        if name.ends_with(".blade.php") {
                            let component_name = name.replace(".blade.php", "");
                            components.push(format!("<x-{}-{} />", dir_str, component_name));
                        }
                    }
                }
            }
        }
    }

    components
}

/// Parses an Eloquent model file and returns all entries in `$fillable`.
///
/// Handles both single-quoted and double-quoted string values.
pub fn extract_fillable(content: &str) -> Vec<String> {
    // Match `$fillable = [ ... ]` across multiple lines (non-greedy)
    let re = Regex::new(r"\$fillable\s*=\s*\[([\s\S]*?)\]").unwrap();
    if let Some(cap) = re.captures(content) {
        let inner = &cap[1];
        let item_re = Regex::new(r#"['"]([^'"]+)['"]"#).unwrap();
        return item_re
            .captures_iter(inner)
            .map(|c| c[1].to_string())
            .collect();
    }
    vec![]
}

/// Parses a routes file and returns all named route strings (the value passed to `->name(...)`).
pub fn extract_routes(content: &str) -> Vec<String> {
    let re = Regex::new(r#"->name\(\s*['"]([^'"]+)['"]\s*\)"#).unwrap();
    re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

/// Scans `app/Http/Controllers/` and returns the short class name of every controller found.
pub fn get_controllers(project_root: &Path) -> Vec<String> {
    let mut controllers = Vec::new();
    let path = project_root.join("app/Http/Controllers");

    if !path.exists() {
        return controllers;
    }

    scan_controllers_dir(&path, &mut controllers);
    controllers
}

fn scan_controllers_dir(dir: &Path, out: &mut Vec<String>) {
    let class_re = Regex::new(r"class\s+(\w+Controller)").unwrap();

    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                // Recurse into sub-directories (e.g. Api/, Admin/)
                scan_controllers_dir(&path, out);
            } else if path.extension().and_then(|s| s.to_str()) == Some("php") {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    for cap in class_re.captures_iter(&content) {
                        out.push(cap[1].to_string());
                    }
                }
            }
        }
    }
}

/// Parses a PHP file and returns all `use Namespace\ClassName;` imports.
pub fn extract_use_statements(content: &str) -> Vec<String> {
    let re = Regex::new(r"^use\s+([\w\\]+);").unwrap();
    re.captures_iter(content)
        .filter_map(|cap| cap[1].split('\\').last().map(|s| s.to_string()))
        .collect()
}

/// Returns all view names referenced via `view('...')` calls in the given content.
pub fn extract_view_references(content: &str) -> Vec<String> {
    let re = Regex::new(r#"view\(\s*['"]([^'"]+)['"]\s*[,\)]"#).unwrap();
    re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_fillable_single_line() {
        let php = r#"
            protected $fillable = ['name', 'email', 'password'];
        "#;
        let fields = extract_fillable(php);
        assert_eq!(fields, vec!["name", "email", "password"]);
    }

    #[test]
    fn test_extract_fillable_multiline() {
        let php = r#"
            protected $fillable = [
                'title',
                'body',
                'published_at',
            ];
        "#;
        let fields = extract_fillable(php);
        assert_eq!(fields, vec!["title", "body", "published_at"]);
    }

    #[test]
    fn test_extract_fillable_empty() {
        assert!(extract_fillable("class Foo {}").is_empty());
    }

    #[test]
    fn test_extract_routes() {
        let php = r#"
            Route::get('/home', [HomeController::class, 'index'])->name('home.index');
            Route::post('/login', [AuthController::class, 'login'])->name('auth.login');
        "#;
        let routes = extract_routes(php);
        assert_eq!(routes, vec!["home.index", "auth.login"]);
    }

    #[test]
    fn test_extract_routes_empty() {
        assert!(extract_routes("Route::get('/', fn() => 'ok');").is_empty());
    }

    #[test]
    fn test_extract_view_references() {
        let php = r#"
            return view('dashboard.index', compact('user'));
            return view("auth.login");
        "#;
        let views = extract_view_references(php);
        assert_eq!(views, vec!["dashboard.index", "auth.login"]);
    }

    #[test]
    fn test_extract_use_statements() {
        let php = r#"
            use App\Models\User;
            use Illuminate\Http\Request;
        "#;
        let imports = extract_use_statements(php);
        assert_eq!(imports, vec!["User", "Request"]);
    }
}
