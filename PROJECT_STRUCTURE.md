# RustDesk 项目文件结构文档 (PROJECT_STRUCTURE.md)

> **本文件是本项目唯一的"代码定位索引"，供 AI/开发者每次改动前先读、改动后即改。**
> 目标：任意功能需求，都能在 1~2 分钟内定位到"该改哪些文件"。

---

## 0. 维护规则（AI 必读）

1. **先读后改**：接到任何需求，先读本文件第 7 节"功能模块索引"，再动手。
2. **改完即更新**：任何一次代码变动（增删文件、改模块职责、改入口函数）都必须同步更新：
   - 对应目录/文件表格中的条目；
   - 第 7 节的模块→文件映射；
   - 第 9 节"变更记录"追加一行（日期 / 变更摘要 / 涉及文件）。
3. **删除文件时**：从本文件中删除对应行，并在第 9 节记录"已移除模块"，避免 AI 再次搜索不存在的文件。
4. **验证命令**（见第 1 节）在提交前必须执行，结果写入第 9 节。
5. 本文件使用中文描述 + 英文标识符；路径一律使用仓库相对路径。

---

## 1. 构建与验证（本机实测可用）

### 1.1 本机开发环境（2026-02-05 实测搭好，**Rust + Dart 全链路可用**）

| 组件 | 位置 / 版本 | 安装来源 |
| --- | --- | --- |
| 代理 | `127.0.0.1:10808`（HTTP/SOCKS5 混合口） | **必需**：源地址直连不通，googlesource 与 GitHub git 依赖都靠它 |
| Flutter SDK | `C:\flutter-sdk\flutter` — **3.24.5 stable / Dart 3.5.4** | `storage.flutter-io.cn`（官方 `storage.googleapis.com` 直连超时） |
| pub 包 | — | `PUB_HOSTED_URL=https://pub.flutter-io.cn` |
| vcpkg | `C:\vcpkg`（`VCPKG_ROOT`），triplet `x64-windows-static` | 工具链走 `gh-proxy.com` 预置，端口源码走代理 |
| 原生静态库 | **aom 3.14.1 / opus 1.6.1 / libvpx 1.15.2 / libyuv**（真实 .lib，均已装） | `vcpkg install opus libvpx libyuv aom --triplet x64-windows-static` |
| cmake / nasm | 4.4.3（pip） / 2.16.03（`C:\tools\nasm`） | 清华 PyPI 镜像 / nasm.us |
| libclang | `C:\tools\llvm\{libclang.dll,bin\libclang.dll}` | bindgen 找前者；**ffigen 只认 `C:\Program Files\llvm\bin\libclang.dll`** |

一键注入（新会话先执行）：`pwsh -File scripts\dev-env-cn.ps1`

**踩过的坑（重要）**

1. **git 与 curl/vcpkg 不读系统代理**：系统代理开着也必须显式配 `git config --global http.proxy http://127.0.0.1:10808` 与 `HTTP(S)_PROXY`，
   否则 GitHub git 依赖（nokhwa/rdev/kcp-sys）与 `*.googlesource.com` 源码全部拉不下来。
2. vcpkg 在仓库根目录会因 `vcpkg.json` 进入 manifest 模式而拒绝包名参数，**必须在 `C:\vcpkg` 下执行**。
3. vcpkg 的工具链（powershell-core / 7zip / 7zr）下载走 GitHub Releases；若代理不可用，可预置到 `C:\vcpkg\downloads\<hash>-<name>`。
4. aom/libvpx 用仓库定制端口（`--overlay-ports=H:\rustdesk\res\vcpkg`）才能装上项目 pin 的版本：
   aom 的定制端口会取 3.14.1（与 `res/vcpkg/aom/portfile.cmake` 的默认分支一致）。
5. **打包时 Flutter 3.24.5 不认识 VS 2026（v18）**：它的 `cmakeGenerator` 只映射 17，v18 落入 `default => 'Visual Studio 16 2019'`，
   CMake 随即报 "could not find any instance of Visual Studio"。本机补丁（不在仓库内）：
   `visual_studio.dart` 加 `18 => 'Visual Studio 18 2026',`，再删 `bin/cache/flutter_tools.{stamp,snapshot}` 强制重建快照
   （Flutter 按 SDK revision 缓存工具快照，改源码不会自动重编），并清掉 `flutter/build/windows` 里残留的 CMakeCache。
6. 打包命令：`python build.py --portable --flutter --skip-portable-pack`（CI 还带 `--hwcodec`，本机无 ffmpeg/mfx 故省去）；
   产物 `flutter/build/windows/x64/runner/Release/`，实测 `rustdesk.exe --version` 输出 1.5.0 并可正常启动。

**验证命令（三种粒度，交付前按需执行）**

```powershell
pwsh -File scripts\dev-env-cn.ps1                    # 注入代理 + 镜像环境变量

# ① Rust 类型检查（最快）
cargo check --all-targets --message-format short                    # 实测 exit 0
# ② 单元测试（真实链接 + 运行）
cargo test --lib                                                    # 实测 278 passed / 0 failed / 3 ignored
cargo test --no-run                                                 # 全目标可链接，产出 4 个测试可执行文件
cargo build --bins                                                  # 实测产出 rustdesk.exe(69MB)/service.exe/naming.exe
# ③ FFI/Flutter 路径：先重新生成桥接，再检查
flutter_rust_bridge_codegen --rust-input ./src/flutter_ffi.rs --dart-output ./flutter/lib/generated_bridge.dart --c-output ./flutter/macos/Runner/bridge_generated.h
cargo check --features flutter --all-targets --message-format short  # 实测 exit 0
# ④ Dart 静态分析（需 ③ 生成的 generated_bridge.dart，否则会报 11 个 uri_does_not_exist）
cd flutter; flutter analyze
#    当前基线：0 error / 3 warning（3 个 warning 均为 file_manager_page.dart 既有问题，与本次改造无关）

# 注意：不要在 vcpkg 编译原生库的同时跑 cargo check/test —— 已观察到偶发假失败，串行执行即可。
```

### 1.2 历史兜底：桩依赖类型检查（**已不再需要**，1.1 的真实环境已完整）

> 真实原生库装齐后本节仅作灾备参考；原先的 `%TEMP%\vcpkg-stub` 已在 2026-02-05 清理。
> 需要时按下述结构重建即可（`cargo check` 不链接，只需头文件）：

若只想做 Rust 类型检查、不装 Flutter/vcpkg，可用"桩依赖"（仅需头文件，因为 `cargo check` 不链接）：

```powershell
# 仅需一次：桩 vcpkg（提供 opus/vpx/aom/libyuv 头文件给 bindgen）
$env:LIBCLANG_PATH = 'C:\Program Files\Python313\Lib\site-packages\clang\native'
$env:VCPKG_ROOT   = "$env:TEMP\vcpkg-stub"
cargo check --message-format short     # 全量类型检查（不含 flutter feature）
```

* 桩目录结构：`$VCPKG_ROOT\installed\x64-windows-static\include\{opus,vpx,aom,libyuv}\*.h`
  （内容为真实第三方头文件；`cargo check` 不做链接，因此无需 .lib）。
