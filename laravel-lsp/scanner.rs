use std::{fs, path::Path};

use crate::indexer::{
    extract_fillable, extract_routes, extract_route_uris, get_blade_components, get_controllers,
};

// ─── Index ────────────────────────────────────────────────────────────────────

/// A snapshot of everything discovered about the Laravel project.
/// Populated once on `initialize` and used for all completion requests.
#[derive(Debug, Default, Clone)]
pub struct LaravelIndex {
    /// Blade component tags ready for insertion, e.g. `<x-alert />`
    pub blade_components: Vec<String>,
    /// Named route identifiers, e.g. `"home.index"`, `"auth.login"`
    pub routes: Vec<String>,
    /// Route URIs, e.g. `"/dashboard"`, `"/users/{id}"`
    pub route_uris: Vec<String>,
    /// Eloquent `$fillable` property names across all models
    pub model_properties: Vec<String>,
    /// Short controller class names, e.g. `"UserController"`
    pub controllers: Vec<String>,
}

// ─── Entry point ──────────────────────────────────────────────────────────────

/// Walks the Laravel project rooted at `root` and returns a fully-populated
/// [`LaravelIndex`].  All I/O uses `std::fs` (synchronous, no async runtime
/// required from the scanner itself).
pub fn scan_laravel_project(root: &Path) -> LaravelIndex {
    LaravelIndex {
        blade_components: scan_blade_components(root),
        routes:           scan_named_routes(root),
        route_uris:       scan_route_uris(root),
        model_properties: scan_model_properties(root),
        controllers:      get_controllers(root),
    }
}

// ─── Blade components ─────────────────────────────────────────────────────────

fn scan_blade_components(root: &Path) -> Vec<String> {
    get_blade_components(root)
}

// ─── Routes ───────────────────────────────────────────────────────────────────

fn scan_named_routes(root: &Path) -> Vec<String> {
    let mut routes = Vec::new();
    scan_routes_dir(root, &mut routes, &mut Vec::new());
    routes.sort();
    routes.dedup();
    routes
}

fn scan_route_uris(root: &Path) -> Vec<String> {
    let mut uris = Vec::new();
    scan_routes_dir(root, &mut Vec::new(), &mut uris);
    uris.sort();
    uris.dedup();
    uris
}

/// Reads every `.php` file inside `routes/` and accumulates named routes and
/// route URIs into the supplied buffers.
fn scan_routes_dir(
    root:        &Path,
    named_out:   &mut Vec<String>,
    uri_out:     &mut Vec<String>,
) {
    let routes_dir = root.join("routes");
    if !routes_dir.exists() {
        return;
    }

    let entries = match fs::read_dir(&routes_dir) {
        Ok(e)  => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if is_php_file(&path) {
            if let Ok(content) = fs::read_to_string(&path) {
                named_out.extend(extract_routes(&content));
                uri_out.extend(extract_route_uris(&content));
            }
        }
    }
}

// ─── Model properties ─────────────────────────────────────────────────────────

fn scan_model_properties(root: &Path) -> Vec<String> {
    let mut props = Vec::new();

    // Laravel 8+: app/Models/
    let modern_dir = root.join("app/Models");
    if modern_dir.exists() {
        collect_fillable_from_dir(&modern_dir, &mut props);
    }

    // Laravel 7 and below: models live directly in app/
    let legacy_dir = root.join("app");
    if legacy_dir.exists() {
        collect_fillable_from_dir_flat(&legacy_dir, &mut props);
    }

    props.sort();
    props.dedup();
    props
}

/// Recursively scans `dir` for PHP model files and extracts `$fillable` entries.
fn collect_fillable_from_dir(dir: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e)  => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_fillable_from_dir(&path, out);
        } else if is_php_file(&path) {
            if let Ok(content) = fs::read_to_string(&path) {
                out.extend(extract_fillable(&content));
            }
        }
    }
}

