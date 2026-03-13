/// All known Artisan commands shipped with a standard Laravel installation.
pub static ARTISAN_COMMANDS: &[&str] = &[
    // Application
    "about",
    "completion",
    "down",
    "env",
    "help",
    "inspire",
    "list",
    "migrate",
    "optimize",
    "optimize:clear",
    "serve",
    "test",
    "tinker",
    "up",
    // Auth
    "auth:clear-resets",
    // Cache
    "cache:clear",
    "cache:forget",
    "cache:prune-stale-tags",
    "cache:table",
    // Config
    "config:cache",
    "config:clear",
    "config:publish",
    "config:show",
    // Database
    "db",
    "db:monitor",
    "db:seed",
    "db:show",
    "db:table",
    "db:wipe",
    // Event
    "event:cache",
    "event:clear",
    "event:generate",
    "event:list",
    // Key
    "key:generate",
    // Make
    "make:cast",
    "make:channel",
    "make:class",
    "make:command",
    "make:component",
    "make:controller",
    "make:enum",
    "make:event",
    "make:exception",
    "make:factory",
    "make:interface",
    "make:job",
    "make:listener",
    "make:livewire",
    "make:mail",
    "make:middleware",
    "make:migration",
    "make:model",
    "make:notification",
    "make:observer",
    "make:policy",
    "make:provider",
    "make:request",
    "make:resource",
    "make:rule",
    "make:scope",
    "make:seeder",
    "make:test",
    "make:trait",
    "make:view",
    // Migrate
    "migrate:fresh",
    "migrate:install",
    "migrate:refresh",
    "migrate:reset",
    "migrate:rollback",
    "migrate:status",
    // Model
    "model:prune",
    "model:show",
    // Notification
    "notification:table",
    // Package
    "package:discover",
    // Queue
    "queue:batches-table",
    "queue:clear",
    "queue:failed",
    "queue:failed-table",
    "queue:flush",
    "queue:forget",
    "queue:listen",
    "queue:monitor",
    "queue:prune-batches",
    "queue:prune-failed",
    "queue:restart",
    "queue:retry",
    "queue:retry-batch",
    "queue:table",
    "queue:work",
    // Route
    "route:cache",
    "route:clear",
    "route:list",
    // Schedule
    "schedule:clear-cache",
    "schedule:interrupt",
    "schedule:list",
    "schedule:run",
    "schedule:test",
    "schedule:work",
    // Schema
    "schema:dump",
    // Session
    "session:table",
    // Storage
    "storage:link",
    "storage:unlink",
    // Stub
    "stub:publish",
    // Vendor
    "vendor:publish",
    // View
    "view:cache",
    "view:clear",
];

/// Returns `true` when `name` exactly matches a known Artisan command.
pub fn is_artisan_command(name: &str) -> bool {
    ARTISAN_COMMANDS.binary_search(&name).is_ok()
}

/// Returns the Artisan sub-command portion of a full command string.
///
/// e.g. `"make:controller"` → `Some("make:controller")`
///       `"php artisan make:controller"` → `Some("make:controller")`
pub fn extract_command(input: &str) -> Option<&str> {
    let trimmed = input.trim();
    // Strip leading "php artisan " if present
    let cmd = if let Some(rest) = trimmed.strip_prefix("php artisan ") {
        rest.trim()
    } else if let Some(rest) = trimmed.strip_prefix("artisan ") {
        rest.trim()
    } else {
        trimmed
    };

    if cmd.is_empty() {
        None
    } else {
        Some(cmd)
    }
}

/// Returns the namespace portion of an Artisan command.
///
/// e.g. `"make:controller"` → `Some("make")`
///       `"migrate"` → `None`
#[allow(dead_code)]
pub fn command_namespace(command: &str) -> Option<&str> {
    command.split_once(':').map(|(ns, _)| ns)
}

/// Returns all commands that belong to the given namespace.
///
/// e.g. `commands_in_namespace("make")` returns all `make:*` commands.
#[allow(dead_code)]
pub fn commands_in_namespace(namespace: &str) -> Vec<&'static str> {
    ARTISAN_COMMANDS
        .iter()
        .copied()
        .filter(|cmd| {
            cmd.split_once(':')
                .map(|(ns, _)| ns == namespace)
                .unwrap_or(false)
        })
        .collect()
}

/// Returns all top-level namespaces present in the Artisan command list.
#[allow(dead_code)]
pub fn all_namespaces() -> Vec<&'static str> {
    let mut seen = std::collections::HashSet::new();
    let mut namespaces = Vec::new();
    for cmd in ARTISAN_COMMANDS {
        if let Some((ns, _)) = cmd.split_once(':') {
            if seen.insert(ns) {
                namespaces.push(ns);
            }
        }
    }
    namespaces
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_command_is_recognised() {
        assert!(is_artisan_command("migrate"));
        assert!(is_artisan_command("make:controller"));
        assert!(is_artisan_command("route:list"));
    }

    #[test]
    fn unknown_command_is_rejected() {
        assert!(!is_artisan_command("make:banana"));
        assert!(!is_artisan_command("php artisan migrate"));
    }

    #[test]
    fn extract_command_strips_prefix() {
        assert_eq!(extract_command("php artisan migrate"), Some("migrate"));
        assert_eq!(extract_command("artisan route:list"), Some("route:list"));
        assert_eq!(extract_command("tinker"), Some("tinker"));
        assert_eq!(extract_command("  "), None);
    }

    #[test]
    fn namespace_extraction() {
        assert_eq!(command_namespace("make:controller"), Some("make"));
        assert_eq!(command_namespace("migrate"), None);
    }

    #[test]
    fn commands_in_namespace_returns_subset() {
        let make_cmds = commands_in_namespace("make");
        assert!(make_cmds.contains(&"make:controller"));
        assert!(make_cmds.contains(&"make:model"));
        assert!(!make_cmds.contains(&"migrate"));
    }
}
