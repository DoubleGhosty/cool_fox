@echo off
setlocal EnableExtensions DisableDelayedExpansion
if not "%~1"=="" if /i not "%~1"=="pareto-residency" (
  echo Usage: Play-VIctory.cmd pareto-residency 1>&2
  exit /b 2
)
if not defined VICTORY_GAME_DIR if exist "C:\Users\Ari Tech\Downloads\Other\GTA6.Xbox.v1.0.2.1-CYBERLEEK\game_scarlett_master.exe" set "VICTORY_GAME_DIR=C:\Users\Ari Tech\Downloads\Other\GTA6.Xbox.v1.0.2.1-CYBERLEEK"
if not defined VICTORY_GAME_DIR (
  echo Set VICTORY_GAME_DIR to the folder containing your title executable. 1>&2
  exit /b 2
)
if not exist "%VICTORY_GAME_DIR%\game_scarlett_master.exe" (
  echo game_scarlett_master.exe not found in VICTORY_GAME_DIR: "%VICTORY_GAME_DIR%" 1>&2
  exit /b 2
)
set "runner=%~dp0target\release\victory.exe"
if not exist "%runner%" set "runner=%~dp0victory-3d-rtx-pareto-residency.exe"
if not exist "%runner%" (
  echo Extract the Windows release or run cargo build --release --locked -p victory. 1>&2
  exit /b 2
)
rem Ambient lighting, crease shading & volumetric air ceiling
if not defined VICTORY_AMBIENT set "VICTORY_AMBIENT=0.8"
if not defined VICTORY_AMBIENT_AIR set "VICTORY_AMBIENT_AIR=1.0"
if not defined VICTORY_AMBIENT_SHADE set "VICTORY_AMBIENT_SHADE=1.2,0.8"
if not defined VICTORY_CACHE_DIR set "VICTORY_CACHE_DIR=%LOCALAPPDATA%\VIctory\cache-pareto"
rem Memory trimming & VRAM management (prevents 99% RAM stall & NVMe pagefile thrashing)
if not defined VICTORY_LEVELS_ON_DEMAND set "VICTORY_LEVELS_ON_DEMAND=1"
if not defined VICTORY_TEXTURE_KEEP set "VICTORY_TEXTURE_KEEP=0"
if not defined VICTORY_TEXTURE_SWEEP set "VICTORY_TEXTURE_SWEEP=whole"
if not defined VICTORY_TEXTURE_BUDGET_MB set "VICTORY_TEXTURE_BUDGET_MB=3584"
if not defined VICTORY_TEXTURE_STAGE_MB set "VICTORY_TEXTURE_STAGE_MB=64"
if not defined VICTORY_RECHECK_SOON set "VICTORY_RECHECK_SOON=0"
if not defined VICTORY_STAGING_IDLE_MS set "VICTORY_STAGING_IDLE_MS=2000"
if not defined VICTORY_WINDOW_IDLE_MS set "VICTORY_WINDOW_IDLE_MS=2000"
if not defined VICTORY_LIST_IDLE_TRIM_S set "VICTORY_LIST_IDLE_TRIM_S=10"
if not defined VICTORY_READ_KEEP_MB set "VICTORY_READ_KEEP_MB=256"
if not defined VICTORY_SCRATCH_SPARE_MB set "VICTORY_SCRATCH_SPARE_MB=128"
if not defined VICTORY_SEND_SPARE_MB set "VICTORY_SEND_SPARE_MB=128"
rem CPU multi-threaded list translation (tuned for i5-13400F 6 P-cores)
if not defined VICTORY_THIN set "VICTORY_THIN=1"
if not defined VICTORY_THIN_THREADS set "VICTORY_THIN_THREADS=6"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=8"
if not defined VICTORY_ASYNC_SUBMIT set "VICTORY_ASYNC_SUBMIT=1"
if not defined VICTORY_FAST_CORES set "VICTORY_FAST_CORES=2"
if not defined VICTORY_D3D12_BACKGROUND set "VICTORY_D3D12_BACKGROUND=0"
if not defined VICTORY_TITLE_PRIORITIES set "VICTORY_TITLE_PRIORITIES=0"
rem GPU buffer acceleration & pipelined relay
if not defined VICTORY_CARD_BUFFERS_MIN_KB set "VICTORY_CARD_BUFFERS_MIN_KB=64"
if not defined VICTORY_CARD_BUFFERS_MB set "VICTORY_CARD_BUFFERS_MB=512"
if not defined VICTORY_CARD_STEPS set "VICTORY_CARD_STEPS=2"
if not defined VICTORY_TEXTURE_NO_WAIT set "VICTORY_TEXTURE_NO_WAIT=1"
if not defined VICTORY_RELAY set "VICTORY_RELAY=2"
if not defined VICTORY_LATE_WAIT set "VICTORY_LATE_WAIT=3"
if not defined VICTORY_WIDE_BACK set "VICTORY_WIDE_BACK=1"
if not defined VICTORY_CARD_REHASH_RUNS set "VICTORY_CARD_REHASH_RUNS=1"
if not defined VICTORY_JOIN_COPIES set "VICTORY_JOIN_COPIES=1"
if not defined VICTORY_INDIRECT_WINDOW set "VICTORY_INDIRECT_WINDOW=1"
if not defined VICTORY_RELAY_COPY set "VICTORY_RELAY_COPY=1"
if not defined VICTORY_INDIRECT_BATCH set "VICTORY_INDIRECT_BATCH=2"
if not defined VICTORY_MAP_THREADS set "VICTORY_MAP_THREADS=1"
if not defined VICTORY_CARD_AWAKE set "VICTORY_CARD_AWAKE=1"
if not defined VICTORY_SHOW_AFTER set "VICTORY_SHOW_AFTER=1"
if not defined VICTORY_TEXTURE_GUARD set "VICTORY_TEXTURE_GUARD=0"
rem Renderer ports 6 & 7 (write-guard lock sharing, journal split) save 2-5 ms per frame
if not defined VICTORY_PORT_GUARD set "VICTORY_PORT_GUARD=1"
if not defined VICTORY_PORT_JOURNAL set "VICTORY_PORT_JOURNAL=1"
if not defined VICTORY_PORT_CONSTANTS set "VICTORY_PORT_CONSTANTS=0"
if not defined VICTORY_PORT_CBV set "VICTORY_PORT_CBV=0"
if not defined VICTORY_LIBRARY_STORE_LATER set "VICTORY_LIBRARY_STORE_LATER=1"
if not defined VICTORY_PIPELINE_WAIT_MS set "VICTORY_PIPELINE_WAIT_MS=100"
if not defined VICTORY_PIPELINE_BOOK set "VICTORY_PIPELINE_BOOK=1"
if not defined VICTORY_UPLOAD_AHEAD set "VICTORY_UPLOAD_AHEAD=1"
if not defined VICTORY_LOG if not defined RUST_LOG set "VICTORY_LOG=warn"
rem Series X, not S: on an 8 GB card series-s drops the head/skin meshes (faces
rem render as eyes and teeth only); series-x renders them. Verified 2026-11-26.
"%runner%" run "%VICTORY_GAME_DIR%\game_scarlett_master.exe" --console series-x --execute
exit /b %errorlevel%
