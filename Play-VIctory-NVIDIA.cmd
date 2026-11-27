@echo off
setlocal EnableExtensions DisableDelayedExpansion
rem GeForce profile. The shared launcher supplies the remaining Series X settings.
rem The menus use a 4K layout. Render the 3D scene at 1080p and fit the
rem composited frame to a 1080p window. An explicit output keeps its own scale.
if not defined VICTORY_OUTPUT (
  set "VICTORY_OUTPUT=3840x2160"
  if not defined VICTORY_RENDER_SCALE set "VICTORY_RENDER_SCALE=0.5"
  if not defined VICTORY_WINDOW_MAX set "VICTORY_WINDOW_MAX=1920x1080"
)
rem Split automatic shader mask branches per lane on GeForce hardware.
if not defined VICTORY_WAVE_SCC set "VICTORY_WAVE_SCC=0"

rem THIN can hide parts of models and JOIN_PARTS can repeat cutscene frames.
if not defined VICTORY_THIN set "VICTORY_THIN=0"
if not defined VICTORY_JOIN_PARTS set "VICTORY_JOIN_PARTS=0"
rem On some GeForce cards the fast constant-buffer paths can hide roads or crash.
if not defined VICTORY_PORT_CBV set "VICTORY_PORT_CBV=0"
if not defined VICTORY_PORT_CONSTANTS set "VICTORY_PORT_CONSTANTS=0"

call "%~dp0Play-VIctory.cmd"
exit /b %errorlevel%
