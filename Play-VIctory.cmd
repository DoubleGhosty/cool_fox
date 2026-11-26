@echo off
setlocal EnableExtensions DisableDelayedExpansion
rem RX 9070 XT / other 16 GB RDNA is the default.
rem   Play-VIctory.cmd              Series S, 16 GB RDNA budgets
rem   Play-VIctory.cmd 9070-x       Series X (about 64 GB of system RAM)
rem   Play-VIctory.cmd pareto-residency
rem                                 the older RTX residency settings
set "PROFILE=%~1"
if "%PROFILE%"=="" set "PROFILE=9070"
if /i "%PROFILE%"=="amd" set "PROFILE=9070"
if /i "%PROFILE%"=="amd-x" set "PROFILE=9070-x"
if /i "%PROFILE%"=="9070" goto amd
if /i "%PROFILE%"=="9070-x" goto amdx
if /i "%PROFILE%"=="pareto-residency" goto rtx
echo Usage: Play-VIctory.cmd [9070 ^| 9070-x ^| pareto-residency] 1>&2
exit /b 2

:amd
set "CONSOLE=series-s"
goto amd_common

:amdx
set "CONSOLE=series-x"
goto amd_common

:amd_common
if not defined VICTORY_GAME_DIR set "VICTORY_GAME_DIR=%~dp0..\GTA6.Xbox.v1.0.2.1"
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
  echo Set VICTORY_GAME_DIR to your own dump folder containing game_scarlett_master.exe. 1>&2
  exit /b 2
)
rem A local build, not the RTX-named prebuilt. That binary does not have this profile.
set "runner=%~dp0target\release\victory.exe"
if not exist "%runner%" set "runner=%~dp0target\diag\victory.exe"
if not exist "%runner%" set "runner=%~dp0victory.exe"
if not exist "%runner%" (
  echo Build on this PC first: cargo build --release --locked -p victory 1>&2
  echo Do not use victory-3d-rtx-pareto-residency.exe on the 9070 XT. 1>&2
  exit /b 2
)
if not defined VICTORY_GPU set "VICTORY_GPU=amd"
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\VIctory\cache-rdna4"
if not defined VICTORY_HOST_RDNA set "VICTORY_HOST_RDNA=1"
if not defined VICTORY_HOST_RDNA4 set "VICTORY_HOST_RDNA4=1"
if not defined VICTORY_SHADER_OPT set "VICTORY_SHADER_OPT=59"
if not defined VICTORY_HOST_WAVE set "VICTORY_HOST_WAVE=64"
if not defined VICTORY_HOST_MAX_WAVE set "VICTORY_HOST_MAX_WAVE=64"
if not defined VICTORY_LEVELS_ON_DEMAND set "VICTORY_LEVELS_ON_DEMAND=1"
if not defined VICTORY_TEXTURE_NO_WAIT set "VICTORY_TEXTURE_NO_WAIT=1"
if not defined VICTORY_RELAY set "VICTORY_RELAY=2"
if not defined VICTORY_WIDE_BACK set "VICTORY_WIDE_BACK=1"
if not defined VICTORY_CARD_REHASH_RUNS set "VICTORY_CARD_REHASH_RUNS=1"
if not defined VICTORY_JOIN_COPIES set "VICTORY_JOIN_COPIES=1"
if not defined VICTORY_INDIRECT_WINDOW set "VICTORY_INDIRECT_WINDOW=1"
if not defined VICTORY_RELAY_COPY set "VICTORY_RELAY_COPY=1"
if not defined VICTORY_CARD_STEPS set "VICTORY_CARD_STEPS=2"
if not defined VICTORY_INDIRECT_BATCH set "VICTORY_INDIRECT_BATCH=2"
if not defined VICTORY_MAP_THREADS set "VICTORY_MAP_THREADS=1"
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=1"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_LATE_WAIT_LISTS set "VICTORY_LATE_WAIT_LISTS=192"
if not defined VICTORY_BURY_LAG set "VICTORY_BURY_LAG=8"
if not defined VICTORY_MEMORY_CARD_MB set "VICTORY_MEMORY_CARD_MB=1536"
if not defined VICTORY_PIPELINE_WORKERS set "VICTORY_PIPELINE_WORKERS=4"
if not defined VICTORY_PRELOAD_JOBS set "VICTORY_PRELOAD_JOBS=4"
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
if not defined VICTORY_TITLE_PRIORITIES set "VICTORY_TITLE_PRIORITIES=0"
if not defined VICTORY_PIPELINE_BOOK set "VICTORY_PIPELINE_BOOK=1"
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"
echo RDNA profile %PROFILE% on %CONSOLE%. Cache %VICTORY_CACHE_DIR%
echo Adapter filter VICTORY_GPU=%VICTORY_GPU%. Set VICTORY_D3D12_ADAPTER=9070 if another Radeon is installed.
echo Precompiling shaders, then loading host pipelines, before the title starts.
echo Set VICTORY_PRELOAD=0 to skip that. The first launch compiles everything it finds.
"%runner%" run "%VICTORY_GAME_DIR%\game_scarlett_master.exe" --console %CONSOLE% --execute
exit /b %errorlevel%

:rtx
if not defined VICTORY_GAME_DIR set "VICTORY_GAME_DIR=%~dp0..\GTA6.Xbox.v1.0.2.1"
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
  echo Set VICTORY_GAME_DIR to your own dump folder containing game_scarlett_master.exe. 1>&2
  exit /b 2
)
set "runner=%~dp0victory-3d-rtx-pareto-residency.exe"
if not exist "%runner%" set "runner=%~dp0target\release\victory.exe"
if not exist "%runner%" (
  echo Extract the Windows release or run cargo build --release --locked -p victory. 1>&2
  exit /b 2
)
if not defined VICTORY_GPU set "VICTORY_GPU=nvidia"
if not defined VICTORY_AMBIENT_AIR set "VICTORY_AMBIENT_AIR=1.5"
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\VIctory\cache-pareto"
if not defined VICTORY_TEXTURE_NO_WAIT set "VICTORY_TEXTURE_NO_WAIT=1"
if not defined VICTORY_RELAY set "VICTORY_RELAY=2"
if not defined VICTORY_WIDE_BACK set "VICTORY_WIDE_BACK=1"
if not defined VICTORY_CARD_REHASH_RUNS set "VICTORY_CARD_REHASH_RUNS=1"
if not defined VICTORY_JOIN_COPIES set "VICTORY_JOIN_COPIES=1"
if not defined VICTORY_INDIRECT_WINDOW set "VICTORY_INDIRECT_WINDOW=1"
if not defined VICTORY_RELAY_COPY set "VICTORY_RELAY_COPY=1"
if not defined VICTORY_CARD_STEPS set "VICTORY_CARD_STEPS=2"
if not defined VICTORY_INDIRECT_BATCH set "VICTORY_INDIRECT_BATCH=2"
if not defined VICTORY_MAP_THREADS set "VICTORY_MAP_THREADS=1"
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=1"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
if not defined VICTORY_LEVELS_ON_DEMAND set "VICTORY_LEVELS_ON_DEMAND=1"
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=64"
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"
"%runner%" run "%VICTORY_GAME_DIR%\game_scarlett_master.exe" --console series-s --execute
exit /b %errorlevel%
