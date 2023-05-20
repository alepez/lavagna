use crate::drawing::make_chalk;
use crate::Chalk;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_matchbox::prelude::*;
use serde::{Deserialize, Serialize};

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
        app.add_system(room_system);
        app.add_system(emit_events);
        app.add_system(receive_events);
    }
}

fn setup(mut commands: Commands, opt: Res<CollabPluginOpt>) {
    let socket = MatchboxSocket::new_reliable(&opt.url);
    let collab_id = CollabId(opt.collab_id);
    let room = Room::new(socket, collab_id);
    commands.insert_resource(room);
}

fn emit_events(chalk: ResMut<LocalChalk>, mut room: ResMut<Room>) {
    let chalk = chalk.get();

    if chalk.updated && chalk.pressed {
        room.send(Event::Draw(chalk.into()));
    }

    if chalk.just_released {
        room.send(Event::Release);
    }
}

fn receive_events(mut commands: Commands, mut room: ResMut<Room>, mut chalk_q: Query<&mut Chalk>) {
    for AddressedEvent { src, event } in room.receive() {
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
    socket: MatchboxSocket<SingleChannel>,
    collab_id: CollabId,
    peers: Peers,
}

impl Room {
    fn new(socket: MatchboxSocket<SingleChannel>, collab_id: CollabId) -> Self {
        Self {
            socket,
            collab_id,
            peers: Peers::default(),
        }
    }

    fn send(&mut self, event: Event) {
        let event = AddressedEvent {
            src: self.collab_id,
            event,
        };
        let peers: Vec<_> = self.socket.connected_peers().collect();
        let packet = serde_json::to_vec(&event).unwrap().into_boxed_slice();
        for peer in peers {
            self.socket.send(packet.clone(), peer);
        }
    }

    fn receive(&mut self) -> Vec<AddressedEvent> {
        self.socket
            .receive()
            .iter()
            .filter_map(|(_peer_id, payload)| serde_json::from_slice(&payload).ok())
            .inspect(|event| info!("RX {:?}", event))
            .collect()
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

// regularly call update_peers to update the list of connected peers
fn room_system(mut room: ResMut<Room>) {
    for (peer, new_state) in room.socket.update_peers() {
        match new_state {
            PeerState::Connected => info!("peer {peer:?} connected"),
            PeerState::Disconnected => info!("peer {peer:?} disconnected"),
        }
    }
}
