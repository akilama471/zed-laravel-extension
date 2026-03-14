![License](https://img.shields.io/github/license/akilama471/zed-laravel-extension)
![Zed Extension](https://img.shields.io/badge/Zed-Extension-blue)
![Version](https://img.shields.io/badge/version-0.1.0-green)

# Laravel Support for Zed

A [Zed](https://zed.dev) extension that brings first-class Laravel development tools directly into your editor — including Blade & PHP syntax support with rich snippets, smart Laravel-aware completions, a custom Laravel LSP server, Artisan task integration, and route/model/component indexing.

---

## Features

### 🎨 Blade Template Language Support

Full language support for Laravel's Blade templating engine (`.blade.php` files):

- **Syntax Highlighting** — Blade directives highlighted within HTML/PHP context using the `text.html.php.blade` scope.
- **File Association** — Automatically activates for all `.blade.php` files.
- **Block Comments** — Native `{{-- --}}` comment toggling.
- **TailwindCSS** — Opt-in support for `tailwindcss-language-server` in Blade files and strings.
- **Prettier** — Integration with [`@shufo/prettier-plugin-blade`](https://github.com/shufo/prettier-plugin-blade) for formatting.
- **60+ Built-in Snippets** — Organised by category:

  | Category | Prefixes |
  |---|---|
  | **Control Flow** | `@if`, `@ifelse`, `@elseif`, `@unless`, `@isset`, `@empty` |
  | **Loops** | `@foreach`, `@forelse`, `@for`, `@while`, `@continue`, `@break` |
  | **Switch** | `@switch`, `@case` |
  | **Layout** | `@extends`, `@section`, `@yield`, `@show`, `@parent` |
  | **Includes** | `@include`, `@includeIf`, `@includeWhen`, `@includeUnless`, `@includeFirst`, `@each` |
  | **Stacks** | `@stack`, `@push`, `@pushonce`, `@prepend` |
  | **Components** | `@component`, `@slot`, `x-component`, `x-self-closing` |
  | **Auth / Gates** | `@auth`, `@guest`, `@can`, `@cannot`, `@canany` |
  | **Environment** | `@env`, `@production` |
  | **Forms** | `@csrf`, `@method`, `@error` |
  | **Livewire** | `@livewire`, `@livewireStyles`, `@livewireScripts` |
  | **Translations** | `@lang` |
  | **Raw PHP** | `@verbatim`, `@php` |
  | **Vite** | `@vite`, `@viteReactRefresh` |

---

### 🐘 PHP Language Support

Full PHP language support with a rich Laravel-focused snippet library for `.php` files:

- **Syntax Highlighting** — Using the built-in `php` grammar.
- **100+ Built-in Snippets** — Covering every layer of a Laravel application:

  | Category | Prefixes |
  |---|---|
  | **Migration Columns** | `colstring`, `coltext`, `collongtext`, `colint`, `colbigint`, `colunsigned`, `colfloat`, `coldecimal`, `colbool`, `coldate`, `coldatetime`, `coltimestamp`, `coltimestamps`, `colsoftdeletes`, `coljson`, `coljsonb`, `colenum`, `colforeign`, `colnullable`, `coldefault`, `colunique`, `colindex`, `colid`, `coluuid`, `colip`, `colmac`, `colmorph` |
  | **Migration Structure** | `migration`, `migalter` |
  | **Eloquent Model** | `model`, `fillable`, `hidden`, `casts`, `scope`, `hasone`, `hasmany`, `belongsto`, `belongstomany`, `morphto`, `morphmany` |
  | **Controller** | `controller`, `resourcecontroller` |
  | **Request / Validation** | `formrequest`, `validate`, `rulereq`, `rulenull` |
  | **Routes** | `routeget`, `routepost`, `routeput`, `routedelete`, `routeresource`, `routeapiresource`, `routegroup`, `routemiddleware` |
  | **Service / Repository** | `service`, `interface` |
  | **Events / Listeners / Jobs** | `event`, `listener`, `job` |
  | **Helpers & Facades** | `env`, `config`, `cache`, `cacheforget`, `session`, `sessionput`, `redirect`, `redirectback`, `view`, `response`, `abort`, `dispatch`, `log`, `dd`, `dump` |

---

### 🧠 Smart Laravel Completions

The extension enriches the editor's completion menu with Laravel-specific knowledge:

- **75+ Laravel Helper Functions** — `route()`, `view()`, `auth()`, `config()`, `session()`, `abort()`, `dispatch()`, and more — all surfaced with typed `fn` labels.
- **33 Laravel Facades** — `Auth`, `Cache`, `DB`, `Event`, `Gate`, `Log`, `Mail`, `Queue`, `Route`, `Schema`, `Storage`, and more — surfaced with `class` labels.
- **Blade Directives** — `@csrf`, `@if`, `@foreach`, `@yield`, `@auth`, and more — surfaced with `string.special` labels.
- **Typed Code Labels** — Completions are styled by kind: functions show as `fn name()`, classes as `class Name`, variables as `$name`, constants as `const NAME`, and so on.

---

### 🔍 Project-Aware Indexing

The extension scans your open Laravel project and provides context-aware completions for:

- **Blade Components** — All components under `resources/views/components/` are discovered (including nested sub-directories) and surfaced as `<x-component-name />` completions.
- **Named Routes** — All `->name('...')` calls across every file in your `routes/` directory are indexed and available as completions.
- **Model `$fillable` Properties** — All entries in `$fillable` (and `$guarded`) arrays across `app/Models/` and `app/` are indexed for completion.
- **Controllers** — All controller class names under `app/Http/Controllers/` (including sub-directories) are indexed.

---

### 🖥️ Language Server Integration

The extension registers a language server (`laravel-lsp`) for both PHP and Blade files. It uses a two-tier strategy:

1. **Custom `laravel-lsp` binary** *(if installed)* — A standalone [Tower-LSP](https://github.com/ebkalderon/tower-lsp) server (`laravel-lsp/`) that provides Laravel-aware completions for helpers, facades, blade directives, named routes, model properties, and blade components, all sourced from your live project.

2. **Intelephense** *(automatic fallback)* — If the `laravel-lsp` binary is not found, the extension automatically installs and launches the latest version of [PHP Intelephense](https://intelephense.com/) via npm, providing:
   - Code completion
   - Go-to-definition
   - Hover documentation
   - Diagnostics

---

### ⚙️ Artisan Task Runner

Run common Artisan commands directly from Zed's task panel.

> **Note:** Zed does not currently support extensions injecting tasks automatically. You must copy the task definitions into your project manually. See [Usage → Running Artisan Tasks](#running-artisan-tasks) below.

The task definitions are provided in [`tasks/artisan.json`](./tasks/artisan.json) and cover:

| Category | Tasks |
|---|---|
| **Server** | `artisan: serve` |
| **Database** | `artisan: migrate`, `artisan: migrate:fresh`, `artisan: migrate:fresh --seed`, `artisan: migrate:rollback`, `artisan: migrate:status`, `artisan: db:seed` |
| **Testing** | `artisan: test`, `artisan: test --filter $ZED_SYMBOL`, `artisan: test --parallel` |
| **Queue** | `artisan: queue:work`, `artisan: queue:listen`, `artisan: schedule:work`, `artisan: schedule:run` |
| **Routing** | `artisan: route:list`, `artisan: route:cache`, `artisan: route:clear` |
| **Cache** | `artisan: cache:clear`, `artisan: config:cache`, `artisan: config:clear`, `artisan: view:cache`, `artisan: view:clear`, `artisan: optimize`, `artisan: optimize:clear` |
| **Maintenance** | `artisan: tinker`, `artisan: key:generate`, `artisan: storage:link`, `artisan: vendor:publish`, `artisan: down`, `artisan: up`, `artisan: about`, `artisan: list` |

> **Tip:** The `artisan: test --filter $ZED_SYMBOL` task uses Zed's `$ZED_SYMBOL` variable to run only the test under your cursor.

---

### 🗺️ Route List Utility

A PHP helper script (`commands/route_list.php`) reads your application's `routes/web.php` and outputs the registered routes as JSON — useful for tooling and external integrations.

---

## Requirements

- [Zed Editor](https://zed.dev) (latest stable recommended)
- [Node.js & npm](https://nodejs.org) — required for the automatic Intelephense installation
- PHP installed and available in your `PATH`
- A Laravel project (for project-aware indexing features)

---

## Installation

1. Open Zed and go to the **Extensions** panel (`Ctrl+Shift+X` / `Cmd+Shift+X`).
2. Search for **Laravel Support**.
3. Click **Install**.

Intelephense will be automatically downloaded and installed the first time the extension activates in a PHP or Blade file.

---

## Usage

### Running Artisan Tasks

Zed's extension API does not yet support extensions injecting tasks automatically. To use the Artisan tasks you must copy the provided task definitions into your project (or your global Zed config) manually.

#### Per-project setup (recommended)

1. In your Laravel project root, create the `.zed/` directory if it doesn't already exist.
2. Copy `tasks/artisan.json` from this extension into your project as `.zed/tasks.json`:

   ```sh
   mkdir -p .zed
   cp /path/to/zed-laravel-extension/tasks/artisan.json .zed/tasks.json
   ```

   Or copy the contents of [`tasks/artisan.json`](./tasks/artisan.json) into your existing `.zed/tasks.json`.

3. Open the Zed task panel via the Command Palette → `task: spawn` (or `Alt+Shift+T`).
4. All **artisan:** tasks will now appear in the list.

#### Global setup (all projects)

To have the Artisan tasks available in every project, add the contents of `tasks/artisan.json` to your global Zed tasks file:

- **macOS/Linux:** `~/.config/zed/tasks.json`
- **Windows:** `%APPDATA%\Zed\tasks.json`

---

### Blade Snippets

Open any `.blade.php` file and type a snippet prefix (e.g. `@foreach`, `@csrf`, `x-component`) — Zed's completion menu will suggest the full snippet expansion.

### PHP Snippets

Open any `.php` file and type a snippet prefix (e.g. `migration`, `model`, `resourcecontroller`, `routeget`) — Zed's completion menu will suggest the full expansion with tab stops.

---

## Project Structure

```
zed-laravel-extension/
├── commands/
│   ├── artisan.rs              # Artisan command label helpers (Rust)
│   └── route_list.php          # Route listing helper script (JSON output)
├── languages/
│   ├── blade/
│   │   ├── config.toml         # Blade language config, brackets, Prettier & Tailwind opts
│   │   └── snippets.toml       # 60+ Blade directive snippets
│   └── php/
│       ├── config.toml         # PHP language config & brackets
│       └── snippets.toml       # 100+ PHP/Laravel snippets
├── laravel-lsp/
│   ├── indexer.rs              # Project indexer (components, routes, models, controllers)
│   ├── scanner.rs              # Low-level file scanner utilities
│   ├── server.rs               # Tower-LSP server entry point & completion handler
│   └── Cargo.toml              # laravel-lsp standalone binary crate
├── src/
│   ├── completion.rs           # Built-in helpers, facades, blade directives + label builder
│   ├── laravel_indexer.rs      # Project indexer used inside the WASM extension
│   ├── laravel_scanner.rs      # File scanner used inside the WASM extension
│   └── lib.rs                  # Extension entry point & Intelephense bootstrap
├── tasks/
│   └── artisan.json            # Artisan task definitions (copy to .zed/tasks.json)
├── extension.toml              # Extension manifest
└── Cargo.toml                  # Rust crate configuration
```

---

## Changelog

### [0.1.0] — Initial Release

#### Added
- **Blade language support** — syntax scope, file extension association (`.blade.php`), `{{-- --}}` block comments, TailwindCSS LSP opt-in, Prettier plugin integration, and 60+ `@directive` / `x-component` snippets across 14 categories.
- **PHP language support** — syntax scope, file extension association (`.php`), and 100+ PHP/Laravel snippets covering migrations, models, controllers, requests, routes, services, events, jobs, and helper/facade patterns.
- **Smart Laravel completions** — typed code labels for 75+ helper functions, 33 facades, and core Blade directives, with completion kind–aware display (`fn`, `class`, `const`, `$variable`, `keyword`).
- **Project-aware indexing** — scans Blade components, named routes, model `$fillable` properties, and controller class names from the open workspace.
- **Custom Laravel LSP** — standalone `laravel-lsp` Tower-LSP binary providing Laravel-specific hover, completion, and go-to-definition support.
- **Intelephense integration** — automatic npm installation and launch of PHP Intelephense as a fallback language server.
- **Artisan task integration** — 32 predefined tasks for `serve`, `migrate`, `test`, `queue`, `schedule`, `cache`, `optimize`, `route`, `tinker`, and more.
- **Route list utility** — PHP helper script to extract and expose registered web routes as JSON.

---

## Contributing

Contributions are welcome! Feel free to open an issue or pull request on [GitHub](https://github.com/akilama471/zed-laravel-extension).

1. Fork the repository.
2. Create a feature branch: `git checkout -b feature/my-feature`.
3. Commit your changes: `git commit -m 'Add my feature'`.
4. Push to the branch: `git push origin feature/my-feature`.
5. Open a Pull Request.

---

## License

MIT License — see [LICENSE](./LICENSE) for details.