use std::time::Duration;

use bevy::{diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin}, prelude::*};
use rand::SeedableRng;

fn main() {
    let mut app = App::new();
    app
        .add_plugins(DefaultPlugins)
        .add_plugins((FrameTimeDiagnosticsPlugin, EntityCountDiagnosticsPlugin))
        .add_systems(Startup, spawn_cam);
        #[cfg(feature="ring")]
        app.add_systems(Update, spawn_ring)
        .insert_resource(RingCount(0));
        #[cfg(feature="static")]
        app.add_systems(Startup, big_ring);
        app.add_systems(FixedUpdate, frame_time)
        .insert_resource(FixedTime::new(Duration::from_secs(5)))
        .init_resource::<HexMeshs>()
    .run()
}

#[derive(Resource)]
struct HexMeshs{
    rng: rand::rngs::StdRng,
    handles: Vec<Handle<Scene>>,
}

impl HexMeshs {
    fn next(&mut self) -> Handle<Scene> {
        use rand::Rng;
        self.handles[self.rng.gen_range(0..self.handles.len())].clone()
    }
}

const SEED: [u8; 32] = [
    1, 0, 52, 0, 0, 0, 0, 0, 1, 0, 10, 0, 22, 32, 0, 0, 2, 0, 55, 49, 0, 11, 0, 0, 3, 0, 0, 0, 0,
    0, 2, 92,
];

impl FromWorld for HexMeshs {
    fn from_world(world: &mut World) -> Self {
        let asset_server = world.resource::<AssetServer>();
        HexMeshs {
            rng: rand::rngs::StdRng::from_seed(SEED),
            handles: vec![
                asset_server.load("Hexs/sand.glb#Scene0"),
                asset_server.load("Hexs/grass.glb#Scene0"),
                asset_server.load("Hexs/dirt.glb#Scene0"),
                asset_server.load("Hexs/stone.glb#Scene0"),
                asset_server.load("Hexs/water.glb#Scene0"),
                asset_server.load("Hexs/water-rocks.glb#Scene0"),
                asset_server.load("Hexs/water-island.glb#Scene0"),
                asset_server.load("Hexs/grass-hill.glb#Scene0"),
                asset_server.load("Hexs/grass-forest.glb#Scene0"),
            ]
        }
    }
}

fn spawn_cam(
    mut commands: Commands,
) {
    commands.spawn(Camera3dBundle{
        #[cfg(feature="static")]
        transform: Transform::from_translation(Vec3::new(0., 15., 35.)).looking_at(Vec3::ZERO, Vec3::Y),
        #[cfg(feature="ring")]
        transform: Transform::from_translation(Vec3::new(100., 100., 100.)).looking_at(Vec3::ZERO, Vec3::Y),
        ..Camera3dBundle::default()
    });
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_translation(Vec3::Y * 100.).looking_at(Vec3::ZERO, Vec3::X),
        ..Default::default()
    });
}

#[derive(Debug, Resource, DerefMut, Deref)]
struct RingCount(i32);

fn spawn_ring(
    frame_time: Res<DiagnosticsStore>,
    mut commands: Commands,
    mut ring_cound: ResMut<RingCount>,
    mut hexs: ResMut<HexMeshs>,
) {
    if let Some(d) = frame_time.get(FrameTimeDiagnosticsPlugin::FPS) {
        if let Some(d) = d.value() {
            if d < 30. {
                return;
            }
        } else {
            error!("No FPS");
            return;
        }
    } else {
        return;
    }
    ring_cound.0 += 1;
    for id in HexIdIterator::new(ring_cound.0) {
        if (id.distance(CellId::ZERO) as i32) < ring_cound.0 {continue;}
        commands.spawn(SceneBundle {
            scene: hexs.next(),
            transform: Transform::from_translation(id.xyz(0.)),
            ..Default::default()
        });
    }
}

fn big_ring(
    mut commands: Commands,
    mut hexs: ResMut<HexMeshs>,
) {
    for id in HexIdIterator::new(25) {
        commands.spawn(SceneBundle {
            scene: hexs.next(),
            transform: Transform::from_translation(id.xyz(0.)),
            ..Default::default()
        });
    }
}

fn frame_time(
    frame_time: Res<DiagnosticsStore>,
    ring: Option<Res<RingCount>>
) {
    if let Some(d) = frame_time.get(FrameTimeDiagnosticsPlugin::FRAME_TIME) {
        println!("Frame Took {:?}", (d.average().unwrap_or_default() * 100.).round() / 100.);   
    }
    if let Some(d) = frame_time.get(FrameTimeDiagnosticsPlugin::FPS) {
        println!("FPS is {:?}", (d.average().unwrap_or_default() * 100.).round() / 100.);   
    }
    if let Some(d) = frame_time.get(EntityCountDiagnosticsPlugin::ENTITY_COUNT) {
        println!("Rendering {} Entitys", d.value().unwrap_or_default());
    }
    if let Some(ring) = ring {
        println!("Spawned {} Rings", ring.0)
    }
}

struct CellId {
    q: i32,
    r: i32,
}

struct HexIdIterator {
    q: std::ops::RangeInclusive<i32>,
    current_q: i32,
    r: std::ops::RangeInclusive<i32>,
    range: i32,
}

impl HexIdIterator {
    fn new(range: i32) -> HexIdIterator {
        HexIdIterator {
            q: -range + 1..=range,
            current_q: -range,
            r: 0..=range,
            range,
        }
    }
}

impl Iterator for HexIdIterator {
    type Item = CellId;
    fn next(&mut self) -> Option<Self::Item> {
        match self.r.next() {
            None => match self.q.next() {
                Some(q) => {
                    self.current_q = q;
                    self.r = (-self.range).max(-q - self.range)..=(self.range).min(-q + self.range);
                    if let Some(r) = self.r.next() {
                        Some(CellId {
                            q: self.current_q,
                            r,
                        })
                    } else {
                        None
                    }
                }
                None => None,
            },
            Some(r) => Some(CellId {
                q: self.current_q,
                r,
            }),
        }
    }
}

impl CellId {
    pub const ZERO: CellId = CellId { q: 0, r: 0 };

    pub const fn s(&self) -> i32 {
        -self.q - self.r
    }
    
    pub const fn r(&self) -> i32 {
        self.r
    }

    pub const fn q(&self) -> i32 {
        self.q
    }

    pub const fn new(q: i32, r: i32) -> CellId {
        Self { q, r }
    }

    #[inline(always)]
    pub fn xyz(&self, y: f32) -> Vec3 {
        let z = 0.75 * self.q as f32;
        let x = (self.q as f32 * 0.5 + self.r as f32) * 0.86602540378443864676372317075294;
        Vec3::new(x, y, z)
    }

    pub fn distance(&self, othor: CellId) -> u32 {
        let res =
            ((self.q - othor.q).abs() + (self.r - othor.r).abs() + (self.s() - othor.s()).abs())
                / 2;
        res as u32
    }
}