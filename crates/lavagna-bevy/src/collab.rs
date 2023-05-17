use bevy::prelude::*;
use futures::{select, FutureExt};
use futures_timer::Delay;
use matchbox_socket::{PeerId, WebRtcSocket};
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::local_chalk::LocalChalk;
use crate::Chalk;

pub(crate) struct CollabPlugin;

impl Plugin for CollabPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup);
        app.add_system(emit_events);
        app.add_system(handle_events);
    }
}

fn setup(mut commands: Commands) {
    // FIXME
    let room_url = "ws://127.0.0.1:3536/lavagna";
    commands.insert_resource(Room::new(room_url));
}

fn emit_events(chalk_q: Query<&mut Chalk, With<LocalChalk>>, room: Res<Room>) {
    let chalk = chalk_q.single();

    if chalk.updated {
        let event = Event::Draw(DrawEvent {
            color: chalk.color.as_rgba_u32(),
            x: (chalk.x + 65535) as u32,
            y: (chalk.y + 65535) as u32,
        });
        room.send(event);
    }
}

fn handle_events(mut room: ResMut<Room>) {
    while let Some(event) = room.try_recv() {
        // TODO
        info!("{:?}", event);
    }
}

#[derive(Resource)]
struct Room {
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
    outgoing_tx: Sender<Event>,
    incoming_rx: Receiver<Event>,
    has_peers: Arc<AtomicBool>,
}

/// If this timeout is too long, the reactivity degrades
const TIMEOUT: Duration = Duration::from_millis(10);

impl Room {
    fn new(room_url: &str) -> Self {
        let (incoming_tx, incoming_rx) = channel::<Event>(1024);
        let (outgoing_tx, mut outgoing_rx) = channel::<Event>(1024);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let room_url = room_url.to_string();

        let has_peers: Arc<AtomicBool> = Arc::new(false.into());
        let has_peers_arc = has_peers.clone();

        runtime.spawn(async move {
            info!("connecting to matchbox server: {:?}", room_url);
            let (mut socket, loop_fut) = WebRtcSocket::new_reliable(room_url);
            info!("connected");

            let mut channel = socket.take_channel(0).unwrap();

            let loop_fut = loop_fut.fuse();
            futures::pin_mut!(loop_fut);

            let timeout = Delay::new(TIMEOUT);
            futures::pin_mut!(timeout);

            info!("enter loop");

            loop {
                socket.update_peers();

                let peers: Vec<_> = socket.connected_peers().collect();
                has_peers_arc.store(!peers.is_empty(), std::sync::atomic::Ordering::Relaxed);

                while let Ok(msg) = outgoing_rx.try_recv() {
                    for peer in &peers {
                        let packet = serde_json::to_vec(&msg).unwrap().into_boxed_slice();
                        channel.send(packet, *peer);
                    }
                }

                for (_peer, packet) in channel.receive() {
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

        Self {
            runtime,
            outgoing_tx,
            incoming_rx,
            has_peers,
        }
    }

    fn send(&self, event: Event) {
        if self.has_peers.load(std::sync::atomic::Ordering::Relaxed) {
            self.outgoing_tx
                .blocking_send(event)
                .map_err(|e| {
                    error!("Cannot send to WebRtc: {}", e);
                })
                .ok();
        }
    }

    fn try_recv(&mut self) -> Option<Event> {
        if self.has_peers.load(std::sync::atomic::Ordering::Relaxed) {
            self.incoming_rx.try_recv().ok()
        } else {
            None
        }
    }
}

// TODO
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
enum Event {
    Draw(DrawEvent),
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct DrawEvent {
    pub color: u32,
    pub x: u32,
    pub y: u32,
}
