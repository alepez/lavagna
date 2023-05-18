use crate::drawing::make_chalk;
use crate::Chalk;
use bevy::prelude::*;
use bevy::utils::HashMap;
use futures::{select, FutureExt};
use futures_timer::Delay;
use matchbox_socket::WebRtcSocket;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::channel;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::local_chalk::LocalChalk;

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
    let room = Room::new(room_url);
    commands.insert_resource(room);
}

fn emit_events(mut chalk: ResMut<LocalChalk>, room: Res<Room>) {
    let chalk = &mut chalk.get_mut();

    if chalk.updated && chalk.pressed {
        let event = Event::Draw(DrawEvent {
            color: chalk.color.as_rgba_u32(),
            x: (chalk.x + 65535) as u32,
            y: (chalk.y + 65535) as u32,
        });
        room.send(event);
    }
}

fn handle_events(mut commands: Commands, mut room: ResMut<Room>, mut chalk_q: Query<&mut Chalk>) {
    while let Some(AddressedEvent { src, event }) = room.try_recv() {
        match event {
            Event::Draw(e) => {
                handle_draw(&mut commands, src, &e, &mut room, &mut chalk_q);
            }
        }
    }
}

fn handle_draw(
    commands: &mut Commands,
    src: CollabId,
    event: &DrawEvent,
    room: &mut Room,
    chalk_q: &mut Query<&mut Chalk>,
) {
    let entity: &Entity = room
        .peers
        .0
        .entry(src)
        .or_insert_with(|| commands.spawn(make_chalk(event.into())).id());

    if let Ok(mut chalk) = chalk_q.get_mut(*entity) {
        chalk.pressed = true;
        chalk.updated = true;
        chalk.x = (event.x as i32) - 65535;
        chalk.y = (event.y as i32) - 65535;
        chalk.color = Color::WHITE; // TODO
        chalk.line_width = 10; // TODO
    }
}

impl From<&DrawEvent> for Chalk {
    fn from(event: &DrawEvent) -> Self {
        Self {
            pressed: true,
            updated: true,
            x: (event.x as i32) - 65535,
            y: (event.y as i32) - 65535,
            color: Color::WHITE, // TODO
            line_width: 10,      // TODO
        }
    }
}

#[derive(Default)]
struct Peers(HashMap<CollabId, Entity>);

#[derive(Resource)]
struct Room {
    #[allow(dead_code)]
    runtime: tokio::runtime::Runtime,
    outgoing_tx: Sender<Event>,
    incoming_rx: Receiver<AddressedEvent>,
    has_peers: Arc<AtomicBool>,
    peers: Peers,
}

/// If this timeout is too long, the reactivity degrades
const TIMEOUT: Duration = Duration::from_millis(10);

async fn room_run(
    room_url: std::string::String,
    my_id: CollabId,
    has_peers: Arc<AtomicBool>,
    incoming_tx: Sender<AddressedEvent>,
    mut outgoing_rx: Receiver<Event>,
) {
    let (mut socket, loop_fut) = WebRtcSocket::new_reliable(room_url);

    let mut channel = socket.take_channel(0).unwrap();

    let loop_fut = loop_fut.fuse();
    futures::pin_mut!(loop_fut);

    let timeout = Delay::new(TIMEOUT);
    futures::pin_mut!(timeout);

    loop {
        socket.update_peers();

        let peers: Vec<_> = socket.connected_peers().collect();
        has_peers.store(!peers.is_empty(), std::sync::atomic::Ordering::Relaxed);

        while let Ok(event) = outgoing_rx.try_recv() {
            let event = AddressedEvent { src: my_id, event };
            for peer in &peers {
                let packet = serde_json::to_vec(&event).unwrap().into_boxed_slice();
                info!("TX {}", std::str::from_utf8(&packet).unwrap());
                channel.send(packet, *peer);
            }
        }

        for (_peer, packet) in channel.receive() {
            let packet = packet;
            info!("RX {}", std::str::from_utf8(&packet).unwrap());
            let event = serde_json::from_slice(&packet).unwrap();
            incoming_tx.send(event).await.unwrap();
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
}

impl Room {
    fn new(room_url: &str) -> Self {
        let (incoming_tx, incoming_rx) = channel::<AddressedEvent>(1024);
        let (outgoing_tx, outgoing_rx) = channel::<Event>(1024);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let room_url = room_url.to_string();
        let has_peers: Arc<AtomicBool> = Arc::new(false.into());
        let has_peers_clone = has_peers.clone();
        let my_id = CollabId::random();

        runtime.spawn(async move {
            room_run(room_url, my_id, has_peers_clone, incoming_tx, outgoing_rx).await;
        });

        Self {
            runtime,
            outgoing_tx,
            incoming_rx,
            has_peers,
            peers: default(),
        }
    }

    fn send(&self, event: Event) {
        if self.has_peers.load(std::sync::atomic::Ordering::Relaxed) {
            let _ = self.outgoing_tx.blocking_send(event).map_err(|e| {
                error!("Cannot send to WebRtc: {}", e);
            });
        }
    }

    fn try_recv(&mut self) -> Option<AddressedEvent> {
        if self.has_peers.load(std::sync::atomic::Ordering::Relaxed) {
            self.incoming_rx.try_recv().ok()
        } else {
            None
        }
    }
}

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

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct AddressedEvent {
    src: CollabId,
    event: Event,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
struct CollabId(u16);

impl CollabId {
    fn random() -> Self {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let n = rng.gen();
        Self(n)
    }
}
