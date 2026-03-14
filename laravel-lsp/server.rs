mod indexer;
mod scanner;

use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

use scanner::{scan_laravel_project, LaravelIndex};

// ─── Laravel helper functions ─────────────────────────────────────────────────

static LARAVEL_HELPERS: &[(&str, &str)] = &[
    ("abort",           "abort(int $code, string $message = '')"),
    ("abort_if",        "abort_if(bool $condition, int $code, string $message = '')"),
    ("abort_unless",    "abort_unless(bool $condition, int $code, string $message = '')"),
    ("app",             "app(string|null $abstract = null): mixed"),
    ("app_path",        "app_path(string $path = ''): string"),
    ("asset",           "asset(string $path, bool|null $secure = null): string"),
    ("auth",            "auth(string|null $guard = null): \\Illuminate\\Auth\\AuthManager"),
    ("back",            "back(int $status = 302, array $headers = [], bool $fallback = false): RedirectResponse"),
    ("base_path",       "base_path(string $path = ''): string"),
    ("bcrypt",          "bcrypt(string $value, array $options = []): string"),
    ("blank",           "blank(mixed $value): bool"),
    ("broadcast",       "broadcast(mixed $event): PendingBroadcast"),
    ("cache",           "cache(array|string|null $key = null, mixed $default = null): mixed"),
    ("config",          "config(array|string|null $key = null, mixed $default = null): mixed"),
    ("config_path",     "config_path(string $path = ''): string"),
    ("cookie",          "cookie(string|null $name = null, mixed $value = null, int $minutes = 0): mixed"),
    ("csrf_field",      "csrf_field(): HtmlString"),
    ("csrf_token",      "csrf_token(): string"),
    ("database_path",   "database_path(string $path = ''): string"),
    ("dd",              "dd(mixed ...$vars): never"),
    ("dispatch",        "dispatch(mixed $job): mixed"),
    ("dispatch_sync",   "dispatch_sync(mixed $job, mixed $handler = null): mixed"),
    ("dump",            "dump(mixed ...$vars): void"),
    ("encrypt",         "encrypt(mixed $value, bool $serialize = true): string"),
    ("env",             "env(string $key, mixed $default = null): mixed"),
    ("event",           "event(string|object $event, mixed $payload = [], bool $halt = false): array|null"),
    ("filled",          "filled(mixed $value): bool"),
    ("info",            "info(string $message, array $context = []): void"),
    ("lang_path",       "lang_path(string $path = ''): string"),
    ("logger",          "logger(string|null $message = null, array $context = []): mixed"),
    ("method_field",    "method_field(string $method): HtmlString"),
    ("mix",             "mix(string $path, string $manifestDirectory = ''): HtmlString|string"),
    ("now",             "now(DateTimeZone|string|null $tz = null): Illuminate\\Support\\Carbon"),
    ("old",             "old(string|null $key = null, mixed $default = null): mixed"),
    ("optional",        "optional(mixed $value = null, callable|null $callback = null): mixed"),
    ("policy",          "policy(object|string $class): mixed"),
    ("public_path",     "public_path(string $path = ''): string"),
    ("redirect",        "redirect(string|null $to = null, int $status = 302): mixed"),
    ("report",          "report(Throwable|string $exception): void"),
    ("request",         "request(array|string|null $key = null, mixed $default = null): mixed"),
    ("rescue",          "rescue(callable $callback, mixed $rescue = null, bool $report = true): mixed"),
    ("resolve",         "resolve(string $name, array $parameters = []): mixed"),
    ("resource_path",   "resource_path(string $path = ''): string"),
    ("response",        "response(mixed $content = null, int $status = 200, array $headers = []): mixed"),
    ("retry",           "retry(int $times, callable $callback, int $sleep = 0, callable|null $when = null): mixed"),
    ("route",           "route(string $name, mixed $parameters = [], bool $absolute = true): string"),
    ("secure_asset",    "secure_asset(string $path): string"),
    ("secure_url",      "secure_url(string $path, mixed $parameters = []): string"),
    ("session",         "session(array|string|null $key = null, mixed $default = null): mixed"),
    ("storage_path",    "storage_path(string $path = ''): string"),
    ("str",             "str(string|null $string = null): mixed"),
    ("tap",             "tap(mixed $value, callable|null $callback = null): mixed"),
    ("throw_if",        "throw_if(mixed $condition, Throwable|string $exception, mixed ...$parameters): mixed"),
    ("throw_unless",    "throw_unless(mixed $condition, Throwable|string $exception, mixed ...$parameters): mixed"),
    ("today",           "today(DateTimeZone|string|null $tz = null): Illuminate\\Support\\Carbon"),
    ("trans",           "trans(string|null $key = null, array $replace = [], string|null $locale = null): mixed"),
    ("trans_choice",    "trans_choice(string $key, int|float|array $number, array $replace = [], string|null $locale = null): string"),
    ("to_route",        "to_route(string $route, mixed $parameters = [], int $status = 302, array $headers = []): RedirectResponse"),
    ("url",             "url(string|null $path = null, mixed $parameters = [], bool|null $secure = null): mixed"),
    ("validator",       "validator(array $data = [], array $rules = [], array $messages = [], array $attributes = []): mixed"),
    ("value",           "value(mixed $value, mixed ...$args): mixed"),
    ("view",            "view(string|null $view = null, mixed $data = [], array $mergeData = []): mixed"),
    ("with",            "with(mixed $value, callable|null $callback = null): mixed"),
];

