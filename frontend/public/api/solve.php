<?php
declare(strict_types=1);

header('Content-Type: application/json; charset=utf-8');

function fail_json(int $status, string $message, array $extra = []): never {
    http_response_code($status);
    echo json_encode(['status' => 'error', 'error' => $message] + $extra, JSON_UNESCAPED_SLASHES);
    exit;
}

$raw = file_get_contents('php://input');
$request = json_decode($raw ?: '', true);
if (!is_array($request)) {
    fail_json(400, 'Invalid JSON request.');
}

$state = $request['state'] ?? $request;
if (!is_array($state)) {
    fail_json(400, 'Missing state object.');
}

$layout = strtoupper((string)($state['layoutId'] ?? $state['layout'] ?? 'E'));
$difficulty = strtolower((string)($state['difficultyId'] ?? $state['difficulty'] ?? 'classic'));
$profile = strtolower((string)($request['profile'] ?? $state['profile'] ?? 'auto'));
$colors = $state['colors'] ?? null;
$timeoutMs = (int)($request['timeoutMs'] ?? 300000);
if ($timeoutMs <= 0) {
    $timeoutMs = 300000;
}
$timeoutMs = max(1000, min($timeoutMs, 900000));
$timeoutSeconds = (int)ceil($timeoutMs / 1000);
set_time_limit($timeoutSeconds + 30);

if (!preg_match('/^[A-F]$/', $layout)) {
    fail_json(400, 'Invalid layout.');
}
if (!in_array($difficulty, ['easy', 'moderate', 'classic'], true)) {
    fail_json(400, 'Invalid difficulty.');
}
if ($profile === 'auto') {
    if ($layout === 'E' && $difficulty === 'classic') {
        $profile = 'quality';
    } elseif ($layout === 'E') {
        $profile = 'balanced';
    } else {
        $profile = 'fast';
    }
}
if (!in_array($profile, ['fast', 'balanced', 'quality'], true)) {
    fail_json(400, 'Invalid solver profile.');
}
if (!is_array($colors) || count($colors) === 0) {
    fail_json(400, 'Missing colors array.');
}

$colorValues = [];
foreach ($colors as $color) {
    if (!is_int($color) && !(is_string($color) && preg_match('/^\d+$/', $color))) {
        fail_json(400, 'Colors must be integer codes.');
    }
    $value = (int)$color;
    if ($value < 0 || $value > 5) {
        fail_json(400, 'Color code out of range.');
    }
    $colorValues[] = (string)$value;
}

$solverDir = realpath(dirname(__DIR__)) ?: dirname(__DIR__);
$solverBin = $solverDir . '/bin/ashtree_native_bench';
if (!is_executable($solverBin)) {
    fail_json(500, 'Solver binary is not executable.');
}

$cmd = implode(' ', [
    'cd',
    escapeshellarg($solverDir),
    '&&',
    'timeout',
    escapeshellarg($timeoutSeconds . 's'),
    escapeshellarg($solverBin),
    'solve-state',
    '--layout',
    escapeshellarg($layout),
    '--difficulty',
    escapeshellarg($difficulty),
    '--profile',
    escapeshellarg($profile),
    '--colors',
    escapeshellarg(implode(',', $colorValues)),
]);

$lines = [];
$exitCode = 0;
exec($cmd . ' 2>&1', $lines, $exitCode);
$output = trim(implode("\n", $lines));

if ($exitCode !== 0) {
    if ($exitCode === 124) {
        echo json_encode([
            'status' => 'timeout',
            'found' => false,
            'layout' => $layout,
            'difficulty' => $difficulty,
            'profile' => $profile,
            'method' => 'timeout',
            'reason' => 'timeout',
            'moves' => [],
            'text' => '',
            'nodes' => 0,
            'elapsedMs' => $timeoutSeconds * 1000,
        ], JSON_UNESCAPED_SLASHES);
        exit;
    }
    fail_json(500, 'Solver process failed.', [
        'exitCode' => $exitCode,
        'output' => mb_substr($output, 0, 2000),
    ]);
}

$result = json_decode($output, true);
if (!is_array($result)) {
    fail_json(500, 'Solver returned invalid JSON.', [
        'output' => mb_substr($output, 0, 2000),
    ]);
}

echo json_encode($result, JSON_UNESCAPED_SLASHES);
