# Starts the title on an AMD card (RX 9070 XT is the 16 GB profile).
#   scripts\play.ps1 -Exe <path of your own copy's executable>
# -Safe         without the newer, faster ways (the first thing to try when something is wrong)
# -RayTracing   the title's own ray-traced indirect light: lit interiors, about a third slower
# -Build        target\diag (the default, the build all measurements are of) or target\release
# -Gpu          amd (default), nvidia, or any
# -Console      series-s (default, fits 16 GB) or series-x (wants about 64 GB of system RAM)
# -More         other settings: -More @{VICTORY_PANEL='1'}
# Run it from a normal window, not as administrator: the driver then keeps its shader cache elsewhere.
param(
    [Parameter(Mandatory)][string]$Exe,
    [switch]$Safe,
    [switch]$RayTracing,
    [ValidateSet('diag', 'release')][string]$Build = 'diag',
    [ValidateSet('amd', 'nvidia', 'any')][string]$Gpu = 'amd',
    [ValidateSet('series-s', 'series-x')][string]$Console = 'series-s',
    [hashtable]$More = @{}
)
$settings = @{
    VICTORY_GPU = $Gpu
    VICTORY_TEXTURE_NO_WAIT = '1'; VICTORY_RELAY = '2'; VICTORY_WIDE_BACK = '1'; VICTORY_CARD_REHASH_RUNS = '1'
    VICTORY_JOIN_COPIES = '1'; VICTORY_INDIRECT_WINDOW = '1'; VICTORY_RELAY_COPY = '1'; VICTORY_CARD_STEPS = '2'
    VICTORY_INDIRECT_BATCH = '2'; VICTORY_MAP_THREADS = '1'; VICTORY_TITLE_PRIORITIES = '0'; VICTORY_CARD_AWAKE = '1'
    VICTORY_SHOW_AFTER = '1'; VICTORY_AMBIENT = '0.8'; VICTORY_D3D12_DRED = 'light'; VICTORY_SKIP_PIPELINES = 'none'
    VICTORY_LEVELS_ON_DEMAND = '1'
    VICTORY_PIPELINE_BOOK = '1'
}
if ($Gpu -eq 'amd') {
    # Do not set VICTORY_CARD_BUFFERS_MIN_KB. On a 16 GB 9070 XT that RTX knob
    # puts every small buffer on the card. The runtime sizes that from VRAM.
    $settings.VICTORY_HOST_RDNA = '1'
    $settings.VICTORY_HOST_RDNA4 = '1'
    # 59 is every clean-up pass except "one instruction for one value".
    # That pass writes DXIL that amdxc64.dll faults on, on the 9070 XT.
    $settings.VICTORY_SHADER_OPT = '59'
    $settings.VICTORY_HOST_WAVE = '64'
    $settings.VICTORY_HOST_MAX_WAVE = '64'
    if (-not $env:VICTORY_CACHE_DIR) { $env:VICTORY_CACHE_DIR = Join-Path $env:LOCALAPPDATA 'VIctory\cache-rdna4' }
}
if ($Safe) {
    $settings.VICTORY_UNDER_WIDE = '0'; $settings.VICTORY_LAND_ECHO = '0'
} else {
    $settings.VICTORY_THIN = '1'; $settings.VICTORY_JOIN_PARTS = '8'; $settings.VICTORY_UPLOAD_AHEAD = '1'
    $settings.VICTORY_LATE_WAIT_LISTS = '192'; $settings.VICTORY_BURY_LAG = '8'
    $settings.VICTORY_MEMORY_CARD_MB = '1536'; $settings.VICTORY_PIPELINE_WORKERS = '4'
    $settings.VICTORY_PRELOAD_JOBS = '4'
}
if ($RayTracing) { $settings.VICTORY_BVH_BUILDER = '1' }
foreach ($k in $More.Keys) { $settings[$k] = $More[$k] }

$victory = Join-Path $PSScriptRoot "..\target\$Build\victory.exe"
if (-not (Test-Path $victory)) { Write-Host "No $victory. Build it first (README.md)."; return }
if (-not (Test-Path $Exe)) { Write-Host "No $Exe."; return }
# (Other settings left over in this window go; where your files are stays.)
$paths = 'VICTORY_CONFIG', 'VICTORY_GAME_DIR', 'VICTORY_CACHE_DIR', 'VICTORY_DATA_DIR'
Get-ChildItem Env:VICTORY_* | Where-Object { $_.Name -notin $paths } | ForEach-Object { Remove-Item "Env:$($_.Name)" }
foreach ($k in $settings.Keys) { Set-Item "Env:$k" $settings[$k] }
$Exe = (Resolve-Path $Exe).Path
Push-Location (Join-Path $PSScriptRoot '..')   # victory.toml is looked for here
try { & $victory run $Exe --console $Console --execute } finally { Pop-Location }
