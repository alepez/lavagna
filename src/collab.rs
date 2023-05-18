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

pub(crate) struct CollabPlugin {
    opt: CollabPluginOpt,
}

#[derive(Debug, Resource, Clone)]
pub struct CollabPluginOpt {
    pub url: String,
    pub collab_id: u16,
}

impl CollabPlugin {
    pub fn new(opt: CollabPluginOpt) -> Self {
        Self { opt }
    }
}

impl Plugin for CollabPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.insert_resource(self.opt.clone());
        app.add_startup_system(setup);
        app.add_system(emit_events);
        app.add_system(handle_events);
    }
}

fn setup(mut commands: Commands, opt: Res<CollabPluginOpt>) {
    let room = Room::new(&opt.url, opt.collab_id);
    commands.insert_resource(room);
}

fn emit_events(chalk: ResMut<LocalChalk>, room: Res<Room>) {
    let chalk = chalk.get();

    if chalk.updated && chalk.pressed {
        room.send(Event::Draw(chalk.into()));
    }

    if chalk.just_released {
        room.send(Event::Release);
    }
}

fn handle_events(mut commands: Commands, mut room: ResMut<Room>, mut chalk_q: Query<&mut Chalk>) {
    while let Some(AddressedEvent { src, event }) = room.try_recv() {
        match event {
            Event::Draw(e) => handle_draw(&mut commands, src, &e, &mut room, &mut chalk_q),
            Event::Release => handle_release(src, &mut room, &mut chalk_q),
        }
    }
}

fn handle_release(src: CollabId, room: &mut Room, chalk_q: &mut Query<&mut Chalk>) {
    if let Some(entity) = room.peers.0.get(&src) {
        if let Ok(mut chalk) = chalk_q.get_mut(*entity) {
            chalk.pressed = false;
            chalk.just_released = true;
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
        *chalk = event.into();
    }
}

const POSITION_OFFSET: i32 = 1 << 30;

impl From<&Chalk> for DrawEvent {
    fn from(chalk: &Chalk) -> Self {
        Self {
            color: chalk.color.as_rgba_u32(),
            x: (chalk.x + POSITION_OFFSET) as u32,
            y: (chalk.y + POSITION_OFFSET) as u32,
            line_width: chalk.line_width,
        }
    }
}

impl From<&DrawEvent> for Chalk {
    fn from(event: &DrawEvent) -> Self {
        Self {
            pressed: true,
            updated: true,
            x: (event.x as i32) - POSITION_OFFSET,
            y: (event.y as i32) - POSITION_OFFSET,
            color: color_from_u32(event.color),
            line_width: event.line_width,
            just_released: false,
        }
    }
}

fn color_from_u32(n: u32) -> Color {
    let r = ((n) & 0xFF) as u8;
    let g = ((n >> 8) & 0xFF) as u8;
    let b = ((n >> 16) & 0xFF) as u8;
    let a = ((n >> 24) & 0xFF) as u8;
    Color::rgba_u8(r, g, b, a)
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
    fn new(room_url: &str, my_id: u16) -> Self {
        let (incoming_tx, incoming_rx) = channel::<AddressedEvent>(1024);
        let (outgoing_tx, outgoing_rx) = channel::<Event>(1024);

        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let room_url = room_url.to_string();
        let has_peers: Arc<AtomicBool> = Arc::new(false.into());
        let has_peers_clone = has_peers.clone();
        let my_id = CollabId::from(my_id);

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
    Release,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct DrawEvent {
    color: u32,
    line_width: u32,
    x: u32,
    y: u32,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct AddressedEvent {
    src: CollabId,
    event: Event,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone, PartialEq, Eq, Hash)]
struct CollabId(u16);

impl From<u16> for CollabId {
    fn from(value: u16) -> Self {
        Self(value)
    }
}
