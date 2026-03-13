use std::{fs, path::Path};

use crate::laravel_indexer::{
    extract_fillable, extract_routes, get_blade_components, get_controllers,
};

// ─── Index ────────────────────────────────────────────────────────────────────

/// A snapshot of everything the extension has discovered about the Laravel project.
#[derive(Debug, Default)]
pub struct LaravelIndex {
    /// Blade component tags, e.g. `<x-alert />`, `<x-forms-input />`
    pub blade_components: Vec<String>,
    /// Named route identifiers, e.g. `"home.index"`, `"auth.login"`
    pub routes: Vec<String>,
    /// Model `$fillable` property names, e.g. `"email"`, `"title"`
    pub model_properties: Vec<String>,
    /// Short controller class names, e.g. `"UserController"`
    pub controllers: Vec<String>,
}

// ─── Scanner ──────────────────────────────────────────────────────────────────

/// Walks the Laravel project rooted at `root` and returns a populated [`LaravelIndex`].
///
/// All filesystem access uses `std::fs`, which works via WASI inside Zed's
/// WebAssembly extension sandbox.
pub fn scan_laravel_project(root: &Path) -> LaravelIndex {
    LaravelIndex {
        blade_components: scan_blade_components(root),
        routes: scan_routes(root),
        model_properties: scan_model_properties(root),
        controllers: get_controllers(root),
    }
}

// ─── Blade components ─────────────────────────────────────────────────────────

fn scan_blade_components(root: &Path) -> Vec<String> {
    get_blade_components(root)
}

// ─── Routes ───────────────────────────────────────────────────────────────────

fn scan_routes(root: &Path) -> Vec<String> {
    let routes_dir = root.join("routes");
    let mut routes = Vec::new();

    if !routes_dir.exists() {
        return routes;
    }

    // Scan every PHP file directly inside routes/ (web.php, api.php, …)
    if let Ok(entries) = fs::read_dir(&routes_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if is_php_file(&path) {
                if let Ok(content) = fs::read_to_string(&path) {
                    routes.extend(extract_routes(&content));
                }
            }
        }
    }

    routes.sort();
    routes.dedup();
    routes
}

// ─── Model properties ─────────────────────────────────────────────────────────

fn scan_model_properties(root: &Path) -> Vec<String> {
    let mut props = Vec::new();

    // Laravel 8+: app/Models/
    let modern_dir = root.join("app/Models");
    if modern_dir.exists() {
        scan_models_in_dir(&modern_dir, &mut props);
    }

    // Laravel 7 and below: app/ (models placed directly in app/)
    let legacy_dir = root.join("app");
    if legacy_dir.exists() {
        scan_models_in_dir(&legacy_dir, &mut props);
    }

    props.sort();
    props.dedup();
    props
}

fn scan_models_in_dir(dir: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Recurse for nested model directories
            scan_models_in_dir(&path, out);
        } else if is_php_file(&path) {
            if let Ok(content) = fs::read_to_string(&path) {
                out.extend(extract_fillable(&content));
            }
        }
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

#[inline]
fn is_php_file(path: &Path) -> bool {
    path.extension().and_then(|s| s.to_str()) == Some("php")
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn make_project() -> TempDir {
        let dir = tempfile::tempdir().unwrap();

        // Routes
        let routes = dir.path().join("routes");
        fs::create_dir_all(&routes).unwrap();
        fs::write(
            routes.join("web.php"),
            "Route::get('/home', fn() => '')->name('home');\n\
             Route::get('/about', fn() => '')->name('about');",
        )
        .unwrap();

        // Models
        let models = dir.path().join("app/Models");
        fs::create_dir_all(&models).unwrap();
        fs::write(
            models.join("User.php"),
            "<?php\nclass User extends Model {\n\
             protected $fillable = ['name', 'email'];\n}",
        )
        .unwrap();

        // Blade components
        let components = dir.path().join("resources/views/components");
        fs::create_dir_all(&components).unwrap();
        fs::write(components.join("alert.blade.php"), "").unwrap();
        fs::write(components.join("button.blade.php"), "").unwrap();

        dir
    }

    #[test]
    fn scans_routes() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        assert!(index.routes.contains(&"home".to_string()));
        assert!(index.routes.contains(&"about".to_string()));
    }

    #[test]
    fn scans_model_properties() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        assert!(index.model_properties.contains(&"name".to_string()));
        assert!(index.model_properties.contains(&"email".to_string()));
    }

    #[test]
    fn scans_blade_components() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        assert!(index
            .blade_components
            .iter()
            .any(|c| c.contains("alert")));
        assert!(index
            .blade_components
            .iter()
            .any(|c| c.contains("button")));
    }

    #[test]
    fn empty_project_returns_empty_index() {
        let dir = tempfile::tempdir().unwrap();
        let index = scan_laravel_project(dir.path());
        assert!(index.blade_components.is_empty());
        assert!(index.routes.is_empty());
        assert!(index.model_properties.is_empty());
        assert!(index.controllers.is_empty());
    }
}