* **FFI 层验证**（`src/flutter_ffi.rs` 受 `feature = "flutter"` 门控）：先生成桥接文件，再带 feature 检查——
  改过 FFI 函数后**必须**重新生成，否则 `bridge_generated.rs` 会引用已删除的函数：
  ```powershell
  flutter_rust_bridge_codegen --rust-input ./src/flutter_ffi.rs --dart-output ./flutter/lib/generated_bridge.dart --c-output ./flutter/macos/Runner/bridge_generated.h
  cargo check --features flutter --message-format short
  ```
* 交付前两条命令都要跑：默认 feature 覆盖 Sciter/桌面路径，`flutter` feature 覆盖 FFI/Flutter 路径。
* 生成物均在 .gitignore 中：`src/bridge_generated.rs`、`src/bridge_generated.io.rs`、
  `flutter/lib/generated_bridge.dart`、`flutter/**/Runner/bridge_generated.h`。
* 完整打包（需要 Flutter/vcpkg）：`python build.py`。

---

## 2. 顶层目录总览

| 路径 | 职责 |
| --- | --- |
| `Cargo.toml` / `Cargo.lock` | 根 crate `rustdesk`（lib: `librustdesk`，bin: `rustdesk`/`naming`/`service`） |
| `build.rs` | 构建脚本：版本生成、Windows 清单、Android 依赖、macOS 打包 |
| `build.py` | 打包/发布主脚本（Flutter + Rust + 各平台产物） |
| `src/` | Rust 主程序（客户端 + 被控端 + UI 桥接），见第 3 节 |
| `libs/` | Rust 子 crate（workspace 成员），见第 4 节 |
| `flutter/` | 当前 UI（Flutter + 各平台壳工程），见第 5 节 |
| `res/` | 图标、安装包模板、vcpkg 端口补丁、CI 辅助脚本 |
| `docs/` | 多语言 README / 行为准则 / 安全策略等文档 |
| `.github/workflows/` | CI：`ci.yml`、`flutter-build.yml`、`bridge.yml` 等（**`flutter-nightly.yml` 已移除**：每日构建产出的 nightly 制品只服务于已删除的公网自动更新通道） |
| `examples/ipc.rs` | IPC 协议示例 |
| `vcpkg.json` | Windows/macOS 原生依赖清单（aom/libvpx/libyuv/opus/ffmpeg…） |
| `AGENTS.md` / `CLAUDE.md` | 编码规范（Rust/Tokio/本地化/最小改动原则） |

---

## 3. Rust 主 crate：`src/`

### 3.1 入口与生命周期

| 文件 | 职责 / 关键符号 |
| --- | --- |
| `src/lib.rs` | **模块总表**：所有 `mod` 声明与 cfg 门控都在这里 |
| `src/main.rs` | 桌面/移动入口 `main()`；非 Flutter 路径走 `core_main::core_main()` + `ui::start()` |
| `src/core_main.rs` | 启动流程编排（单实例、托盘、服务、IPC、UI 选择） |
| `src/service.rs` | macOS 服务入口（`--write-plists` / `start_os_service()`） |
| `src/naming.rs` | `naming` 小工具：生成/解析 "custom server" 可执行文件名（`custom_server`） |
| `src/version.rs` | **生成文件**（build.rs 产出，勿手改） |
| `src/lang.rs` + `src/lang/*.rs` | 本地化：`template.rs` 是 key 总表；`en.rs` 只放与 key 不同的英文 |

### 3.2 控制端（主控）—— 连接远端的一方

| 文件 | 职责 / 关键符号 |
| --- | --- |
| `src/client.rs` (~4168 行) | **控制端核心**：`Client::start/_start`（**仅直连 IP/域名**）、`LoginConfigHandler`、`AudioHandler`、`VideoHandler`、`ClientClipboardHandler`；信令/中继/WebRTC 协商路径已移除 |
| `src/client/io_loop.rs` (~131KB) | 控制端消息循环（`io_loop`），处理对端发来的各类消息 |
| `src/client/file_trait.rs` | 文件传输抽象（本地/远端读写） |
| `src/client/audio_playback*.rs` | 音频播放、恢复、启动、测试 |
| `src/client/screenshot.rs` | 会话截图 |
| `src/client/helper.rs` | 控制端小工具函数 |
| `src/client/tests/*` | 音频等回归测试 |

### 3.3 被控端（服务端进程内）

| 文件 | 职责 |
| --- | --- |
| `src/server.rs` | 被控端对外导出（`pub use self::server::*`） |
| `src/server/connection.rs` (~344KB) | **被控端核心**：一条连接的完整生命周期、登录校验、各类服务分发 |
| `src/server/video_service.rs` / `display_service.rs` / `drm_capturer.rs` | 屏幕采集与编码、显示器管理、Linux DRM 采集 |
| `src/server/input_service.rs` / `uinput.rs` / `rdp_input.rs` | 输入注入（跨平台 / Linux uinput / RDP） |
| `src/server/audio_service*` | 音频采集与编码 |
| `src/server/clipboard_service.rs` / `printer_service.rs` | 剪贴板、远程打印 |
| `src/server/terminal_service.rs` / `terminal_helper.rs` | 远程终端 |
| `src/server/portable_service.rs` / `service.rs` | 便携模式 / 服务安装 |
| `src/server/video_qos*` | 视频 QoS 策略与仿真测试 |
| `src/server/login_failure_check.rs` | 登录失败锁定 |

### 3.4 网络 / 信令 / 中继（★本次改造重点）

| 文件 | 职责 / 关键符号 |
| --- | --- |
| `src/direct_server.rs` | **直连（局域网/IP）监听**：`start_all()`、`direct_server()`；信令客户端已移除，这是原 mediator 唯一保留的部分 |
| `src/hbbs_http.rs` | hbbs HTTP API 模块声明（`record_upload`/`http_client`） |
| `src/hbbs_http/http_client.rs` | `reqwest` 客户端工厂（按 TLS 构造） |
| `src/hbbs_http/record_upload.rs` | 录屏文件上传 |
| `src/kcp_stream.rs` | KCP over UDP 传输（`KcpStream::connect/accept`） |
| `src/lan.rs` | **局域网**发现（UDP 广播）、WOL 唤醒（`discover`、`send_wol`） |
| `src/custom_server.rs` | 自定义服务器字符串（ID/中继/API）的编解码 |
| `src/auth_2fa.rs` | 2FA（TOTP 等） |

### 3.5 UI 层

| 文件 | 职责 |
| --- | --- |
| `src/ui.rs` + `src/ui/*.tis/.html/.css/.rs` | **旧 Sciter UI（已废弃）**：`index.tis`、`remote.tis`、`cm.tis`、`ab.tis` 等 |
| `src/flutter.rs` (~84KB) | Flutter 桌面端集成（窗口、托盘、事件循环） |
| `src/flutter_ffi.rs` (~99KB) | **Flutter ←→ Rust 唯一 FFI 面**（`main_*`、`session_*`、`peer_*`、`cm_*`）；`#[frb]` 注解生成本文档提到的 `generated_bridge.dart` |
| `src/ui_interface.rs` / `ui_cm_interface.rs` / `ui_session_interface.rs` | UI 与核心之间的接口层（旧 UI 仍复用其中不少结构） |
| `src/tray.rs` | 系统托盘 |
| `src/whiteboard/*` | 白板功能（client/server/平台实现） |

