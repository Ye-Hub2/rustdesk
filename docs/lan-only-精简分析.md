# 仅局域网 IP 直连场景下的模块裁剪分析

目标：只保留「输入对方局域网 IP → TCP 直连 → 远程桌面」这一条链路，其余模块不参与编译。
托盘按需求保留。

## 一、这条链路到底用到什么

直连在源码里是两条独立入口，与被控端监听端口构成闭环：

| 环节 | 位置 | 说明 |
| --- | --- | --- |
| 主控端判定 IP | `src/client.rs:438` | `if hbb_common::is_ip_str(peer)` → `connect_tcp_local(check_port(peer, RELAY_PORT + 1), ...)`，即连 21117+1 = 21118 |
| 主控端判定 域名:端口 | `src/client.rs:453` | `is_domain_port_str` 走同一分支 |
| 端口常量 | `libs/hbb_common/src/config.rs:119-121` | `RENDEZVOUS_PORT=21116`、`RELAY_PORT=21117`、`WS_RENDEZVOUS_PORT=21118` |
| 直连判定封装 | `src/common.rs:3065` | `is_direct_ip_access(peer)`，全仓仅 4 个调用点：`src/client/io_loop.rs:241`、`src/port_forward.rs:182`、`src/port_forward.rs:429`、`src/server/connection.rs:2998` |
| 被控端监听 | `src/direct_server.rs:22` | `direct_server()`（本次从 `rendezvous_mediator.rs` 抽出），`listen_any(get_direct_port())`，accept 后交给 `crate::server::create_tcp_connection(...)` |
| 监听端口取值 | `src/direct_server.rs:10` | 读 option `direct-access-port`，`<=0` 时回退 `RENDEZVOUS_PORT + 2` = 21118 |
| 开关 | `libs/hbb_common/src/config.rs:2969` | `OPTION_DIRECT_SERVER`（`direct-server`）。`option2bool` 对它的语义是 `value != "N"`，即**默认开启，只有显式写 `N` 才关闭**（`config.rs:2931-2932` 有专门注释说明这一点） |

关键结论：**`direct_server()` 原本住在 `rendezvous_mediator.rs` 里**，所以这个文件不能整体删除，必须先把 `get_direct_port()` + `direct_server()`（原 `:1333-1409`）抽成独立模块，剩下的 1600 多行信令/打洞/中继才是可裁部分。

主控端一旦命中 `is_ip_str`，函数立刻 `return`，`rendezvous_server` 探测、UDP NAT 测试、IPv6 打洞、WebRTC ICE、`RequestRelay` 全部不会执行（`src/client.rs:467-597` 整段都在直连 return 之后）。

## 二、已裁定：可以整体不编译的模块（8 项，已实施）

做法是 `#[cfg_attr(feature = "lan-only", path = "lan_only/<name>.rs")]`——模块路径不变、公开符号面逐字对齐，只换实现文件。**共享文件一行没改**，所以默认构建零回归。