// ─── Laravel facades ──────────────────────────────────────────────────────────

static LARAVEL_FACADES: &[(&str, &str)] = &[
    ("App", "Illuminate\\Foundation\\Application"),
    ("Artisan", "Illuminate\\Contracts\\Console\\Kernel"),
    ("Auth", "Illuminate\\Auth\\AuthManager"),
    ("Blade", "Illuminate\\View\\Compilers\\BladeCompiler"),
    ("Broadcast", "Illuminate\\Contracts\\Broadcasting\\Factory"),
    ("Bus", "Illuminate\\Contracts\\Bus\\Dispatcher"),
    ("Cache", "Illuminate\\Cache\\CacheManager"),
    ("Config", "Illuminate\\Config\\Repository"),
    ("Cookie", "Illuminate\\Cookie\\CookieJar"),
    ("Crypt", "Illuminate\\Encryption\\Encrypter"),
    ("DB", "Illuminate\\Database\\DatabaseManager"),
    ("Event", "Illuminate\\Events\\Dispatcher"),
    ("File", "Illuminate\\Filesystem\\Filesystem"),
    ("Gate", "Illuminate\\Contracts\\Auth\\Access\\Gate"),
    ("Hash", "Illuminate\\Hashing\\HashManager"),
    ("Http", "Illuminate\\Http\\Client\\Factory"),
    ("Lang", "Illuminate\\Translation\\Translator"),
    ("Log", "Illuminate\\Log\\LogManager"),
    ("Mail", "Illuminate\\Mail\\Mailer"),
    ("Notification", "Illuminate\\Notifications\\ChannelManager"),
    (
        "Password",
        "Illuminate\\Auth\\Passwords\\PasswordBrokerManager",
    ),
    ("Queue", "Illuminate\\Queue\\QueueManager"),
    ("Redirect", "Illuminate\\Routing\\Redirector"),
    ("Request", "Illuminate\\Http\\Request"),
    (
        "Response",
        "Illuminate\\Contracts\\Routing\\ResponseFactory",
    ),
    ("Route", "Illuminate\\Routing\\Router"),
    ("Schema", "Illuminate\\Database\\Schema\\Builder"),
    ("Session", "Illuminate\\Session\\SessionManager"),
    ("Storage", "Illuminate\\Filesystem\\FilesystemManager"),
    ("URL", "Illuminate\\Routing\\UrlGenerator"),
    ("Validator", "Illuminate\\Validation\\Factory"),
    ("View", "Illuminate\\View\\Factory"),
];

