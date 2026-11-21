@echo off
rem Builds Vellmark from this folder and runs it in the valley. Double-click it.
rem Anything after it on a command line is passed to the game instead, e.g.
rem   run-windows.bat --scene yard
setlocal
cd /d "%~dp0"

where cargo >nul 2>nul
if errorlevel 1 if exist "%USERPROFILE%\.cargo\bin\cargo.exe" set "PATH=%USERPROFILE%\.cargo\bin;%PATH%"
where cargo >nul 2>nul
if errorlevel 1 (
  echo Rust is not installed. Install it from https://rustup.rs ^(choose the default, which
  echo also installs the Visual Studio build tools it needs^), then run this again.
  goto :fail
)
where cmake >nul 2>nul
if errorlevel 1 (
  echo CMake is needed to build SDL. Install it with:
  echo   winget install Kitware.CMake
  echo then open a new window and run this again.
  goto :fail
)

rem The newest code, if this folder is a git checkout and nothing here is changed.
if exist ".git" (
  where git >nul 2>nul
  if not errorlevel 1 (
    git diff --quiet HEAD 2>nul && (
      echo Taking the newest code...
      git pull --ff-only --quiet || echo Could not take the newest code; building what is here.
    )
  )
)

rem SDL's CMake build fails in a folder whose path has a space in it: build elsewhere then.
set "HERE=%CD%"
if not "%HERE: =%"=="%HERE%" (
  set "CARGO_TARGET_DIR=%LOCALAPPDATA%\vellmark-build"
)
if defined CARGO_TARGET_DIR if not "%CARGO_TARGET_DIR: =%"=="%CARGO_TARGET_DIR%" set "CARGO_TARGET_DIR=C:\vellmark-build"
if defined CARGO_TARGET_DIR echo This folder's path has a space in it, so the build goes to %CARGO_TARGET_DIR%.

set "ARGS=%*"
if "%ARGS%"=="" set "ARGS=--scene valley"
echo Building ^(the first time takes several minutes^)...
cargo run --release -p vellmark-app --features dev -- %ARGS%
if errorlevel 1 (
  echo.
  echo The build or the game stopped with an error, shown above.
  goto :fail
)
exit /b 0

:fail
echo.
pause
exit /b 1