| # | 模块 | 原行数 | 职责 | stub 行为 | 为什么签名能 100% 保真 |
| --- | --- | --- | --- | --- | --- |
| 1 | `src/lan.rs` | 328 | UDP 广播发现、WOL 唤醒 | `start_listening()` 直接 `Ok(())`；`discover()` 直接 `Ok(())`；`send_wol()` 空 | 只有 3 个对外符号。真身 `discover` 带 `#[tokio::main]`，调用点本来就是同步形态，stub 写成同步 fn 完全等价 |
| 2 | `src/hbbs_http/`（account/downloader/record_upload/sync） | 1320 | api-server 账号登录、下载器、会话录像上传、专业版策略同步 | 全部返回 `bail!` 或空实现；`sync::signal_receiver()` 返回一个**永不发送**的 broadcast receiver，保证会话 select 循环永久 pending | 4 个子模块的 pub 面已逐符号枚举；`http_client.rs`（376 行）零 crate 内部依赖，直接 `#[path]` 复用，不复制 |
| 3 | `src/updater/` | 497 | 自动更新检查、下载、安装 | `verified_update_artifact_*` 返回 `Err`；`has_no_active_conns()` 仍真实读 `Connection::alive_conns()` | 10 个外部符号签名固定，返回值类型 `UpdateArtifactQuery`/`VerifiedUpdateArtifact` 由 update_metadata stub 提供 |
| 4 | `src/update_metadata/` | 219 | 更新包元数据签名校验 | `verify_update_metadata` / `verify_offline_update_metadata` 返回 `bail!` | 5 个 pub(crate) 项，字段 `pub`、类型 `pub(crate)`，与真身一致 |
| 5 | `src/manual_update.rs` | 196 | macOS `--update` 离线更新 | `bail!` | 仅 macOS、仅 1 个外部调用点 `core_main.rs:357` |
| 6 | `src/custom_server.rs` | 246 | 定制客户端（服务器信息编进文件名） | `get_custom_server_from_string` 返回 `bail!`，struct 保留完整字段 | 1 struct + 1 fn；`src/naming.rs` 是独立 bin、自带 `mod custom_server;`，不受本次路由影响 |
| 7 | `src/kcp_stream.rs` | 255 | KCP over UDP 打洞传输 | `KcpStream` 收成空 struct；`accept`/`connect` 返回 `Err`；`peer_silent_for` 返回 `None` | 4 个方法签名固定；直连路径下 `Option<KcpStream>` 永远是 `None` |
| 8 | `src/rendezvous_mediator.rs`（主循环部分） | 1740 → 保留 | 连接 hbbs 信令、注册公钥、UDP/TCP 打洞、ICE 路由、relay 请求 | `lan-only` 下 `start_all()` 在启动 `direct_server` 与 LAN 监听后进入 `loop { sleep(3600.).await }`，跳过整个信令主循环 | 文件保留（`NEEDS_DEPLOY`/`restart`/`PunchSlot`/`CheckIfResendPk` 仍被 `ipc.rs`、`server.rs` 引用），只 cfg 掉主循环与 `test_nat_type` |

**没有裁 `update_hash`**：`platform/windows/installer_files.rs:55`、`platform/windows/verified_update/file.rs:187`、`platform/macos/verified_dmg.rs:84`、`updater/download.rs:205` 都在用 `verify_sha256_reader`；裁它要连三平台安装器验证路径一起改，风险高于收益。

## 三、评估后不裁的模块（附实测理由）

| 模块 | 行数 | 外部 `crate::X::` 引用 | 不裁的理由 |
| --- | --- | --- | --- |
| `src/privacy_mode/` | 1479 | 7 处 | 深度嵌在 `server/video_service.rs:340` 的 capturer 构造里；`platform/windows.rs:6` 还要 `win_topmost_window::WIN_TOPMOST_INJECTED_PROCESS_EXE`（broker exe 名，:1451/1452/1529/1530/1860/3381/4153）。stub 等于重写采集状态机 |
| `src/virtual_display_manager.rs` | 844 | 5 处 | `server/connection.rs:1993/4799-4853/7104` + `display_service.rs:643/743/986/994/1019` 参与显示拓扑状态机 |
| `src/whiteboard/` | 1524 | 5 处 | `connection.rs:5129-5167/7114`、`input_service.rs:1478-1504`、`ipc.rs:518` 的 `Data::Whiteboard(CustomEvent)` 变体，属协议面 |
| `src/port_forward_mux.rs` + `src/server/port_forward_mux.rs` | 1253 + 471 | 11 处（server 侧） | `connection.rs:371` 的 `Option<PortForwardMux>` 字段贯穿登录与隧道状态机 |
| `src/port_forward.rs` | 727 | 2 处 | 只有 `ui_session_interface.rs:1984/2084` 两处，但那是客户端 UI 会话路径，改动要碰 UI 接口 |
| `src/update_hash.rs` | 88 | 4 处 | 见上，平台安装器验证依赖 |
| `src/auth_2fa.rs` + `src/trusted_devices/` | 184 + 2111 | 67 处 | 引用密度最高，保留成本远低于拆除成本 |

