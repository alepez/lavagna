#![allow(clippy::needless_pass_by_value)]
#![allow(clippy::cast_possible_truncation)]
#![allow(clippy::module_name_repetitions)]

use crate::drawing::{make_chalk, ClearEvent};
use crate::{Chalk, Stats};
use bevy::prelude::*;
use bevy::utils::{Duration, HashMap, Instant};
use bevy_matchbox::prelude::*;
use bevy_prototype_lyon::prelude::{
    GeometryBuilder, LineCap, LineJoin, ShapeBundle, Stroke, StrokeOptions,
};
use bevy_prototype_lyon::shapes;
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

        app.add_systems(Update, room_system);
        app.add_systems(Update, emit_events);
        app.add_systems(Update, receive_events);
        app.add_systems(Update, handle_clear_event);
        app.add_systems(Update, update_peer_cursor_visibility);
        app.add_systems(Update, update_stats);
    }
}

fn emit_events(chalk: ResMut<LocalChalk>, mut room: ResMut<Room>) {
    let chalk = chalk.get();

    if chalk.updated {
        room.send(Event::Move(chalk.into()));
    }

    if chalk.just_released {
        room.send(Event::Release);
    }
}

fn receive_events(
    mut commands: Commands,
    mut room: ResMut<Room>,
    mut chalk_q: Query<&mut Chalk>,
    mut cursor_q: Query<(&mut Transform, &mut Stroke, &mut PeerCursor), With<PeerCursor>>,
    mut clear_event: EventWriter<ClearEvent>,
) {
    // This is needed, otherwise it can hangs forever when the connection is not established
    if !room.is_ok() {
        return;
    }

    let my_id = room.collab_id;

    for &AddressedEvent { src, event } in room.receive().iter().filter(|e| e.src != my_id) {
        match event {
            Event::Move(e) => handle_draw(
                &mut commands,
                src,
                &e,
                &mut room,
                &mut chalk_q,
                &mut cursor_q,
            ),
            Event::Release => handle_release(src, &room, &mut chalk_q),
            Event::Clear => {
                clear_event.send(ClearEvent::local_only());
            }
        }
    }
}

fn handle_release(src: CollabId, room: &Room, chalk_q: &mut Query<&mut Chalk>) {
    if let Some(peer) = room.peers.0.get(&src) {
        if let Ok(mut chalk) = chalk_q.get_mut(peer.chalk) {
            chalk.pressed = false;
            chalk.just_released = true;
        }
    }
}

fn handle_draw(
    commands: &mut Commands,
    src: CollabId,
    event: &MoveEvent,
    room: &mut Room,
    chalk_q: &mut Query<&mut Chalk>,
    cursor_q: &mut Query<(&mut Transform, &mut Stroke, &mut PeerCursor), With<PeerCursor>>,
) {
    let peer: &Peer = room.peers.0.entry(src).or_insert_with(|| {
        let peer_cursor = make_peer_cursor(color_from_u32(event.color), src);
        let cursor_id = commands.spawn(peer_cursor).id();
        let chalk_id = commands.spawn(make_chalk(event.into())).id();

        Peer::new(chalk_id, cursor_id)
    });

    if let Ok(mut chalk) = chalk_q.get_mut(peer.chalk) {
        *chalk = event.into();
    }

    if let Ok((mut t, mut stroke, mut peer_cursor)) = cursor_q.get_mut(peer.cursor) {
        t.translation.x = event.x.into();
        t.translation.y = event.y.into();
        stroke.color = color_from_u32(event.color).into();
        peer_cursor.touch();
    }
}

