# RustDesk 开发环境一键配置（Windows x64，2026-02-05 实测可用）
# 用法: pwsh -File scripts\dev-env-cn.ps1
# 依赖: Flutter 3.24.5(C:\flutter-sdk\flutter)、vcpkg(C:\vcpkg)、libclang(C:\tools\llvm)

$ErrorActionPreference = 'Stop'

# --- 1. 代理（本机源地址直连不通，googlesource / GitHub git 依赖都依赖它）---
#    系统代理已开启；git 与 curl/vcpkg 不自动读取系统代理，需显式配置。
$proxy = 'http://127.0.0.1:10808'
$env:HTTP_PROXY  = $proxy
$env:HTTPS_PROXY = $proxy
git config --global http.proxy  $proxy
git config --global https.proxy $proxy

# --- 2. Flutter / Dart / pub：官方源不可达，走 flutter-io.cn 镜像 ---
$env:FLUTTER_STORAGE_BASE_URL = 'https://storage.flutter-io.cn'
$env:PUB_HOSTED_URL           = 'https://pub.flutter-io.cn'
$flutterBin = 'C:\flutter-sdk\flutter\bin'
if (Test-Path $flutterBin) { $env:PATH = "$flutterBin;$env:PATH" }

# --- 3. 原生依赖（已装真实静态库：aom 3.14.1 / opus 1.6.1 / libvpx 1.15.2 / libyuv）---
$env:VCPKG_ROOT                 = 'C:\vcpkg'
$env:VCPKG_DEFAULT_TRIPLET      = 'x64-windows-static'
$env:VCPKG_DEFAULT_HOST_TRIPLET = 'x64-windows-static'

# --- 4. libclang：bindgen 找 <dir>\libclang.dll；ffigen 只认 C:\Program Files\llvm\bin\libclang.dll ---
$env:LIBCLANG_PATH = 'C:\tools\llvm'

Write-Host '--- 环境就绪 ---'
Write-Host "proxy   : $proxy (git + HTTP(S)_PROXY)"
Write-Host "Flutter : $flutterBin"
Write-Host "vcpkg   : $env:VCPKG_ROOT ($env:VCPKG_DEFAULT_TRIPLET)"
Write-Host "libclang: $env:LIBCLANG_PATH"
Write-Host ''
Write-Host '验证命令：'
Write-Host '  cargo check --all-targets                                   # Rust 类型检查'
Write-Host '  cargo test --lib                                            # 单元测试（实测 278 passed）'
Write-Host '  flutter_rust_bridge_codegen --rust-input ./src/flutter_ffi.rs --dart-output ./flutter/lib/generated_bridge.dart --c-output ./flutter/macos/Runner/bridge_generated.h'
Write-Host '  cargo check --features flutter --all-targets                # FFI 路径'
Write-Host '  cd flutter; flutter analyze                                 # Dart 分析（基线 0 error / 3 warning）'
# --- 5. 打包（本机已验证：python build.py --portable --flutter --skip-portable-pack）---
# 产物: flutter\build\windows\x64\runner\Release\  (rustdesk.exe + librustdesk.dll + Flutter 运行时)
#
# 本机特有补丁（Flutter 3.24.5 不认识 Visual Studio 2026 / v18）：
#   文件 C:\flutter-sdk\flutter\packages\flutter_tools\lib\src\windows\visual_studio.dart
#   在 cmakeGenerator 的 switch 中加一行:  18 => 'Visual Studio 18 2026',
#   然后删除 bin\cache\flutter_tools.stamp 与 flutter_tools.snapshot 强制重建快照，
#   并删除 flutter\build\windows 下残留的 CMakeCache（否则报 generator 不匹配）。
