<?php

/**
 * route_list.php
 *
 * Bootstraps the Laravel application and outputs all registered routes as JSON.
 *
 * Usage (run from project root):
 *   php commands/route_list.php
 *
 * Output format:
 *   [
 *     { "method": "GET|HEAD", "uri": "/", "name": "home", "action": "HomeController@index" },
 *     ...
 *   ]
 */

$appRoot = realpath(__DIR__ . "/..");

// Verify this looks like a Laravel project root
if (!file_exists($appRoot . "/vendor/autoload.php")) {
    fwrite(
        STDERR,
        "Error: vendor/autoload.php not found. Run 'composer install' first.\n",
    );
    echo json_encode([]);
    exit(1);
}

if (!file_exists($appRoot . "/bootstrap/app.php")) {
    fwrite(
        STDERR,
        "Error: bootstrap/app.php not found. Is this a Laravel project root?\n",
    );
    echo json_encode([]);
    exit(1);
}

require $appRoot . "/vendor/autoload.php";

/** @var \Illuminate\Foundation\Application $app */
$app = require_once $appRoot . "/bootstrap/app.php";

/** @var \Illuminate\Contracts\Console\Kernel $kernel */
$kernel = $app->make(\Illuminate\Contracts\Console\Kernel::class);
$kernel->bootstrap();

$routes = [];

foreach (\Illuminate\Support\Facades\Route::getRoutes() as $route) {
    $methods = array_filter($route->methods(), fn(string $m) => $m !== "HEAD");

    $routes[] = [
        "method" => implode("|", array_values($methods)),
        "uri" => $route->uri(),
        "name" => $route->getName(),
        "action" => $route->getActionName(),
        "middleware" => array_values(
            array_map(
                fn($m) => is_string($m) ? $m : (string) $m,
                $route->gatherMiddleware(),
            ),
        ),
    ];
}

echo json_encode($routes, JSON_PRETTY_PRINT | JSON_UNESCAPED_SLASHES);
