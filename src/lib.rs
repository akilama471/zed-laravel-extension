mod completion;
#[allow(dead_code)]
mod laravel_indexer;
#[allow(dead_code)]
mod laravel_scanner;

#[path = "../commands/artisan.rs"]
mod artisan;

use std::collections::HashSet;
use std::path::PathBuf;
use zed_extension_api::{
    self as zed, lsp::Completion, CodeLabel, CodeLabelSpan, LanguageServerId, Result,
};

// ─── Constants ────────────────────────────────────────────────────────────────

const INTELEPHENSE_SERVER_PATH: &str = "node_modules/intelephense/lib/intelephense.js";
const INTELEPHENSE_PACKAGE: &str = "intelephense";
const LARAVEL_LSP_ID: &str = "laravel-lsp";

// ─── Extension struct ─────────────────────────────────────────────────────────

pub struct LaravelExtension {
    cached_server_path: Option<String>,
    cached_helpers: HashSet<String>,
    cached_facades: HashSet<String>,
    cached_blade_directives: HashSet<String>,
}

// ─── Extension trait impl ─────────────────────────────────────────────────────

impl zed::Extension for LaravelExtension {
    fn new() -> Self {
        Self {
            cached_server_path: None,
            cached_helpers: completion::LARAVEL_HELPERS
                .iter()
                .map(|s| s.to_string())
                .collect(),
            cached_facades: completion::LARAVEL_FACADES
                .iter()
                .map(|s| s.to_string())
                .collect(),
            cached_blade_directives: completion::BLADE_DIRECTIVES
                .iter()
                .map(|s| s.to_string())
                .collect(),
        }
    }

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        if let Some(bin) = worktree.which(LARAVEL_LSP_ID) {
            return Ok(zed::Command {
                command: bin,
                args: vec!["--stdio".to_string()],
                env: Default::default(),
            });
        }

        match self.ensure_intelephense(language_server_id) {
            Ok(server_path) => Ok(zed::Command {
                command: zed::node_binary_path()?,
                args: vec![server_path, "--stdio".to_string()],
                env: Default::default(),
            }),
            Err(err) => {
                eprintln!("⚠ Could not start PHP language server: {}", err);
                Ok(zed::Command {
                    command: "true".to_string(),
                    args: vec![],
                    env: Default::default(),
                })
            }
        }
    }

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "storagePath": worktree.root_path(),
            "clearCache": false,
            "globalStoragePath": worktree.root_path(),
            "licenceKey": "",
        })))
    }

    fn label_for_completion(
        &self,
        _language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        let label = &completion.label;

        // ── Artisan command ─────────────────────────────
        if let Some(cmd) = artisan::extract_command(label) {
            if artisan::is_artisan_command(cmd) {
                let code = format!("// artisan {}", label);
                let start = "// artisan ".len() as u32;
                let end = start + label.len() as u32;
                return Some(CodeLabel {
                    code,
                    spans: vec![CodeLabelSpan::code_range(start..end)],
                    filter_range: (0u32..label.len() as u32).into(),
                });
            }
        }

        // ── Laravel helper ─────────────────────────────
        if self.cached_helpers.contains(label) {
            return completion::build_label(&completion);
        }

        // ── Laravel facade ─────────────────────────────
        if self.cached_facades.contains(label) {
            return completion::build_label(&completion);
        }

        // ── Blade directive ────────────────────────────
        if self.cached_blade_directives.contains(label) {
            // Return enhanced label using the shared builder
            return completion::build_label(&completion);
        }

        // ── Fallback to generic LSP completion ────────
        completion::build_label(&completion)
    }
}

// ─── Private helpers ──────────────────────────────────────────────────────────

impl LaravelExtension {
    fn ensure_intelephense(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        if let Some(ref path) = self.cached_server_path {
            if PathBuf::from(path).exists() {
                return Ok(path.clone());
            }
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let installed = zed::npm_package_installed_version(INTELEPHENSE_PACKAGE)?;
        let latest = zed::npm_package_latest_version(INTELEPHENSE_PACKAGE)?;

        if installed.as_deref() != Some(latest.as_str()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );
            zed::npm_install_package(INTELEPHENSE_PACKAGE, &latest)?;
        }

        let server_path = std::env::current_dir()
            .ok()
            .map(|cwd| {
                cwd.join(INTELEPHENSE_SERVER_PATH)
                    .to_string_lossy()
                    .into_owned()
            })
            .unwrap_or_else(|| INTELEPHENSE_SERVER_PATH.to_string());

        if !PathBuf::from(&server_path).exists() {
            return Err(format!("intelephense not found at '{}'", server_path).into());
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        self.cached_server_path = Some(server_path.clone());
        Ok(server_path)
    }
}

// ─── Registration ─────────────────────────────────────────────────────────────

zed::register_extension!(LaravelExtension);
