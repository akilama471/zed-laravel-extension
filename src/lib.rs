mod completion;
#[allow(dead_code)]
mod laravel_indexer;
#[allow(dead_code)]
mod laravel_scanner;

#[path = "../commands/artisan.rs"]
mod artisan;

use zed_extension_api::{
    self as zed, lsp::Completion, CodeLabel, CodeLabelSpan, LanguageServerId, Result,
};

// ─── Constants ────────────────────────────────────────────────────────────────

/// Path to the intelephense entry-point script, relative to the extension's
/// working directory (i.e. where `npm_install_package` places packages).
const INTELEPHENSE_SERVER_PATH: &str = "node_modules/intelephense/lib/intelephense.js";

/// The npm package name for the PHP language server.
const INTELEPHENSE_PACKAGE: &str = "intelephense";

/// The ID declared in `extension.toml` under `[language_servers]`.
const LARAVEL_LSP_ID: &str = "laravel-lsp";

// ─── Extension struct ─────────────────────────────────────────────────────────

struct LaravelExtension {
    /// Cached path to the intelephense script so we don't stat the filesystem
    /// on every request once we know the package is installed.
    cached_server_path: Option<String>,
}

// ─── Extension trait impl ─────────────────────────────────────────────────────

impl zed::Extension for LaravelExtension {
    fn new() -> Self {
        LaravelExtension {
            cached_server_path: None,
        }
    }

    // ── Language server binary ─────────────────────────────────────────────