/// Scans only the top-level PHP files in `dir` (no recursion) to avoid
/// accidentally treating sub-directories (Controllers, Services, …) as models
/// when in legacy layout.
fn collect_fillable_from_dir_flat(dir: &Path, out: &mut Vec<String>) {
    let entries = match fs::read_dir(dir) {
        Ok(e)  => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_file() && is_php_file(&path) {
            if let Ok(content) = fs::read_to_string(&path) {
                // Only collect if the file actually contains a $fillable array
                let fields = extract_fillable(&content);
                if !fields.is_empty() {
                    out.extend(fields);
                }
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

    /// Creates a minimal Laravel project skeleton under a temporary directory.
    fn make_project() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");

        // routes/web.php
        let routes = dir.path().join("routes");
        fs::create_dir_all(&routes).unwrap();
        fs::write(
            routes.join("web.php"),
            "<?php\n\
             Route::get('/home', [HomeController::class, 'index'])->name('home.index');\n\
             Route::post('/login', [AuthController::class, 'login'])->name('auth.login');\n",
        )
        .unwrap();

        // routes/api.php
        fs::write(
            routes.join("api.php"),
            "<?php\n\
             Route::get('/users', [UserController::class, 'index'])->name('api.users.index');\n",
        )
        .unwrap();

        // app/Models/User.php
        let models = dir.path().join("app/Models");
        fs::create_dir_all(&models).unwrap();
        fs::write(
            models.join("User.php"),
            "<?php\nnamespace App\\Models;\n\
             class User extends Model {\n\
                 protected $fillable = ['name', 'email', 'password'];\n\
             }",
        )
        .unwrap();

        // app/Models/Post.php
        fs::write(
            models.join("Post.php"),
            "<?php\nnamespace App\\Models;\n\
             class Post extends Model {\n\
                 protected $fillable = ['title', 'body', 'published_at'];\n\
             }",
        )
        .unwrap();

        // app/Http/Controllers/UserController.php
        let controllers = dir.path().join("app/Http/Controllers");
        fs::create_dir_all(&controllers).unwrap();
        fs::write(
            controllers.join("UserController.php"),
            "<?php\nnamespace App\\Http\\Controllers;\n\
             class UserController extends Controller {}",
        )
        .unwrap();

        // resources/views/components/alert.blade.php
        let components = dir.path().join("resources/views/components");
        fs::create_dir_all(&components).unwrap();
        fs::write(components.join("alert.blade.php"), "").unwrap();
        fs::write(components.join("button.blade.php"), "").unwrap();

        // Sub-directory component: components/forms/input.blade.php
        let form_components = components.join("forms");
        fs::create_dir_all(&form_components).unwrap();
        fs::write(form_components.join("input.blade.php"), "").unwrap();

        dir
    }

    #[test]
    fn scans_named_routes_from_multiple_files() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        assert!(index.routes.contains(&"home.index".to_string()));
        assert!(index.routes.contains(&"auth.login".to_string()));
        assert!(index.routes.contains(&"api.users.index".to_string()));
    }

    #[test]
    fn scans_route_uris() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        assert!(index.route_uris.contains(&"/home".to_string()));
        assert!(index.route_uris.contains(&"/login".to_string()));
        assert!(index.route_uris.contains(&"/users".to_string()));
    }

    #[test]
    fn scans_model_fillable_properties() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        // User model
        assert!(index.model_properties.contains(&"name".to_string()));
        assert!(index.model_properties.contains(&"email".to_string()));
        assert!(index.model_properties.contains(&"password".to_string()));
        // Post model
        assert!(index.model_properties.contains(&"title".to_string()));
        assert!(index.model_properties.contains(&"body".to_string()));
        assert!(index.model_properties.contains(&"published_at".to_string()));
    }

    #[test]
    fn scans_blade_components() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        assert!(index.blade_components.iter().any(|c| c.contains("alert")));
        assert!(index.blade_components.iter().any(|c| c.contains("button")));
    }

    #[test]
    fn scans_controllers() {
        let dir = make_project();
        let index = scan_laravel_project(dir.path());
        assert!(index.controllers.contains(&"UserController".to_string()));
    }

    #[test]
    fn empty_project_returns_empty_index() {
        let dir = tempfile::tempdir().unwrap();
        let index = scan_laravel_project(dir.path());
        assert!(index.blade_components.is_empty());
        assert!(index.routes.is_empty());
        assert!(index.route_uris.is_empty());
        assert!(index.model_properties.is_empty());
        assert!(index.controllers.is_empty());
    }

    #[test]
    fn duplicate_routes_are_deduplicated() {
        let dir = tempfile::tempdir().unwrap();
        let routes = dir.path().join("routes");
        fs::create_dir_all(&routes).unwrap();
        // Both files declare the same named route
        fs::write(
            routes.join("web.php"),
            "Route::get('/a', fn() => '')->name('duplicate');",
        )
        .unwrap();
        fs::write(
            routes.join("api.php"),
            "Route::get('/b', fn() => '')->name('duplicate');",
        )
        .unwrap();

        let index = scan_laravel_project(dir.path());
        let count = index.routes.iter().filter(|r| *r == "duplicate").count();
        assert_eq!(count, 1, "duplicate route names should be deduplicated");
    }
}
