//! Direct (LAN / IP) access listener.
//!
//! This module is what remains of the rendezvous mediator: the signaling client
//! is gone, but the TCP listener that accepts direct connections has to stay,
//! or no peer could reach this device at all. `start_all` also keeps the LAN
//! discovery broadcast and the service lifecycle loop that used to live in the
//! mediator.

use hbb_common::{
    allow_err,
    config::{option2bool, Config, RENDEZVOUS_PORT},
    log, sleep, tokio,
};
use base::config::keys::*;

use crate::server::{check_zombie, new as new_server, ConnectionMeta, ServerPtr};

/// Default port of the direct access listener. Kept at the historical value so
/// existing peers keep reaching this device by IP.
pub const DEFAULT_DIRECT_ACCESS_PORT: i32 = RENDEZVOUS_PORT + 2;

fn get_direct_port() -> i32 {
    let mut port = Config::get_option(OPTION_DIRECT_ACCESS_PORT)
        .parse::<i32>()
        .unwrap_or(0);
    if port <= 0 {
        port = DEFAULT_DIRECT_ACCESS_PORT;
    }
    port
}

pub async fn start_all() {
    check_zombie();
    let server = new_server();
    let server_cloned = server.clone();
    tokio::spawn(async move {
        direct_server(server_cloned).await;
    });
    #[cfg(target_os = "android")]
    let start_lan_listening = true;
    #[cfg(not(any(target_os = "android", target_os = "ios")))]
    let start_lan_listening = crate::platform::is_installed();
    if start_lan_listening {
        std::thread::spawn(move || {
            allow_err!(crate::lan::start_listening());
        });
    }
    scrap::codec::test_av1();
    loop {
        if option2bool("stop-service", &Config::get_option("stop-service")) {
            server.write().unwrap().close_connections();
        }
        Config::reset_online();
        sleep(1.).await;
    }
}

async fn direct_server(server: ServerPtr) {
    let mut listener = None;
    let mut port = 0;
    loop {
        let disabled = !option2bool(
            OPTION_DIRECT_SERVER,
            &Config::get_option(OPTION_DIRECT_SERVER),
        ) || option2bool("stop-service", &Config::get_option("stop-service"));
        if !disabled && listener.is_none() {
            port = get_direct_port();
            match hbb_common::tcp::listen_any(port as _).await {
                Ok(l) => {
                    listener = Some(l);
                    log::info!(
                        "Direct server listening on: {:?}",
                        listener.as_ref().map(|l| l.local_addr())
                    );
                }
                Err(err) => {
                    // to-do: pass to ui
                    log::error!(
                        "Failed to start direct server on port: {}, error: {}",
                        port,
                        err
                    );
                    loop {
                        if port != get_direct_port() {
                            break;
                        }
                        sleep(1.).await;
                    }
                }
            }
        }
        if let Some(l) = listener.as_mut() {
            if disabled || port != get_direct_port() {
                log::info!("Exit direct access listen");
                listener = None;
                continue;
            }
            if let Ok(Ok((stream, addr))) = hbb_common::timeout(1000, l.accept()).await {
                stream.set_nodelay(true).ok();
                log::info!("direct access from {}", addr);
                let local_addr = stream
                    .local_addr()
                    .unwrap_or(Config::get_any_listen_addr(true));
                let server = server.clone();
                tokio::spawn(async move {
                    allow_err!(
                        crate::server::create_tcp_connection(
                            server,
                            hbb_common::Stream::from(stream, local_addr),
                            addr,
                            false,
                            ConnectionMeta::default(), // Direct connections don't have server-side user context.
                        )
                        .await
                    );
                });
            } else {
                sleep(0.1).await;
            }
        } else {
            sleep(1.).await;
        }
    }
}