    fn language_server_command(
        &mut self,
        language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<zed::Command> {
        // Prefer a locally-compiled `laravel-lsp` binary on the user's PATH.
        // This allows developers who have built the companion binary to get
        // Laravel-specific completions (routes, models, Blade components) on
        // top of standard PHP intelligence.
        if let Some(bin) = worktree.which(LARAVEL_LSP_ID) {
            return Ok(zed::Command {
                command: bin,
                args: vec!["--stdio".to_string()],
                env: Default::default(),
            });
        }

        // Fall back to intelephense – the best general-purpose PHP LSP that
        // also understands Laravel patterns through its stub library.
        let server_path = self.ensure_intelephense(language_server_id)?;

        Ok(zed::Command {
            command: zed::node_binary_path()?,
            args: vec![server_path, "--stdio".to_string()],
            env: Default::default(),
        })
    }

    // ── Initialisation options (sent once on server start) ────────────────

    fn language_server_initialization_options(
        &mut self,
        _language_server_id: &LanguageServerId,
        worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        // `storagePath` tells intelephense where to cache its index.
        // Using the project root keeps the cache local to the workspace.
        Ok(Some(serde_json::json!({
            "storagePath": worktree.root_path(),
            "clearCache": false,
            "globalStoragePath": worktree.root_path(),
            "licenceKey": "",
        })))
    }

    // ── Workspace configuration (can be updated at runtime) ───────────────

    fn language_server_workspace_configuration(
        &mut self,
        _language_server_id: &LanguageServerId,
        _worktree: &zed::Worktree,
    ) -> Result<Option<serde_json::Value>> {
        Ok(Some(serde_json::json!({
            "intelephense": {
                // ── File handling ────────────────────────────────────────────
                "files": {
                    "maxSize": 5_000_000,
                    "associations": ["*.php", "*.blade.php", "*.phtml"],
                    "exclude": [
                        "**/.git/**",
                        "**/.svn/**",
                        "**/node_modules/**",
                        "**/vendor/composer/**",
                        "**/storage/framework/**",
                        "**/storage/logs/**",
                        "**/bootstrap/cache/**"
                    ]
                },
                // ── PHP stubs ────────────────────────────────────────────────
                // Include the full set of built-in and common framework stubs.
                "stubs": [
                    "apache",
                    "bcmath",
                    "bz2",
                    "calendar",
                    "com_dotnet",
                    "Core",
                    "ctype",
                    "curl",
                    "date",
                    "dba",
                    "dom",
                    "enchant",
                    "exif",
                    "FFI",
                    "fileinfo",
                    "filter",
                    "fpm",
                    "ftp",
                    "gd",
                    "gettext",
                    "gmp",
                    "hash",
                    "iconv",
                    "imagick",
                    "imap",
                    "intl",
                    "json",
                    "ldap",
                    "libxml",
                    "mbstring",
                    "mcrypt",
                    "meta",
                    "mongodb",
                    "mysqli",
                    "oci8",
                    "odbc",
                    "openssl",
                    "pcntl",
                    "pcre",
                    "PDO",
                    "pdo_ibm",
                    "pdo_mysql",
                    "pdo_pgsql",
                    "pdo_sqlite",
                    "pgsql",
                    "Phar",
                    "posix",
                    "pspell",
                    "readline",
                    "Reflection",
                    "redis",
                    "session",
                    "shmop",
                    "SimpleXML",
                    "snmp",
                    "soap",
                    "sockets",
                    "sodium",
                    "SPL",
                    "sqlite3",
                    "standard",
                    "superglobals",
                    "sysvmsg",
                    "sysvsem",
                    "sysvshm",
                    "tidy",
                    "tokenizer",
                    "xml",
                    "xmlreader",
                    "xmlrpc",
                    "xmlwriter",
                    "xsl",
                    "Zend OPcache",
                    "zip",
                    "zlib",
                    "wordpress",
                    "phpunit"
                ],
                // ── Diagnostics ──────────────────────────────────────────────
                "diagnostics": {
                    "enable": true,
                    "undefinedClassConstants": true,
                    "undefinedConstants": true,
                    "undefinedFunctions": true,
                    "undefinedMethods": true,
                    "undefinedProperties": true,
                    "undefinedSymbols": true,
                    "undefinedTypes": true,
                    "unexpectedTokens": true
                },
                // ── Completion ───────────────────────────────────────────────
                "completion": {
                    "insertUseDeclaration": true,
                    "fullyQualifyGlobalConstantsAndFunctions": false,
                    "triggerParameterHints": true,
                    "maxItems": 100
                },
                // ── Code formatting ──────────────────────────────────────────
                "format": {
                    "enable": true,
                    "braces": "per-psr"
                },
                // ── Environment hints ────────────────────────────────────────
                "environment": {
                    "phpVersion": "8.2.0",
                    "shortOpenTag": false,
                    "includePaths": []
                },
                // ── Telemetry ────────────────────────────────────────────────
                "telemetry": {
                    "enable": false
                }
            }
        })))
    }

    // ── Label enhancement ─────────────────────────────────────────────────

    /// Enhances the completion labels Zed shows in the UI.
    ///
    /// Returns `None` to keep Zed's default rendering for completions that
    /// don't need special treatment.
    fn label_for_completion(
        &self,
        language_server_id: &LanguageServerId,
        completion: Completion,
    ) -> Option<CodeLabel> {
        let label = &completion.label;

        // ── Artisan command label ──────────────────────────────────────────
        // When the user is completing inside an Artisan call, annotate the
        // suggestion with a subtle "artisan" hint.
        if let Some(cmd) = artisan::extract_command(label) {
            if artisan::is_artisan_command(cmd) {
                let hint = " // artisan";
                let code = format!("// artisan {}", label);
                let start = "// artisan ".len() as u32;
                let end = start + label.len() as u32;
                let _ = hint; // used only for length calculation above
                return Some(CodeLabel {
                    code,
                    spans: vec![CodeLabelSpan::code_range(start..end)],
                    filter_range: (0u32..label.len() as u32).into(),
                });
            }
        }

        // ── Laravel helper function label ─────────────────────────────────
        // Render helper functions with a `fn name()` code snippet so
        // Tree-sitter can highlight them as function tokens.
        if completion::is_laravel_helper(label) {
            let prefix = "fn ";
            let code = format!("{}{}()", prefix, label);
            let start = prefix.len() as u32;
            let end = start + label.len() as u32;
            return Some(CodeLabel {
                code,
                spans: vec![CodeLabelSpan::code_range(start..end)],
                filter_range: (0u32..label.len() as u32).into(),
            });
        }

        // ── Laravel facade label ──────────────────────────────────────────
        if completion::is_laravel_facade(label) {
            let prefix = "class ";
            let code = format!("{}{}", prefix, label);
            let start = prefix.len() as u32;
            let end = start + label.len() as u32;
            return Some(CodeLabel {
                code,
                spans: vec![CodeLabelSpan::code_range(start..end)],
                filter_range: (0u32..label.len() as u32).into(),
            });
        }

        // ── Generic LSP completion ────────────────────────────────────────
        // Delegate to the shared label builder for everything else.
        let _ = language_server_id;
        completion::build_label(&completion)
    }
}

// ─── Private helpers ──────────────────────────────────────────────────────────

impl LaravelExtension {
    /// Ensures that `intelephense` is installed and returns the path to its
    /// entry-point script.
    ///
    /// On the first call this checks npm for the latest version and installs /
    /// updates if necessary.  Subsequent calls return the cached path directly.
    fn ensure_intelephense(&mut self, language_server_id: &LanguageServerId) -> Result<String> {
        // Return early if we already know the script is on disk.
        if let Some(ref path) = self.cached_server_path {
            if std::path::Path::new(path).exists() {
                return Ok(path.clone());
            }
        }

        // ── Check for updates ──────────────────────────────────────────────
        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::CheckingForUpdate,
        );

        let installed = zed::npm_package_installed_version(INTELEPHENSE_PACKAGE)?;
        let latest = zed::npm_package_latest_version(INTELEPHENSE_PACKAGE)?;

        // ── Install / upgrade if needed ────────────────────────────────────
        if installed.as_deref() != Some(latest.as_str()) {
            zed::set_language_server_installation_status(
                language_server_id,
                &zed::LanguageServerInstallationStatus::Downloading,
            );

            zed::npm_install_package(INTELEPHENSE_PACKAGE, &latest)?;
        }

        // ── Verify the script exists ───────────────────────────────────────
        if !std::path::Path::new(INTELEPHENSE_SERVER_PATH).exists() {
            return Err(format!(
                "intelephense script not found at '{}' after installation",
                INTELEPHENSE_SERVER_PATH
            ));
        }

        zed::set_language_server_installation_status(
            language_server_id,
            &zed::LanguageServerInstallationStatus::None,
        );

        self.cached_server_path = Some(INTELEPHENSE_SERVER_PATH.to_string());
        Ok(INTELEPHENSE_SERVER_PATH.to_string())
    }
}

// ─── Registration ─────────────────────────────────────────────────────────────

zed::register_extension!(LaravelExtension);
