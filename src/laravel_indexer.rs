use std::path::Path;
use regex::Regex;

pub fn get_blade_components(project_root: &Path) -> Vec<String> {
    let mut components = Vec::new();

    let path = project_root.join("resources/views/components");

    if path.exists() {
        for entry in std::fs::read_dir(path).unwrap() {
            let file = entry.unwrap().file_name();
            let name = file.to_string_lossy().replace(".blade.php", "");
            components.push(format!("<x-{} />", name));
        }
    }

    components
}

pub fn extract_fillable(content: &str) -> Vec<String> {
    let re = Regex::new(r"\$fillable\s*=\s*\[(.*?)\]").unwrap();
    if let Some(cap) = re.captures(content) {
        cap[1]
            .split(',')
            .map(|s| s.trim().replace("'", ""))
            .collect()
    } else {
        vec![]
    }
}

pub fn extract_routes(content: &str) -> Vec<String> {
    let re = Regex::new(r"name\(['\"](.*?)['\"]\)").unwrap();

    re.captures_iter(content)
        .map(|cap| cap[1].to_string())
        .collect()
}