// ─── Backend ──────────────────────────────────────────────────────────────────

struct Backend {
    client: Client,
    index: Arc<Mutex<LaravelIndex>>,
    /// The workspace root path stored during `initialize` so that `rescan`
    /// can re-index without receiving `rootUri` a second time.
    root_path: Arc<Mutex<Option<PathBuf>>>,
}

// ─── LanguageServer impl ──────────────────────────────────────────────────────

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    // ── Lifecycle ─────────────────────────────────────────────────────────

    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        // Scan the workspace root so completions are ready on the first request,
        // and persist the root path for subsequent re-scans on file save/open.
        if let Some(root_uri) = params.root_uri {
            if let Ok(root_path) = root_uri.to_file_path() {
                let new_index = scan_laravel_project(&root_path);
                *self.index.lock().await = new_index;
                *self.root_path.lock().await = Some(root_path);
                self.client
                    .log_message(MessageType::INFO, "Laravel LSP: project scanned.")
                    .await;
            }
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "laravel-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
            capabilities: ServerCapabilities {
                // Sync whole-file content so we can re-scan on save if needed
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(false),
                        })),
                        ..Default::default()
                    },
                )),
                // Completions triggered by `'`, `"`, `(`, `>`, `:`, `@`
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "'".into(),
                        "\"".into(),
                        "(".into(),
                        ">".into(),
                        ":".into(),
                        "@".into(),
                        "<".into(),
                    ]),
                    all_commit_characters: None,
                    work_done_progress_options: Default::default(),
                    completion_item: None,
                }),
                // hover_provider and definition_provider are not yet implemented;
                // do not advertise them to avoid misleading the LSP client.
                ..Default::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "Laravel LSP ready.")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    // ── Document events ───────────────────────────────────────────────────

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        // Re-index when a route or model file is opened
        if self.is_indexable_path(&params.text_document.uri) {
            self.rescan().await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        // Re-index when a route file, model, or Blade component is saved
        if self.is_indexable_path(&params.text_document.uri) {
            self.rescan().await;
        }
    }

    // ── Completion ────────────────────────────────────────────────────────

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let index = self.index.lock().await;
        let mut items: Vec<CompletionItem> = Vec::new();

        // ── Named routes ──────────────────────────────────────────────────
        for route in &index.routes {
            items.push(CompletionItem {
                label: route.clone(),
                kind: Some(CompletionItemKind::REFERENCE),
                detail: Some("Named Route".into()),
                insert_text: Some(route.clone()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**Named route** `{}`\n\n```php\nroute('{}')\n```",
                        route, route
                    ),
                })),
                ..Default::default()
            });
        }

        // ── Route URIs ────────────────────────────────────────────────────
        for uri in &index.route_uris {
            items.push(CompletionItem {
                label: uri.clone(),
                kind: Some(CompletionItemKind::VALUE),
                detail: Some("Route URI".into()),
                insert_text: Some(uri.clone()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Route URI** `{}`", uri),
                })),
                ..Default::default()
            });
        }

        // ── Blade components ──────────────────────────────────────────────
        for component in &index.blade_components {
            // Extract component name from `<x-name />` for insert_text
            let tag_inner = component
                .trim_start_matches("<x-")
                .trim_end_matches(" />")
                .to_string();

            items.push(CompletionItem {
                label: component.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Blade Component".into()),
                // Snippet: expand as self-closing tag with cursor inside
                insert_text: Some(format!("<x-{} ${{1}} />", tag_inner)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Blade Component**\n\n```blade\n{}\n```", component),
                })),
                ..Default::default()
            });
        }

        // ── Model properties ──────────────────────────────────────────────
        for prop in &index.model_properties {
            items.push(CompletionItem {
                label: prop.clone(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some("Model Property".into()),
                insert_text: Some(prop.clone()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**Fillable property** `{}`\n\nDeclared in a model's `$fillable` array.",
                        prop
                    ),
                })),
                ..Default::default()
            });
        }

        // ── Controllers ───────────────────────────────────────────────────
        for ctrl in &index.controllers {
            items.push(CompletionItem {
                label: ctrl.clone(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Controller".into()),
                insert_text: Some(format!("{}::class", ctrl)),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Laravel Controller** `{}`", ctrl),
                })),
                ..Default::default()
            });
        }

        // ── Laravel helper functions ──────────────────────────────────────
        for (name, signature) in LARAVEL_HELPERS {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("Laravel Helper".into()),
                // Snippet: place cursor inside the parentheses
                insert_text: Some(format!("{}(${{1}})", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Laravel Helper**\n\n```php\n{}\n```", signature),
                })),
                ..Default::default()
            });
        }

        // ── Laravel facades ───────────────────────────────────────────────
        for (facade, fqn) in LARAVEL_FACADES {
            items.push(CompletionItem {
                label: facade.to_string(),
                kind: Some(CompletionItemKind::CLASS),
                detail: Some("Laravel Facade".into()),
                insert_text: Some(facade.to_string()),
                insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("**Laravel Facade** `{}`\n\nAlias for `{}`", facade, fqn),
                })),
                ..Default::default()
            });
        }

        if items.is_empty() {
            return Ok(None);
        }

        Ok(Some(CompletionResponse::Array(items)))
    }

    // ── Hover ─────────────────────────────────────────────────────────────

    // hover and goto_definition are not yet implemented and their capabilities
    // are not advertised in InitializeResult, so these handlers are never called.
    // They are kept as stubs so the trait impl remains complete.
    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        Ok(None)
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        Ok(None)
    }
}

