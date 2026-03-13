<?php
require __DIR__ . "/../vendor/autoload.php";

$routes = [];

foreach (require __DIR__ . "/../routes/web.php" as $route) {
    $routes[] = $route;
}

echo json_encode($routes);
