#![deny(clippy::all)]
#![forbid(unsafe_code)]

extern crate core;

use futures::{select, FutureExt};
use futures_timer::Delay;
use lavagna_core::Command;
use matchbox_socket::WebRtcSocket;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::error::SendError;
use tokio::sync::mpsc::{Receiver, Sender};

const TIMEOUT: Duration = Duration::from_millis(10);

pub trait CollaborationChannel {
    fn send_command(&self, cmd: Command) -> Result<(), SendError<Command>>;
    fn rx(&mut self) -> &mut Receiver<Command>;
}

pub struct WebRtcCollaborationChannel {
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
    tx: Sender<Command>,
    rx: Receiver<Command>,
}

impl WebRtcCollaborationChannel {
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

impl CollaborationChannel for WebRtcCollaborationChannel {
    fn send_command(&self, cmd: Command) -> Result<(), SendError<Command>> {
        self.tx.blocking_send(cmd)
    }

    fn rx(&mut self) -> &mut Receiver<Command> {
        &mut self.rx
    }
}

pub struct DummyCollaborationChannel(Receiver<Command>);

impl Default for DummyCollaborationChannel {
    fn default() -> Self {
        let (_, rx) = channel(1);
        Self(rx)
    }
}

impl CollaborationChannel for DummyCollaborationChannel {
    fn send_command(&self, _cmd: Command) -> Result<(), SendError<Command>> {
        /* Just ignore everything */
        Ok(())
    }

    fn rx(&mut self) -> &mut Receiver<Command> {
        &mut self.0
    }
}
