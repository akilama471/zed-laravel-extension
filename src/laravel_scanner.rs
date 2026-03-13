use std::fs;
use std::path::{Path, PathBuf};

use crate::laravel_indexer::{extract_fillable, extract_routes, get_blade_components};

pub struct LaravelIndex {
    pub blade_components: Vec<String>,
    pub routes: Vec<String>,
    pub model_properties: Vec<String>,
}

pub fn scan_laravel_project(root: &Path) -> LaravelIndex {
    let mut routes = Vec::new();
    let mut model_props = Vec::new();

    // ------------------------
    // Scan Blade Components
    // ------------------------

    let blade_components = get_blade_components(root);

    // ------------------------
    // Scan Routes
    // ------------------------

    let routes_dir = root.join("routes");

    if routes_dir.exists() {
        for entry in fs::read_dir(routes_dir).unwrap() {
            let path = entry.unwrap().path();

            if path.extension().and_then(|s| s.to_str()) == Some("php") {
                if let Ok(content) = fs::read_to_string(&path) {
                    routes.extend(extract_routes(&content));
                }
            }
        }
    }

    // ------------------------
    // Scan Models
    // ------------------------

    let models_dir = root.join("app/Models");

    if models_dir.exists() {
        for entry in fs::read_dir(models_dir).unwrap() {
            let path = entry.unwrap().path();

            if path.extension().and_then(|s| s.to_str()) == Some("php") {
                if let Ok(content) = fs::read_to_string(&path) {
                    model_props.extend(extract_fillable(&content));
                }
            }
        }
    }

    LaravelIndex {
        blade_components,
        routes,
        model_properties: model_props,
    }
}