### 3.6 平台层与其他

| 路径 | 职责 |
| --- | --- |
| `src/platform/mod.rs` | 平台抽象入口（`start_os_service` 等） |
| `src/platform/windows.rs` / `windows.cc` / `windows/*` | Windows 实现（权限、安装、ACL、注册表） |
| `src/platform/linux.rs` / `gtk_sudo.rs` | Linux 实现（sudo/提权、GTK） |
| `src/platform/macos.rs` / `macos.mm` / `privileges_scripts/*` | macOS 实现（plist/提权脚本） |
| `src/platform/delegate.rs` / `win_device.rs` | 平台委托对象、Windows 设备信息 |
| `src/privacy_mode.rs` + `src/privacy_mode/*` | 隐私模式（黑屏/阻止输入/虚拟显示） |
| `src/virtual_display_manager.rs` | 虚拟显示器管理（Windows） |
| `src/port_forward.rs` / `port_forward_mux.rs` | 端口转发与多路复用 |
| `src/ipc.rs` + `src/ipc/{auth,drm,fs}.rs` | 进程间通信（IPC 服务器/客户端、认证、DRM、文件系统） |
| `src/clipboard.rs` / `clipboard_file.rs` | 剪贴板与剪贴板文件传输 |
| `src/keyboard.rs` / `audio_resampler*` | 键盘映射、音频重采样 |

---

## 4. Rust 子 crate：`libs/`

| crate | 路径 | 职责 / 关键文件 |
| --- | --- | --- |
| `hbb_common` | `libs/hbb_common/` | 与服务端共享的底层库。`src/config.rs` (~131KB) 是 **Config 与全部选项读写**；`protos/rendezvous.proto` **信令协议**；`tcp.rs`/`udp.rs`/`stream.rs`/`tls.rs`/`websocket.rs`/`webrtc.rs`/`socket_client.rs` 传输层；`bytes_codec.rs`；`password_security.rs`；`fingerprint.rs`；`verifier.rs`；`log_throttle.rs`；`lib.rs`（工具 + `is_ip_str` 等） |
| `base` | `libs/base/` | **仅客户端**代码：`src/config/keys.rs` **全部选项 key 的唯一导入路径**；`protos/message.proto` 业务协议；`src/fs.rs` 文件传输；`src/platform/*`；`src/keyboard.rs` |
| `scrap` | `libs/scrap/` | 屏幕捕获 + 编解码（dxgi/quartz/x11/wayland/android、vpx/aom/hwcodec/vram/mediacodec）；`build.rs` 用 bindgen 生成 `{vpx,aom,yuv}_ffi.rs` |
| `enigo` | `libs/enigo/` | 跨平台输入模拟 |
| `clipboard` | `libs/clipboard/` | 剪贴板（Windows C / Linux fuse / macOS） |
| `virtual_display` | `libs/virtual_display/` | 虚拟显示器驱动（Windows IDD） |
| `remote_printer` | `libs/remote_printer/` | 远程打印机驱动安装 |
| `portable` | `libs/portable/` | 便携版启动器（把 exe 写入临时目录再运行） |
| `libxdo-sys-stub` | `libs/libxdo-sys-stub/` | Linux 下 libxdo 的替代实现（避免 xdotool 依赖） |

> 规则（AGENTS.md）：`hbb_common` 与"服务端"共享，改动代价高；**客户端专属代码放 `libs/base`**。

---

## 5. Flutter 前端：`flutter/`

| 路径 | 职责 |
| --- | --- |
| `lib/main.dart` | 应用入口（多窗口、路由、主题、启动各页面） |
| `lib/common.dart` (~138KB) | **共享层核心**：`gFFI`（FFI 单例入口，~1596 行）、通用工具与公共对话框 |
| `lib/consts.dart` | 全局常量 |
| `lib/common/` | `shared_state.dart` 全局状态；`widgets/` 公共组件（`peer_card.dart` 设备卡片、`address_book.dart` 地址簿、`dialog.dart`、`toolbar.dart`、`peer_tab_page.dart` 等）；`hbbs/hbbs.dart` hbbs 策略；`formatter/id_formatter.dart` ID 格式化 |
| `lib/desktop/` | 桌面端页面：`pages/desktop_home_page.dart`（"Your Desktop" ID 卡片）、`server_page.dart`（本机被控/设置总览）、`desktop_setting_page.dart`（**设置页：General/Safety/Network/Display/Account(云账号)/About(更新)**）、`remote_page.dart`、`file_manager_page.dart`、`terminal_page.dart`、`port_forward_page.dart`、`install_page.dart`；`widgets/`（`remote_toolbar.dart`、`tabbar_widget.dart`、`material_mod_popup_menu.dart`） |
| `lib/mobile/` | 移动端页面（home/connection/remote/file_manager/settings/scan…) |
| `lib/models/` | 状态与业务模型：`model.dart` (~154KB FFI 封装总入口)、`ab_model.dart`（地址簿）、`peer_model.dart`、`server_model.dart`、`file_model.dart`、`input_model.dart`、`terminal_*.dart`、`web_model.dart` |
| `lib/utils/` | `http_service.dart`、`multi_window_manager.dart`、`platform_channel.dart`、`event_loop.dart` 等 |
| `lib/web/` | Web 版桥接（`bridge.dart`） |
| `lib/native/` | 平台原生小工具（win32、custom_cursor） |
| `test/` | Dart 单元测试（`flutter test`） |
| `assets/` | SVG 图标/字体/图片（`scam.png` 等） |
| `android/`、`ios/`、`windows/`、`linux/`、`macos/` | 各平台壳工程（Java/Kotlin、Swift、C++） |
| `build_*.sh`、`ndk_*.sh`、`run.sh` | 各平台构建脚本 |

---

## 6. 运行期数据与配置

| 项 | 位置 / 说明 |
| --- | --- |
| 配置目录 | 由 `hbb_common::config::Config::file()` 决定（Windows: `%APPDATA%\RustDesk\config`，Linux: `~/.config/rustdesk`，macOS: `~/Library/Preferences/com.carriez.RustDesk`） |
| 选项存储 | `config/<n>2.toml`（`Config2`，键见 `libs/base/src/config/keys.rs`） |
| **会话默认选项**（设置页"Other Default Options"的勾选框） | `config/_default.toml`（`hbb_common::config::UserDefaultConfig`）。**默认值表在 `UserDefaultConfig::get()` 的 match 里**——未设置时返回 default，显式关闭由 UI 存 `""`/`"N"`。当前默认开启：`disable_audio`（静音）、`show_monitors_toolbar`（工具栏显示监视器）、`enable-file-copy-paste`。会话内是否静音由 `DisableAudio::default_disable_audio()` 读同一处决定，改默认值只需改这一个 match |
| 设备(peer)记录 | 配置目录下 `peers/`，由 `Config::peers()` 读取 |
| 默认服务器 | `hbb_common::config`：`RENDEZVOUS_SERVERS = ["rs-ny.rustdesk.com"]`、`RENDEZVOUS_PORT = 21116`、`RELAY_PORT = 21117`、`WS_RENDEZVOUS_PORT = 21118`、`WS_RELAY_PORT = 21119`、`RS_PUB_KEY` |
| 日志 | `hbb_common::init_log()`（默认写配置目录 `log/`） |