fn update_peer_cursor_visibility(
    mut cursor_q: Query<(&mut Visibility, &PeerCursor), With<PeerCursor>>,
) {
    for (mut visibility, peer_cursor) in &mut cursor_q {
        *visibility = if peer_cursor.is_active() {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }
}

impl From<&Chalk> for MoveEvent {
    fn from(chalk: &Chalk) -> Self {
        Self {
            color: color_to_u32(chalk.color),
            x: chalk.x as i16,
            y: chalk.y as i16,
            line_width: chalk.line_width as u8,
            pressed: chalk.pressed,
        }
    }
}

impl From<&MoveEvent> for Chalk {
    fn from(event: &MoveEvent) -> Self {
        Self {
            pressed: event.pressed,
            updated: true,
            x: event.x.into(),
            y: event.y.into(),
            color: color_from_u32(event.color),
            line_width: event.line_width.into(),
            just_released: false,
        }
    }
}

#[allow(clippy::many_single_char_names)]
fn color_from_u32(n: u32) -> Srgba {
    let r = ((n) & 0xFF) as u8;
    let g = ((n >> 8) & 0xFF) as u8;
    let b = ((n >> 16) & 0xFF) as u8;
    let a = ((n >> 24) & 0xFF) as u8;
    Srgba::rgba_u8(r, g, b, a)
}

#[allow(clippy::cast_sign_loss)]
fn color_to_u32(color: Srgba) -> u32 {
    let r = (color.red * 255.0) as u32;
    let g = (color.green * 255.0) as u32;
    let b = (color.blue * 255.0) as u32;
    let a = (color.alpha * 255.0) as u32;
    (a << 24) | (b << 16) | (g << 8) | r
}

#[derive(Default)]
struct Peers(HashMap<CollabId, Peer>);

struct Peer {
    chalk: Entity,
    cursor: Entity,
}

impl Peer {
    fn new(chalk: Entity, cursor: Entity) -> Self {
        Self { chalk, cursor }
    }
}

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
    Move(MoveEvent),
    Release,
    Clear,
}

#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
struct MoveEvent {
    color: u32,
    line_width: u8,
    x: i16,
    y: i16,
    pressed: bool,
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
    let Ok(peers) = room.socket.try_update_peers() else {
        log::error!("failed to update peers");
        return;
    };
    for (peer, new_state) in peers {
        match new_state {
            PeerState::Connected => info!("peer {peer:?} connected"),
            PeerState::Disconnected => info!("peer {peer:?} disconnected"),
        }
    }
}

fn handle_clear_event(mut events: EventReader<ClearEvent>, mut room: ResMut<Room>) {
    let clear = events.read().filter(|e| e.must_be_forwarded()).count() > 0;

    if clear {
        room.send(Event::Clear);
    }
}

fn update_stats(room: Res<Room>, mut stats: ResMut<Stats>) {
    stats.collab.active = true;
    stats.collab.peers = room.socket.connected_peers().count();
}

#[derive(Component)]
struct PeerCursor {
    #[allow(dead_code)]
    id: CollabId,
    last_seen: Instant,
}

impl PeerCursor {
    fn new(id: CollabId) -> Self {
        log::info!("new peer cursor {:?}", id);

        Self {
            id,
            last_seen: Instant::now(),
        }
    }

    fn is_active(&self) -> bool {
        self.last_seen.elapsed() < Duration::from_secs(1)
    }

    fn touch(&mut self) {
        self.last_seen = Instant::now();
    }
}

fn make_peer_cursor(color: Srgba, id: CollabId) -> (ShapeBundle, Stroke, PeerCursor) {
    let shape = shapes::Circle {
        radius: 1.0,
        center: Vec2::new(0.0, 0.0),
    };

    // z-index at maximum before clipping pane
    let transform = Transform {
        translation: Vec3::new(0., 0., 999.0),
        ..default()
    };

    let shape = ShapeBundle {
        path: GeometryBuilder::build_as(&shape),
        spatial: transform.into(),
        ..default()
    };

    let stroke = Stroke {
        color: color.into(),
        options: StrokeOptions::default()
            .with_line_width(10.0)
            .with_line_join(LineJoin::Round)
            .with_line_cap(LineCap::Round),
    };
    let peer_cursor = PeerCursor::new(id);

    (shape, stroke, peer_cursor)
}
