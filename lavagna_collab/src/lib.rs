#![deny(clippy::all)]
#![forbid(unsafe_code)]

extern crate core;

use futures::{select, FutureExt};
use futures_timer::Delay;
use lavagna_core::{CollabId, Command, CommandSender};
use matchbox_socket::WebRtcSocket;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::{Receiver, Sender};

/// If this timeout is too long, the reactivity degrades
const TIMEOUT: Duration = Duration::from_millis(10);

/// A trait for all kinds of collaboration channels
pub trait CollaborationChannel: CommandSender {
    fn rx(&mut self) -> &mut Receiver<Command>;
}

/// An implementation of collaboration channel over WebRtc
pub struct WebRtcCollaborationChannel {
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
    tx: Sender<Command>,
    rx: Receiver<Command>,
}

impl WebRtcCollaborationChannel {
    /// Create a WebRtc collaboration channel, given an url on a signaling server
    pub fn new(room_url: &str) -> Self {
        let (incoming_tx, incoming_rx) = channel::<Command>(1024);
        let (outgoing_tx, mut outgoing_rx) = channel::<Command>(1024);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let room_url = room_url.to_string();

        runtime.spawn(async move {
            let (mut socket, loop_fut) = WebRtcSocket::new(room_url);

            let loop_fut = loop_fut.fuse();
            futures::pin_mut!(loop_fut);

            let timeout = Delay::new(TIMEOUT);
            futures::pin_mut!(timeout);

            let mut peers = Vec::new();

            loop {
                for peer in socket.accept_new_connections() {
                    peers.push(peer);
                }

                while let Ok(msg) = outgoing_rx.try_recv() {
                    for peer in &peers {
                        let packet = serde_json::to_vec(&msg).unwrap().into_boxed_slice();
                        socket.send(packet, peer);
                    }
                }

                for (_peer, packet) in socket.receive() {
                    let packet = packet;
                    let msg = serde_json::from_slice(&packet).unwrap();
                    incoming_tx.send(msg).await.unwrap();
                }

                select! {
                    _ = (&mut timeout).fuse() => {
                        timeout.reset(TIMEOUT);
                    }

                    _ = &mut loop_fut => {
                        break;
                    }
                }
            }
        });

        WebRtcCollaborationChannel {
            runtime,
            tx: outgoing_tx,
            rx: incoming_rx,
        }
    }
}

impl CommandSender for WebRtcCollaborationChannel {
    fn send_command(&mut self, cmd: Command) {
        self.tx
            .blocking_send(cmd)
            .map_err(|e| {
                log::error!("Cannot send to WebRtc: {}", e);
            })
            .ok();
    }
}

impl CollaborationChannel for WebRtcCollaborationChannel {
    fn rx(&mut self) -> &mut Receiver<Command> {
        &mut self.rx
    }
}

/// A dummy collaboration channel, which sends to nobody and never receive any
/// message.
pub struct DummyCollaborationChannel(Receiver<Command>);

impl Default for DummyCollaborationChannel {
    fn default() -> Self {
        let (_, rx) = channel(1);
        Self(rx)
    }
}

impl CommandSender for DummyCollaborationChannel {
    fn send_command(&mut self, _cmd: Command) {
        /* Just ignore everything */
    }
}

impl CollaborationChannel for DummyCollaborationChannel {
    fn rx(&mut self) -> &mut Receiver<Command> {
        &mut self.0
    }
}

pub enum SupportedCollaborationChannel {
    WebRtc(WebRtcCollaborationChannel),
    Dummy(DummyCollaborationChannel),
}

impl Default for SupportedCollaborationChannel {
    fn default() -> Self {
        Self::Dummy(Default::default())
    }
}

impl SupportedCollaborationChannel {
    pub fn new(collab_url: &str) -> Self {
        Self::WebRtc(WebRtcCollaborationChannel::new(collab_url))
    }
}

impl CommandSender for SupportedCollaborationChannel {
    fn send_command(&mut self, cmd: Command) {
        match self {
            Self::WebRtc(chan) => chan.send_command(cmd),
            Self::Dummy(chan) => chan.send_command(cmd),
        }
    }
}

impl CollaborationChannel for SupportedCollaborationChannel {
    fn rx(&mut self) -> &mut Receiver<Command> {
        match self {
            Self::WebRtc(chan) => chan.rx(),
            Self::Dummy(chan) => chan.rx(),
        }
    }
}

pub struct CollabOpt {
    pub url: String,
    pub id: CollabId,
}