---

## 7. 功能模块索引

### 7.1 本次改造目标模块（6 项，全部已完成）

| 模块 | Rust 侧 | Flutter 侧 | 配置/协议 |
| --- | --- | --- | --- |
| ~~**公网设备 ID 系统**~~ | ✅ 已移除（2026-02-05），详见第 10.4 节 | — | — |
| ~~**Rendezvous 信令服务器**~~ | ✅ 已移除（2026-02-05），详见第 10.4 节 | — | — |
| ~~**Relay 中继服务器**~~ | ✅ 已移除（2026-02-05），详见第 10.4 节 | — | — |
| ~~**云账号体系**~~ | ✅ 已移除（2026-02-05），详见第 10.3 节 | — | — |
| ~~**代理设置**~~ | ✅ 已移除（2026-02-05），详见第 10 节 | — | — |
| ~~**公网自动更新通道**~~ | ✅ 已移除（2026-02-05），详见第 10.2 节 | — | — |

### 7.2 其他主要功能（定位用）

| 功能 | 入口文件 |
| --- | --- |
| 屏幕采集/编码 | `libs/scrap/src/common/{codec,convert,vpxcodec,hwcodec,vram}.rs`、平台子目录 |
| 输入控制 | `libs/enigo/src/*`、`src/server/input_service.rs` |
| 剪贴板同步 | `libs/clipboard/*`、`src/clipboard.rs`、`src/server/clipboard_service.rs` |
| 文件传输 | `libs/base/src/fs.rs`、`src/client/file_trait.rs`、`src/ui_cm_interface.rs`、`flutter/lib/models/file_model.dart`。**吞吐关键：三条发送路径都由定时器 tick 驱动，每条都必须按 `fs::BLOCKS_PER_TICK`(32) 批量发送**：① 控制端上传 `client/io_loop.rs:312`（tick=MILLI1）；② 被控端下载 `server/connection.rs:1046`；③ **Windows 被控端下载实际走 CM 进程**（上游 `969ea28d0`, 2025-12-28 把 `--server` 读文件委托给 CM）：`ui_cm_interface.rs` 的 `handle_read_jobs_tick`（tick=**MILLI5**）→ IPC `FileBlockFromCM` → 服务端转发。只改前两条会漏掉下载方向。块大小/压缩级别见 `fs.rs::BUF_SIZE`(128KB) 与 `hbb_common::config::COMPRESS_LEVEL`(3)。**切勿改回「每 tick 一块」**：Windows 定时器精度约 15ms（实测 1ms 间隔只触发 65 次/秒），会把吞吐压到 8-14 MB/s |
| 端口转发 | `src/port_forward.rs`、`src/port_forward_mux.rs`、`flutter/lib/desktop/pages/port_forward_page.dart` |
| 远程终端 | `src/server/terminal_service.rs`、`flutter/lib/models/terminal_*.dart` |
| 白板 | `src/whiteboard/*` |
| 隐私模式 | `src/privacy_mode.rs`、`src/privacy_mode/*` |
| 虚拟显示器 | `src/virtual_display_manager.rs`、`libs/virtual_display/*` |
| 局域网发现/WOL | `src/lan.rs` |
| 托盘/单实例 | `src/tray.rs`、`src/core_main.rs` |
| IPC（本地进程通信） | `src/ipc.rs`、`src/ipc/*` |
| 2FA | `src/auth_2fa.rs`、`flutter/lib/common/widgets/custom_password.dart` |
| 地址簿 | `flutter/lib/models/ab_model.dart`、`common/widgets/address_book.dart` |
| 连接页在线状态指示（底部"就绪 / 连接中"） | Rust：`src/ui_interface.rs`（`UiStatus.status_num`，由 IPC `ipc::Data::OnlineStatus` 驱动；直连模式下恒为 1）、`src/ipc.rs`、`src/flutter_ffi.rs`（`main_get_connect_status`）；Flutter：`flutter/lib/desktop/pages/connection_page.dart`（`OnlineStatusWidget`）、`flutter/lib/models/state_model.dart`（`SvcStatus`） |

---

## 8. 关键调用链（便于理解改动影响面）

1. **启动**：`main.rs` → `core_main::core_main()` → `common::global_init()` → `ui::start()`/`flutter::run()`。
2. **发起远程连接**：Flutter `gFFI`(`common.dart`) → `src/flutter_ffi.rs` → `src/client.rs::Client::start` →（IP/域名则直连；否则 `get_rendezvous_server()` → `_start_inner` → `connect()`）→ 成功后 `io_loop`。
3. **被控**：`src/server.rs::new` ← `ipc`/直连监听 → `server/connection.rs` → video/input/audio/clipboard service。
4. **信令**：`rendezvous_mediator::RendezvousMediator::start_all()` → `register_pk/register_peer` → 接收 `RequestRelay`/`PunchHole` → `create_relay()`。
5. **UI 刷新**：Rust `flutter_ffi` 回调/事件 → Dart `models/model.dart`/`shared_state.dart` → 页面重建。

---

## 10. 已移除模块（AI 请勿再搜索这些代码）

> 每完成一个模块的移除，在此登记：删了什么、保留了什么、"替代路径"是什么。

### 10.1 代理设置（SOCKS5 / HTTP(S) Proxy）— 已移除 (2026-02-05)

| 删除内容 | 说明 |
| --- | --- |
| `libs/hbb_common/src/proxy.rs` | 整个文件（SOCKS5/HTTP 代理实现、`Proxy`/`ProxyScheme`/`ProxyError`） |
| `hbb_common/src/config.rs` | `Socks5Server` 结构体、`Config2::socks` 字段、`set_socks/get_socks/is_proxy/get_network_type`、`NetworkType` 枚举、`OPTION_PROXY_URL/USERNAME/PASSWORD` |
| `hbb_common/src/{tcp,udp,socket_client,websocket,stream}.rs` | 代理分支：`FramedStream::connect`（代理连接）、`FramedSocket::ProxySocks`/`new_proxy`、`test_if_valid_server_for_proxy_`、`WsFramedStream::new` 的 proxy 形参 |
| `libs/base/src/config/keys.rs` | `OPTION_HIDE_PROXY_SETTINGS` 及其在 KEYS 列表中的登记 |
| `src/ipc.rs` | `Data::Socks` / `Data::SocksWs` 消息与 `get_socks*/set_socks/get_proxy_status` |
| `src/ui_interface.rs` / `src/ui.rs` | `get_socks/set_socks/get_proxy_status` 与 Sciter 绑定；`test_if_valid_server` 去掉 `test_with_proxy` 形参 |
| `src/flutter_ffi.rs` | `main_set_socks` / `main_get_proxy_status` / `main_get_socks`；`main_test_if_valid_server(server)` 单参 |
| `src/common.rs` / `src/client.rs` / `src/rendezvous_mediator.rs` | 所有 `is_proxy()` / `get_socks()` 分支：直连路径成为唯一路径（`is_direct = !use_ws()`） |
| `src/hbbs_http/http_client.rs` | `configure_http_client!` 的代理构造、`get_url_for_tls`（TLS 探测 URL 直接用目标 URL） |
| `Cargo.toml` | `reqwest` 的 `socks` feature |
| Flutter | `desktop_setting_page.dart` 的 Socks5 代理对话框与入口、`mobile/pages/settings_page.dart` 入口、`utils/http_service.dart` 的 `mainGetProxyStatus` 判断、`consts.dart` 的 `kOptionHideProxySetting`、`web/bridge.dart` 对应桩函数 |

