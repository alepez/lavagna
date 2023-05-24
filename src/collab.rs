use crate::drawing::{make_chalk, ClearEvent};
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
        let socket = MatchboxSocket::new_reliable(&self.opt.url);
        let collab_id = CollabId(self.opt.collab_id);
        let room = Room::new(socket, collab_id);
        app.insert_resource(room);

        app.add_system(room_system);
        app.add_system(emit_events);
        app.add_system(receive_events);
        app.add_system(handle_clear_event);
    }
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

fn receive_events(
    mut commands: Commands,
    mut room: ResMut<Room>,
    mut chalk_q: Query<&mut Chalk>,
    mut clear_event: EventWriter<ClearEvent>,
) {
    // This is needed, otherwise it can hangs forever when the connection is not established
    if !room.is_ok() {
        return;
    }

    let my_id = room.collab_id;

    for &AddressedEvent { src, event } in room.receive().iter().filter(|e| e.src != my_id) {
        match event {
            Event::Draw(e) => handle_draw(&mut commands, src, &e, &mut room, &mut chalk_q),
            Event::Release => handle_release(src, &mut room, &mut chalk_q),
            Event::Clear => clear_event.send(ClearEvent::local_only()),
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

impl From<&Chalk> for DrawEvent {
    fn from(chalk: &Chalk) -> Self {
        Self {
            color: chalk.color.as_rgba_u32(),
            x: chalk.x as i16,
            y: chalk.y as i16,
            line_width: chalk.line_width as u8,
        }
    }
}

impl From<&DrawEvent> for Chalk {
    fn from(event: &DrawEvent) -> Self {
        Self {
            pressed: true,
            updated: true,
            x: event.x as i32,
            y: event.y as i32,
            color: color_from_u32(event.color),
            line_width: event.line_width as u32,
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
        let mut payload = Vec::new();
        ciborium::ser::into_writer(&event, &mut payload).unwrap();
        for peer in peers {
            self.socket.send(payload.clone().into(), peer);
        }
    }

    fn receive(&mut self) -> Vec<AddressedEvent> {
        self.socket
            .receive()
            .iter()
            .map(|(_, payload)| payload)
            .filter_map(|payload| ciborium::de::from_reader(&payload[..]).ok())
            .collect()
    }

    fn is_ok(&self) -> bool {
        self.socket.connected_peers().count() > 0
    }
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
enum Event {
    Draw(DrawEvent),
    Release,
    Clear,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct DrawEvent {
    color: u32,
    line_width: u8,
    x: i16,
    y: i16,
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

fn handle_clear_event(mut events: EventReader<ClearEvent>, mut room: ResMut<Room>) {
    let clear = events.iter().filter(|e| e.must_be_forwarded()).count() > 0;

    if clear {
        room.send(Event::Clear);
    }
}