// ─── Backend helpers ──────────────────────────────────────────────────────────

impl Backend {
    fn new(client: Client) -> Self {
        Backend {
            client,
            index: Arc::new(Mutex::new(LaravelIndex::default())),
            root_path: Arc::new(Mutex::new(None)),
        }
    }

    /// Returns `true` for PHP files that are likely to affect the index
    /// (routes, models, Blade components).
    fn is_indexable_path(&self, uri: &Url) -> bool {
        let path = uri.path().to_lowercase();
        path.contains("/routes/")
            || path.contains("/models/")
            || path.contains("/components/")
            || path.ends_with(".blade.php")
    }

    /// Re-scans the project using the root path captured during `initialize`.
    async fn rescan(&self) {
        let maybe_root = self.root_path.lock().await.clone();
        if let Some(root) = maybe_root {
            let new_index = scan_laravel_project(&root);
            *self.index.lock().await = new_index;
            self.client
                .log_message(MessageType::INFO, "Laravel LSP: re-indexed project.")
                .await;
        } else {
            self.client
                .log_message(
                    MessageType::WARNING,
                    "Laravel LSP: rescan requested but root path is unknown.",
                )
                .await;
        }
    }
}

// ─── Entry point ──────────────────────────────────────────────────────────────

#[tokio::main]
async fn main() {
    // Parse CLI flags
    let args: Vec<String> = std::env::args().collect();

    let use_stdio = args.iter().any(|a| a == "--stdio");
    let use_tcp: Option<u16> = args
        .windows(2)
        .find(|w| w[0] == "--port")
        .and_then(|w| w[1].parse().ok());

    if let Some(port) = use_tcp {
        // TCP transport (useful for debugging with a local client)
        let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port))
            .await
            .expect("Failed to bind TCP port");

        eprintln!("Laravel LSP listening on port {}", port);

        loop {
            let (stream, _) = listener
                .accept()
                .await
                .expect("Failed to accept connection");
            let (read, write) = tokio::io::split(stream);

            let (service, socket) = LspService::new(Backend::new);
            Server::new(read, write, socket).serve(service).await;
        }
    } else if use_stdio || args.len() == 1 {
        // stdio transport (default — used by Zed)
        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();

        let (service, socket) = LspService::new(Backend::new);
        Server::new(stdin, stdout, socket).serve(service).await;
    } else {
        eprintln!("Usage: laravel-lsp [--stdio] [--port <port>]");
        std::process::exit(1);
    }
}