**保留（有意为之）**：`hbb_common` 仍依赖 `tokio-socks`，仅用于 `IntoTargetAddr`/`TargetAddr` 两个**类型**（传输层泛型签名与 UDP 目标地址表示），不再有任何代理行为。若要彻底移除该依赖，需要把这两个类型替换为 `ToSocketAddrs`/`SocketAddr`，属于传输层 API 重构，未在本次范围内。

**行为变化**：不再支持通过代理连接；`enableFlutterHttpOnRust` 关闭时 Dart 侧一律走 Flutter HTTP（原逻辑还要求"未设置代理"）。

**验证**：`cargo check` 与 `cargo check --features flutter` 均通过（仅既有告警）。

---

### 10.2 公网自动更新通道 — 已移除 (2026-02-05)

| 删除内容 | 说明 |
| --- | --- |
| `.github/workflows/flutter-nightly.yml` | 每日自动构建（`cron: "0 0 * * *"` → 复用 `flutter-build.yml`，`upload-tag: nightly`）。客户端更新通道删除后，nightly 制品已无消费方，故停止并移除该 workflow（`*.yml` 其余发布流程保留，仍可手动触发） |
| `src/updater.rs` | 整个文件：自动检查（`start_auto_update`/`check_update`）、下载、macOS root 静默升级（`check_update_as_root`）、Windows MSI 触发、`has_no_active_conns*` |
| `src/hbbs_http/downloader.rs` | 整个文件（带进度的更新包下载器）+ `hbbs_http.rs` 的模块声明 |
| `src/common.rs` | `SOFTWARE_UPDATE_URL` 静态量、`check_software_update()`、`do_check_software_update()`（原本 POST `api.rustdesk.com/version/latest`） |
| `hbb_common/src/lib.rs` | `version_check_request()`、`VER_TYPE_RUSTDESK_CLIENT`（`VER_TYPE_RUSTDESK_SERVER` 保留，服务端共享） |
| `src/ui_interface.rs` / `src/ui.rs` | `get_new_version()`、`get_software_update_url`、`get_software_store_path` 及 Sciter 绑定 |
| `src/ui/index.tis` | `UpdateMe` 组件、`software_update_url` 轮询（新版本提示卡片） |
| `src/flutter_ffi.rs` | `main_get_software_update_url`、`main_get_new_version`、`main_get_common` 的 `download-data-`/`download-file-` 分支、`main_set_common` 的 `download-new-version`/`update-me`/`extract-update-dmg`/`remove-downloader`/`cancel-downloader` |
| `src/ipc.rs` | `Data::HasNoActiveConns`、`Data::ControllingSessionCount` 及其处理函数（仅更新流程使用） |
| `src/flutter.rs` | `update_session_count_to_server()` 及其 2 处调用 |
| `src/rendezvous_mediator.rs` | Windows 上 `crate::updater::start_auto_update()` 调用 |
| `src/platform/macos.rs` | `start_auto_update_macos()` 调用、`update_from_dmg_as_root()`(~700 行)、`backup_update_plist()`、`validate_update_tree()` |
| `libs/base/src/config/keys.rs` | `OPTION_ENABLE_CHECK_UPDATE`、`OPTION_ALLOW_AUTO_UPDATE` |
| Flutter | 删除 `desktop/widgets/update_progress.dart`；`desktop_home_page.dart` 新版本卡片与 `handleUpdate`；`mobile/connection_page.dart` 更新按钮；`desktop_setting_page.dart`/`mobile/settings_page.dart` 的更新开关；`state_model.dart` 的 `updateUrl`；`common.dart` 的 `checkUpdate()`/`netWorkErrorWidget` 相关；`consts.dart` 两个 key + `kCheckSoftwareUpdateFinish` |

**保留（有意为之）**：`platform::update_me` / `update_from_dmg`（Windows/macOS）与 `core_main` 的 `--update` 命令行处理——它们是"安装器移交/本地文件升级"机制（安装页 "Click to upgrade" 依赖 `main_update_me`），不属于公网通道。因下载通道已删，`platform::update_to`/`macos::extract_update_dmg` 目前无调用方，可后续清理。

**验证**：`cargo check` ✅ / `cargo check --features flutter` ✅

### 10.3 云账号体系（rustdesk.com 账号 / OIDC 登录）— 已移除 (2026-02-05)

| 删除内容 | 说明 |
| --- | --- |
| `src/hbbs_http/account.rs` | 整个文件：`OidcSession`、`account_auth`、`AuthResult`、`UserInfo`/`UserPayload`、`/api/oidc/auth*` 轮询 |
| `src/hbbs_http.rs` | `pub mod account;` 声明 |
| `src/ui_interface.rs` | `account_auth`/`account_auth_cancel`/`account_auth_result`、对 `hbbs_http::account` 的引用 |
| `src/flutter_ffi.rs` | `main_account_auth`/`main_account_auth_cancel`/`main_account_auth_result`、`is_disable_account` |
| `src/ui.rs` | Sciter 的 `is_disable_account` 绑定 |
| `hbb_common/src/config.rs` | `is_disable_account()`（硬选项 `disable-account`） |
| Flutter | 删除 `common/widgets/login.dart`、`common/widgets/oidc_auth_status.dart`、`models/user_model.dart`；移除 `FFI.userModel` 字段与 `main.dart` 刷新调用；`address_book.dart`/`my_group.dart` 的"未登录/网络错误"分支；`peer_card.dart` 的"加入地址簿"菜单项；`peer_tab_page.dart`/`peer_tab_model.dart` 的登录态判断；`toolbar.dart` 的登录后"Note"菜单；桌面/移动设置页的 Account 分区；`server_page.dart`(移动) 的登录态判断；`web/bridge.dart` 的账号桩函数；`consts.dart` 的 `kWindowRefreshCurrentUser` |

**保留（有意为之）**：
* `main_get_login_device_info`（os/type/name 元数据）——被保留的 `hbbs.dart` API 层使用，与登录无关。
* `api-server` 选项、`hbbs_http/sync.rs`、`record_upload.rs`、`set/get_user_default_option`、`check_super_user_permission`——属于"自建服务器/服务端策略"面，不属于 rustdesk.com 账号本体；`rendezvous/API` 阶段一并评估。
* 行为变化：地址簿/设备组的**服务端同步**（`ab_model.pullAb`/`pushAb`、`group_model.pull`）恒为"未登录"语义（直接返回）；本地 peers/最近/收藏不受影响。