## 四、改动清单（本次实际落地）

新增：

```
Cargo.toml                     + lan-only = []
src/direct_server.rs           get_direct_port() + direct_server()，从 rendezvous_mediator 原样搬出
src/lan_only/lan.rs
src/lan_only/custom_server.rs
src/lan_only/kcp_stream.rs
src/lan_only/manual_update.rs
src/lan_only/update_metadata.rs
src/lan_only/updater.rs
src/lan_only/hbbs_http/mod.rs      <- #[path = "../../hbbs_http/http_client.rs"] 复用真文件
src/lan_only/hbbs_http/account.rs
src/lan_only/hbbs_http/downloader.rs
src/lan_only/hbbs_http/record_upload.rs
src/lan_only/hbbs_http/sync.rs
```

修改（4 处，全部可逆、默认构建不受影响）：

| 文件 | 改动 |
| --- | --- |
| `Cargo.toml:48-54` | `[features]` 追加 `lan-only = []` 及 5 行注释 |
| `src/lib.rs` | 8 行 `#[cfg_attr(feature = "lan-only", path = "lan_only/…")]` + 新增 `#[cfg(not(ios))] mod direct_server;` |
| `src/rendezvous_mediator.rs` | 删原 `:1333-1409`；`:296` 改为 `crate::direct_server::direct_server(...)`；`:277-278` 给 `test_nat_type` 加 `cfg(not(lan-only))`；`:311-315` 加 lan-only 的空转分支 |
| `Cargo.toml` 之外的构建文件 | 无 |

共享文件（`server/`、`client/`、`ipc.rs`、`core_main.rs`、`ui_interface.rs`、`flutter_ffi.rs`、`platform/`）**一行未改**——所有调用点继续用原来的 `crate::X::y()` 路径，由 `#[path]` 换实现。

## 五、默认构建零回归的依据

- `lan-only` 不在 `default` 里，默认构建走原 `#[path]`（即 `src/lan.rs` 等原文件），`cfg_attr` 不生效。
- `direct_server` 是新增模块，原有 `rendezvous_mediator` 行为只是搬移了函数位置，逻辑逐字未变。
- `rendezvous_mediator.rs` 的三处改动全部由 `#[cfg(feature = "lan-only")]` / `#[cfg(not(feature = "lan-only"))]` 包裹。

## 六、仍需在具备工具链的机器上验证

本机无 Rust 工具链（`cargo`/`rustc`/`rustup`/VS 均不存在），全部结论来自静态引用面核对（正则全仓扫描 + 逐文件读 pub 面），**未经编译器验证**。

```bash
# 1. 默认构建必须完全不受影响（零回归）
cargo check

# 2. lan-only 本身
cargo check --no-default-features --features lan-only

# 3. lan-only + Flutter（account 模块此时才编入）
cargo check --features lan-only,flutter

# 4. 测试面
cargo test --lib --no-default-features --features lan-only
```

三个平台各跑一次（Windows / Linux / macOS），重点看 `src/platform/` 下的 cfg 分支与 `#[cfg(target_os = "macos")]` 的 `manual_update` / `update_metadata::offline`。

## 七、第二阶段（本次不做）

`Cargo.toml` 依赖可在 `lan-only` 下用 `optional` 收窄：`stunclient`(:89)、`kcp-sys`(:90)、`wol-rs`(:77)、`default-net`(:76)、`cidr-utils`(:85)、`totp-rs`(:88)、`sciter-rs`(:100)、`virtual_display`(:151)、`remote_printer`(:152)、`portable-pty`(:108)。
其中 `kcp-sys` 可随 `kcp_stream` stub 一起变 optional，`wol-rs`/`default-net` 可随 `lan` stub 一起变 optional——但这两处都要改 `Cargo.toml` 的 `optional = true` 与各 feature 的 `dep:` 引用，属于独立的依赖收窄改动。

`hbb_common` 是 git submodule（`Cargo.toml:55` 带 `features = ["webrtc"]`），改动需要 round-trip，本次不动。
