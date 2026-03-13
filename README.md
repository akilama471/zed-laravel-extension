![License](https://img.shields.io/github/license/akilama471/zed-laravel-extension)
![Zed Extension](https://img.shields.io/badge/Zed-Extension-blue)
![Version](https://img.shields.io/badge/version-0.1.0-green)

# Laravel Support for Zed

A [Zed](https://zed.dev) extension that brings first-class Laravel development tools directly into your editor — including Blade syntax support, Artisan task integration, and route list utilities.

---

## Features

### 🎨 Blade Template Language Support

Full language support for Laravel's Blade templating engine (`.blade.php` files):

- **Syntax Highlighting** — Blade directives are highlighted correctly within HTML/PHP context using the `text.html.php.blade` scope.
- **File Association** — Automatically activates for all `.blade.php` files.
- **Built-in Snippets** — Commonly used Blade directives are available as snippets:

  | Prefix       | Expands To                                      |
  |--------------|-------------------------------------------------|
  | `@if`        | `@if (condition) ... @endif`                    |
  | `@foreach`   | `@foreach (items as item) ... @endforeach`      |

---

### ⚙️ Artisan Task Runner

Run common Artisan commands directly from Zed's task panel without leaving your editor:

| Task Label                  | Command                                      |
|-----------------------------|----------------------------------------------|
| **Laravel: Serve**          | `php artisan serve`                          |
| **Laravel: Make Controller**| `php artisan make:controller {ControllerName}` |
| **Laravel: Make Model**     | `php artisan make:model {ModelName}`         |
| **Laravel: Migrate**        | `php artisan migrate`                        |

Tasks with placeholders (e.g. `{ControllerName}`) will prompt you for input when run.

---

### 🗺️ Route List Utility

Includes a helper script (`commands/route_list.php`) that reads your application's `routes/web.php` and outputs the registered routes as JSON — useful for tooling and autocomplete integrations.

---

### 🔧 PHP Language Server Integration

The extension registers and activates the built-in **PHP language server** (`php`) for Laravel projects, providing:

- Code completion
- Go-to-definition
- Hover documentation
- Diagnostics

---

## Requirements

- [Zed Editor](https://zed.dev) (latest stable recommended)
- PHP installed and available in your `PATH`
- A Laravel project with `vendor/autoload.php` present (for route utilities)

---

## Installation

1. Open Zed and go to the **Extensions** panel (`Ctrl+Shift+X` / `Cmd+Shift+X`).
2. Search for **Laravel Support**.
3. Click **Install**.

Alternatively, install via the Zed CLI:

```sh
zed extension install laravel-support
```

---

## Usage

### Running Artisan Tasks

1. Open the Zed task panel with `Ctrl+Shift+R` / `Cmd+Shift+R` (or via the Command Palette → `task: spawn`).
2. Select any **Laravel:** task from the list.
3. For tasks with placeholders, enter the required name when prompted.

### Blade Snippets

Open any `.blade.php` file and start typing a snippet prefix (e.g. `@if`) — Zed's completion menu will suggest the full snippet expansion.

---

## Project Structure

```
zed-laravel-extension/
├── commands/
│   ├── artisan.rs          # Artisan command definitions (Rust)
│   └── route_list.php      # Route listing helper script
├── languages/
│   └── blade/
│       └── config.toml     # Blade language config & snippets
├── tasks/
│   └── artisan.toml        # Artisan task definitions
├── extension.toml          # Extension manifest
└── Cargo.toml              # Rust crate configuration
```

---

## Changelog

### [0.1.0] — Initial Release

#### Added
- **Blade language support** — syntax scope, file extension association (`.blade.php`), and built-in `@if` / `@foreach` snippets.
- **Artisan task integration** — predefined tasks for `serve`, `make:controller`, `make:model`, and `migrate`.
- **Route list utility** — PHP helper script to extract and expose registered web routes as JSON.
- **PHP language server** — automatic activation of the PHP LSP for Laravel projects.

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