**验证**：`cargo check` ✅ / `cargo check --features flutter` ✅；Dart 侧已 grep 确认无残留 FFI 调用（`mainAccountAuth*`/`isDisableAccount`/`userModel` 等）。

### 10.4 公网设备 ID 系统 + Rendezvous 信令 + Relay 中继 — 已移除 (2026-02-05)

三者耦合，作为一次连续改造完成（分阶段删改，每步都保持可编译）。

**新增（替代结构）**

| 文件 | 说明 |
| --- | --- |
| `src/direct_server.rs` | **新模块**，从 `rendezvous_mediator.rs` 抽出的"直连监听"部分：`start_all()`（`check_zombie` + 直连监听 + 局域网广播 `lan::start_listening` + `scrap::codec::test_av1` + 停止服务/在线状态循环）、`direct_server()`、`get_direct_port()`、`DEFAULT_DIRECT_ACCESS_PORT = 21118`（保持历史端口，客户端与旧被控端仍可互连） |

**删除**

| 文件 / 位置 | 说明 |
| --- | --- |
| `src/rendezvous_mediator.rs` | **整个文件（94KB）**：`RendezvousMediator`、`start_all/start_tcp/start_udp/start`、`register_pk/register_peer`、`handle_resp`、punch hole（`handle_punch_hole/punch_udp_hole/punch_tcp_until_connected/udp_nat_listen`）、WebRTC answerer + ICE 路由、`handle_intranet`、`create_relay`、`NEEDS_DEPLOY` |
| `src/hbbs_http/sync.rs` | 整个文件：hbbs 心跳/策略同步、`is_pro()`、`register_switch_grant()`、`signal_receiver()` |
| `src/client.rs` | `_start_inner`、`connect`、`secure_connection`（不再需要信令侧密钥协商）、`request_relay`、`create_relay`、WebRTC offerer/ICE 桥、`race_transports_prefer_webrtc`、`request_allows_tcp_punch/tcp_punch_allowed`、`OffererGuard`、心跳连接 `hc_connection/hc_connection_`、`peer_online` 模块（在线状态查询）；`_start` 改为**仅直连**：`is_ip_str`/`is_domain_port_str` 直连，其它（设备 ID）直接报错 |
| `src/common.rs` | `get_rendezvous_server(+_)`、`test_rendezvous_server(+_)`、`refresh_rendezvous_server`、`test_nat_type(+_)`、`get_nat_type(+)`、`test_ipv6/test_ipv6_sync`、`CheckTestNatType`、**经 hbbs 的 TCP HTTP 代理**（`tcp_proxy_request/post_request_via_tcp_proxy*/http_request_via_tcp_proxy/with_tcp_proxy_fallback/get_tcp_proxy_addr/should_use_raw_tcp_for_api/can_fallback_to_raw_tcp` 及相关测试）——`post_request*/http_request_sync` 现在只走直连 HTTP |
| `src/server.rs` | 死代码：`accept_connection`/`accept_connection_`（仅被 punch 路径调用）、服务端中继 `create_relay_connection(+_)`；`start_all` 调用改指向 `crate::direct_server` |
| `src/server/connection.rs` | `hbbs_rx`（hbbs 下发"web 控制台关闭连接"）两处 select 分支、`is_pro()` 分支 |
| `src/ipc.rs` | `Data::TestRendezvousServer`、`Data::Deployed` 及其 IPC 函数、`RendezvousMediator::restart()`、`register_switch_grant`、`CheckTestNatType` 守卫 |
| `src/flutter.rs` | 在线状态查询线程（`TX_QUERY_ONLINES`/`handle_query_onlines`）；`async_tasks::query_onlines` 改为**空实现**（保留 FFI 形状，Dart 侧 `callback_query_onlines` 不再触发，UI 不再显示在线点） |
| `src/flutter_ffi.rs` / `src/ui_interface.rs` / `src/ui.rs` / `src/main.rs` | `test_rendezvous_server`/`test_nat_type` 调用、`NEEDS_DEPLOY`/`reset_needs_deploy_notification`、`RendezvousMediator::restart` 调用 |
| Flutter | 桌面/移动设置页的 **"ID/Relay Server"** 入口（公网服务器配置的可见入口） |

**保留（有意为之，作为后续可选清理）**

* 本地设备 ID：`Config::get_id/set_id/update_id` 与 "Your Desktop" ID 卡片——局域网发现、peer 配置与文件传输仍在使用，属于**本地身份**而非公网 ID 分配。
* `rendezvous.proto` 消息类型：**它同时是直连会话的线路协议**，不能删。
* 服务器地址选项（`custom-rendezvous-server`/`relay-server`/`api-server`）与 `Config::get_rendezvous_server()` 的读取：仍是配置键（`client.rs::create_login_msg` 用它拼 `id@server`），但不发起任何连接。
* `hbbs_http/record_upload.rs`：`ENABLE` 恒为 false（无人置位），录屏上传不可达。
* Flutter `mobile/widgets/dialog.dart` 的 `showServerSettings*` 对话框：设置页入口已移除，但扫码流程仍会调用（见 10.5）。

**验证**：`cargo check` ✅ / `cargo check --features flutter` ✅（`client.rs` 6000+ 行 → 4168 行）

### 10.5 死代码清理 (2026-02-05)

以 `cargo check --all-targets` 的 dead-code 告警为线索，**只清理"由本次移除导致"的死代码**（既有历史告警不动，遵循 AGENTS.md）。

| 删除内容 | 原因 |
| --- | --- |
| `src/hbbs_http/record_upload.rs`（整文件）+ `video_service.rs` 中的接线 | `ENABLE` 无人置位 → `is_enable()` 恒 false，录屏上传不可达（也就不会访问公网 API） |
| `src/platform/windows.rs::update_to`、`src/platform/macos.rs::update_to/extract_update_dmg` | 更新通道删除后无调用方 |
| `src/client.rs`：`test_udp_uat`、`udp_nat_connect`、`webrtc_race_tests` 测试模块、`kx_tests` 测试模块 | UDP NAT 测试/打洞与"信令签名的身份握手"（`secure_connection`）已删；**注意：kx_tests 之前会让 `cargo test` 编不过** |
| `src/common.rs`：`secure_tcp_silent`、`stun_ipv6_test`、`test_bind_ipv6`、`IPV6_ROUTE_PROBE`、`STUN_IPV6_TIMEOUT_MS`、`parse_simple_header` 及其 3 个测试 | 均为打洞/IPv6 探测或 hbbs TCP 代理服务 |
| `src/common.rs::test_http_proxy_response_to_json` 测试 | 被测函数已随 TCP 代理删除 |
| `src/hbbs_http.rs`：`HbbHttpResponse` + `parse`；`http_client.rs` 的 6 个构造器（`create_http_client`、`create_http_client_with_url(+_)`、`create_http_client_with_url_strict`、`create_http_client_async_with_url(+strict/+_)`） | 仅被 updater/account/sync/record_upload 使用，均已删除；`hbbs_http.rs` 现在只剩 `mod http_client` + `pub use create_http_client_async` |
| `src/common.rs`：`secure_tcp_silent`（信令专用静默握手包装） | 信令路径删除后无调用方；`secure_tcp`/`secure_tcp_required` 仍被直连/端口转发使用，保留 |
| `src/ipc.rs::CheckIfRestart` 的 5 个字段（`stop_service`/`rendezvous_servers`/`ws`/`disable_udp`/`api_server`） | 这些字段原本只用于"配置变化 → 重启信令连接"，信令已删 |

