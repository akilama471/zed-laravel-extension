mod completion;
mod laravel_indexer;
mod laravel_scanner;

use completion::LaravelCompletion;
use laravel_indexer::{extract_fillable, extract_routes, get_blade_components};
use laravel_scanner::scan_laravel_project;
use zed::worktree::Worktree;

pub fn activate(worktree: &Worktree) {
    // 1️⃣ Scan project
    let index = scan_laravel_project(worktree.root_path());

    // 2️⃣ Initialize autocomplete provider
    let completion_provider = LaravelCompletion::new(index);

    // 3️⃣ Register with Zed
    zed::completion::register_provider("laravel", completion_provider);
}

impl zed::Extension for LaravelExtension {
    fn language_server_command(
        &mut self,
        _id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        Ok(zed::Command {
            command: "laravel-lsp".into(),
            args: vec![],
            env: Default::default(),
        })
    }
}
