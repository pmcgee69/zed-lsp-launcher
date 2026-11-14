@echo off
rem Rebuilds extension.wasm from src\. Only for changing the launcher: extensions copy the
rem extension.wasm in this repo, and none of them builds anything.
rem   build.cmd       needs Rust (rustup.rs); adds the wasm32-wasip2 target once
rem   build.cmd test  also runs the tests of .server files' parsing
setlocal
cd /d "%~dp0"
if /i "%~1"=="test" (cargo test || exit /b 1)
rustup target add wasm32-wasip2 || exit /b 1
cargo build --release --target wasm32-wasip2 || exit /b 1
copy /y target\wasm32-wasip2\release\zed_lsp_launcher.wasm extension.wasm >nul || exit /b 1
echo built: extension.wasm