**仍保留的"惰性死代码"（有意为之）**

| 内容 | 保留理由 |
| --- | --- |
| `src/kcp_stream.rs` 与 `Client::start` 返回元组里的 `Option<KcpStream>` | KCP 仅用于"UDP 打洞后承载会话"。现在恒为 `None`，但 `client/io_loop.rs` 有 6 处围绕它的分支（关闭原因刷新、对端静默判定、发送超时），删类型要重排这些逻辑，属于行为敏感改动，未纳入本次 |
| `client.rs::Interface::is_policy_relay`、`get_switch_code` | 中继策略判断已无调用方；二者是 `Interface` trait 方法，删除需同步改 UI 侧实现（`ui_session_interface` 等），收益低、churn 高，未纳入本次 |
| Flutter `mobile/widgets/dialog.dart::showServerSettings*` | **不是死代码**：`mobile/pages/scan_page.dart` 的扫码流程仍在调用它（扫码写入服务器配置，选项仍存在）。只是"设置页入口"已移除 |
| 服务器地址选项（`custom-rendezvous-server`/`relay-server`/`api-server`）与 `Config::get_rendezvous_server()` | 仍是配置键（`client.rs::create_login_msg` 用它拼 `id@server`、扫码/许可证写入），但不发起任何连接；彻底删除会波及 `hbb_common`（与 rustdesk-server 共享）与 Flutter 设置/审计/在线态多处 |
| `src/auth_2fa.rs`、`src/client/file_trait.rs`、`src/clipboard.rs`、`src/ui.rs`、`src/ui_interface.rs` 等既有 `never used` 告警 | 与本次改造无关的历史遗留，按规范不在本次修 |

**验证**：`cargo check --all-targets` ✅ / `cargo check --features flutter --all-targets` ✅（含测试目标）

### 10.6 去公网化残留：在线状态语义 + "自建服务器"引导 (2026-09-28)

信令（Rendezvous）删除后，`hbb_common::config::ONLINE` 的唯一写入者 `update_latency()` 失去调用方，`get_online_state()` 恒返回 0，连接页底部状态栏因此永久停留在"正在接入 RustDesk 网络..."。改为「直连模式下服务可达即为就绪」，并清除随之首次暴露的公网引导。

| 改动 | 位置 | 说明 |
| --- | --- | --- |
| 收到 IPC `Data::OnlineStatus` 时**忽略其数值**，`status_num` 恒置 1（ready） | `src/ui_interface.rs`（`match ipc::Data` 的 `OnlineStatus` 分支，约 :1299） | 直连模式下该消息到达即代表服务可达；元组解构改为 `Some((_, _c))`（`_c` 在 `feature = "flutter"` 下不使用）。**IPC 断开仍置 -1（notReady），语义不变** |
| 删除连接页公网服务器引导（", 如果需要更快连接速度，你可以选择自建服务器" + 跳转 `https://rustdesk.com/pricing`） | `flutter/lib/desktop/pages/connection_page.dart`：删 `setupServerWidget()`、`onUsePublicServerGuide()`、`_svcIsUsingPublicServer` 字段与赋值、`url_launcher_string` import 及调用点（净 -46 行，0 新增） | 该 widget 原显示条件为 `!_svcStopped && svcStatus == ready && using_public_server()`；本分支 `PROD_RENDEZVOUS_SERVER` 恒为空 → `using_public_server()` 恒 true，此前因状态恒为 connecting 而恒 offstage；状态修复后首次可达，必须随公网一并移除。**注意：不要改 `common.rs::using_public_server()` 返回值**，它同时控制设置页 TLS 回退选项、会话 FPS 显示等多处，影响面过宽 |

**验证**：`cargo check --all-targets` ✅（EXIT=0）/ `cargo check --features flutter --all-targets` ✅（EXIT=0）/ `dart analyze lib/desktop/pages/connection_page.dart` ✅（0 error，仅 1 条既有 `deprecated_member_use` info）

## 9. 变更记录

| 日期 | 变更 | 涉及文件 | 验证 |
| --- | --- | --- | --- |
| 2026-02-05 | 建立本文件（项目结构深度分析） | `PROJECT_STRUCTURE.md` | `cargo check` 通过（基线，仅警告） |
| 2026-02-05 | **移除「代理设置」**（SOCKS5/HTTP 代理）：Rust 侧 12 个文件 + Flutter 侧 5 个文件 | 见第 10.1 节 | `cargo check` ✅ / `cargo check --features flutter` ✅ |
| 2026-02-05 | **移除「公网自动更新通道」**：updater/downloader/版本检查/更新 UI（Rust + Sciter + Flutter） | 见第 10.2 节 | `cargo check` ✅ / `cargo check --features flutter` ✅ |
| 2026-02-05 | **移除「云账号体系」**：OIDC 登录后端 + 登录/用户模型 UI（Rust + Flutter，删除 3 个 Dart 文件） | 见第 10.3 节 | `cargo check` ✅ / `cargo check --features flutter` ✅ |
| 2026-02-05 | **移除「公网设备 ID 系统 + Rendezvous 信令 + Relay 中继」**：删 2 个文件、新增 `direct_server.rs`、`client.rs` 改为仅直连 | 见第 10.4 节 | `cargo check` ✅ / `cargo check --features flutter` ✅ |
| 2026-02-05 | **剪贴板拷文件（会话内 Ctrl+C/V、拖拽）吞吐修复**：根因是 CLIPRDR 的 `IStream::Read` **每次只按调用方请求的字节数（资源管理器约 64KB）向对端发一个 `FILECONTENTS_RANGE` 并同步等待应答**，吞吐被钉在 `块大小 / 往返`（64KB ÷ ≈4.4ms ≈ 14MB/s，与现场实测一致）。改为**预读 1MB**（`WF_CLIPRDR_READ_AHEAD_BYTES`）后，多数 Read 直接命中内存缓冲，只需偶尔过一次网络。注意这与「文件传输窗口」是两套完全不同的引擎：传输窗口走 `handle_read_jobs`（已批处理，现场实测千兆），剪贴板拷文件走 CLIPRDR | `libs/clipboard/src/windows/wf_cliprdr.c`（结构体 +`m_pReadAhead` 等 4 字段、`CliprdrStream_Read` 重写、`Seek` 失效缓冲、`Delete` 释放） | `cargo build -p clipboard` 0 error；`New` 用 `calloc` 自动清零、`Clone` 为 `E_NOTIMPL`（无结构体拷贝，无双重释放） |
| 2026-02-05 | **文件传输收尾**：① 追加修复——界面速率改用文件字节口径（原用压缩后字节，压缩比越高显示越低）；② ≤4 核机器 `Auto` 编解码不再选 AV1 软编（原仅按内存 ≤4GB 降级到 VP8），避免弱机编码占满 CPU；③ 移除全部临时排查打点（`log_xfer_rate` / `xfer[...]` / `-diag1` 版本后缀）| `src/client/io_loop.rs`、`libs/scrap/src/common/codec.rs`、`libs/base/src/fs.rs`、`src/ui_cm_interface.rs`、`src/ui_interface.rs` | 双 feature check 0 error；MSI 载荷哈希 == 构建产物；最终包 MSI 1.5.0.29837553 / EXE SHA256 F4EA905E… |
| 2026-02-05 | **CM 路径排查要点（重要）**：Windows 被控端在**有会话时**由服务另起 CM 进程代读文件（`server/connection.rs:6312` Start cm），数据走 CM→IPC→服务→TCP；**仅传文件模式不走 CM**（进程内直读）。因此「会话中传输只有 14MB/s、仅传文件能跑满千兆」指向 CM 路径未生效（旧构建/复用了旧 CM 进程——`ipc::connect(1000, "_cm")` 会复用已存在的 CM）。现场证据：14.8MB/s ÷ 128KB ≈ 8.7ms/块 = 每 tick 一块的节奏 | `src/ui_cm_interface.rs`、`src/server/connection.rs` | 现场 A/B；诊断版实测 `xfer[send]` 113MB/s（仅传文件） |
| 2026-02-05 | **文件传输吞吐：现场结论**——现场 A/B 实测（两台 Win10）：「仅传文件」模式**跑满千兆**；建立远程桌面连接后掉到 14MB/s。结合被控端日志（`cpus=4/4`、`available memory: 2G`、`new encoder: AOM(AV1) 1920x1080`）判定：**传输代码已达标，剩余瓶颈是 4 核机器上 AV1 软编与传输争抢 CPU**，非传输逻辑缺陷。另：大媒体文件（mp4/iso/mkv…）此前会被白白压缩一遍，已加入跳过压缩白名单 | `libs/base/src/fs.rs`（`is_compressed_file`） | 现场 A/B：仅传文件 ≈ 千兆 ✅ |
| 2026-02-05 | **修复文件传输吞吐（第二轮）**：Windows 被控端下载经上游 `969ea28d0` 改为走 CM 进程，其 `handle_read_jobs_tick` 仍是「每 tick 一块」且 tick=5ms（实测限到 14MB/s）；改为每 tick 最多 `BLOCKS_PER_TICK` 块 | `src/ui_cm_interface.rs` | check 双 feature exit 0；MSI/EXE 载荷哈希与构建产物一致 |
| 2026-02-05 | **取消密码复杂度要求**：永久密码只要求长度 ≥ 1（原为数字+大小写+长度≥8；特殊字符规则本就被注释掉） | `flutter/lib/desktop/pages/desktop_home_page.dart`（`setPasswordDialog` 的 rules 列表） | 规则源码已确认 `MinCharactersValidationRule(1)` |
| 2026-02-05 | **修复文件传输吞吐**：发送侧由“每 tick 一块”改为“每 tick 最多 BLOCKS_PER_TICK=32 块”，解除对定时器精度的依赖 | `libs/base/src/fs.rs`（+34/-1，调用点零改动） | 实测（release、环回、1ms tick）：修复前 **8.1 MB/s** → 修复后 **369 MB/s**；check 双 feature exit 0，`cargo test --lib` 278 passed |
| 2026-02-05 | **默认会话选项改为勾选**：「静音」(`disable_audio`) 与「在工具栏上显示监视器」(`show_monitors_toolbar`)；同时移除每日自动构建 `flutter-nightly.yml` | `libs/hbb_common/src/config.rs`（`UserDefaultConfig::get` 两行）、`.github/workflows/flutter-nightly.yml` | `cargo check --all-targets` ✅ / `--features flutter` ✅ / `cargo test --lib` 278 passed ✅ |
| 2026-02-05 | **打通源地址访问**：配置 git/curl/vcpkg 走本机代理，装齐 4 个真实原生静态库（aom 3.14.1 / opus 1.6.1 / libvpx 1.15.2 / libyuv），首次跑通 `cargo test --lib`(278 passed) 与 `cargo build --bins` | 见第 1.1 节 | `cargo test --lib` ✅ `cargo build --bins` ✅ |
| 2026-02-05 | **搭建镜像源开发环境**：Flutter 3.24.5/Dart 3.5.4（flutter-io.cn）、vcpkg+cmake+nasm+libclang，新增 `scripts/dev-env-cn.ps1`；并用 `flutter analyze` 首次完成 Dart 侧验证，据此修掉 2 个枚举 switch 错误 + 一批死代码告警 | 见第 1.1 节 | `flutter analyze` = 0 error ✅ |
| 2026-02-05 | **死代码清理**：删 `record_upload.rs` 等 9 类残留（含 2 个失效测试模块、hbbs 代理/打洞残留、6 个 HTTP 构造器） | 见第 10.5 节 | `cargo check --all-targets` ✅ / `--features flutter --all-targets` ✅（测试目标可编译） |
| 2026-09-28 | **修复连接页状态栏永久"正在接入 RustDesk 网络..."**：信令删除后 `get_online_state()` 恒 0；改为直连模式下收到 IPC `OnlineStatus` 即视为 ready | `src/ui_interface.rs`（净 -1 行） | `cargo check --all-targets` ✅ / `--features flutter --all-targets` ✅（均 EXIT=0，告警回到基线） |
| 2026-09-28 | **移除连接页"自建服务器"公网引导**（状态修复后首次可达的残留 UI） | `flutter/lib/desktop/pages/connection_page.dart`（-46 行） | `dart analyze` = 0 error ✅；`cargo check` 双 feature ✅ |
| 2026-09-29 | **暂停非 Windows 包构建**（临时，待用户通知恢复）：GitHub 工作流只保留 Windows Flutter 的 exe 自解压包（`rustdesk-<VERSION>-<arch>.exe`）；MSI、Windows Sciter、macOS、iOS、Android×2、Linux×3、AppImage、Flatpak、SBOM 与 `publish_unsigned` 共 13 个 job 置 `if: false`（原条件留在行内 `# was:` 注释里），Windows job 内 3 个 MSI 步骤同样置 `if: false`，发布只列 exe | `.github/workflows/flutter-build.yml` | `yaml.safe_load` 解析通过（16 job）；启用者仅 `generate-bridge`、`build-RustDeskTempTopMostWindow`、`build-for-windows-flutter` |
| 2026-09-29 | **修复「MSI 卸载后服务/残留不清理」**：`get_uninstall()` 在注册表仍写有 `WindowsInstaller=1` 但 MSI product code 已解析不到时，原为 `bail!` 直接放弃整个卸载（服务、安装目录、注册表项全部残留，且 exe 安装又被 `windows.rs:1586-1590` 的 MSI 状态检查拒绝）；改为记 `log::warn!` 后继续执行下方脚本，脚本内的 `sc stop`/`sc delete`、防火墙规则删除与注册表清理照跑（与 exe 安装路径的清理逻辑一致） | `src/platform/windows.rs`（`get_uninstall`，+3/-1 行） | `cargo check --all-targets` ✅ EXIT=0 / `cargo check --features flutter --all-targets` ✅ EXIT=0 |
