extern crate amethyst_navigation as nav;

use amethyst::{
    core::{
        ecs::{Component, Join, NullStorage, Read, ReadStorage, System, WriteStorage},
        transform::{Transform, TransformBundle},
    },
    input::{
        is_close_requested, is_key_down, InputBundle, InputHandler, StringBindings, VirtualKeyCode,
    },
    prelude::*,
    renderer::{
        debug_drawing::DebugLinesComponent,
        palette::rgb::Srgba,
        plugins::{RenderDebugLines, RenderToWindow},
        types::DefaultBackend,
        Camera, RenderingBundle,
    },
    utils::application_root_dir,
    window::ScreenDimensions,
};
use nav::prelude::*;
use winit::MouseButton;

// player tag allows agent to be controlled by user input.
#[derive(Debug, Default, Copy, Clone)]
pub struct PlayerTag;

impl Component for PlayerTag {
    type Storage = NullStorage<Self>;
}

pub struct RenderSystem;

impl<'s> System<'s> for RenderSystem {
    type SystemData = (
        ReadStorage<'s, NavAgent>,
        WriteStorage<'s, DebugLinesComponent>,
    );

    fn run(&mut self, (agents, mut debugs): Self::SystemData) {
        for (agent, debug) in (&agents, &mut debugs).join() {
            debug.clear();
            if let Some(path) = agent.path() {
                for pair in path.windows(2) {
                    let f = pair[0];
                    let t = pair[1];
                    debug.add_line(
                        [f.x as f32, f.y as f32, f.z as f32].into(),
                        [t.x as f32, t.y as f32, t.z as f32].into(),
                        Srgba::new(0.0, 1.0, 0.0, 1.0),
                    );
                }
            }
            debug.add_circle_2d(
                [
                    agent.position.x as f32,
                    agent.position.y as f32,
                    agent.position.z as f32,
                ]
                .into(),
                20.0,
                6,
                Srgba::new(1.0, 0.0, 0.0, 1.0),
            );
        }
    }
}

pub struct CommandAgentsSystem;

impl<'s> System<'s> for CommandAgentsSystem {
    type SystemData = (
        Read<'s, InputHandler<StringBindings>>,
        Read<'s, NavMeshesRes>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, PlayerTag>,
    );

    fn run(&mut self, (input, meshes, mut agents, players): Self::SystemData) {
        for (agent, _) in (&mut agents, &players).join() {
            if input.mouse_button_is_down(MouseButton::Left) {
                if let Some((mut x, mut y)) = input.mouse_position() {
                    // convert screen coords to world coords.
                    x = x.max(0.0).min(800.0);
                    y = 600.0 - y.max(0.0).min(600.0);
                    // get mesh identifier from registry.
                    let mesh = meshes.meshes_iter().nth(0).unwrap().id();
                    // set player agent destination.
                    agent.set_destination(
                        // we can also select another agent to follow.
                        NavAgentTarget::Point((x as f64, y as f64).into()),
                        // use best quality point on nav mesh query.
                        NavQuery::Accuracy,
                        // use best quality of path finding.
                        NavPathMode::Accuracy,
                        mesh,
                    );
                }
            } else if input.mouse_button_is_down(MouseButton::Right) {
                agent.clear_path();
            } else if input.mouse_button_is_down(MouseButton::Middle) {
                if let Some((mut x, mut y)) = input.mouse_position() {
                    x = x.max(0.0).min(800.0);
                    y = 600.0 - y.max(0.0).min(600.0);
                    agent.position.x = x as f64;
                    agent.position.y = y as f64;
                    agent.position.z = 0.0;
                }
            }
        }
    }
}

#[derive(Default)]
pub struct MyState;

impl SimpleState for MyState {
    fn on_start(&mut self, data: StateData<'_, GameData<'_, '_>>) {
        let world = data.world;
        world.add_resource(NavMeshesRes::default());
        let dimensions = world.read_resource::<ScreenDimensions>().clone();
        init_camera(world, &dimensions);
        init_nav_mesh(world);
        init_agent::<PlayerTag>(world, 400.0, 450.0, 100.0);
    }

    fn handle_event(
        &mut self,
        mut _data: StateData<'_, GameData<'_, '_>>,
        event: StateEvent,
    ) -> SimpleTrans {
        if let StateEvent::Window(event) = &event {
            if is_close_requested(&event) || is_key_down(&event, VirtualKeyCode::Escape) {
                return Trans::Quit;
            }
        }

        Trans::None
    }
}

fn init_camera(world: &mut World, dimensions: &ScreenDimensions) {
    let mut transform = Transform::default();
    let w = dimensions.width();
    let h = dimensions.height();
    transform.set_translation_xyz(w * 0.5, h * 0.5, 1.0);

    world
        .create_entity()
        .with(Camera::standard_2d(w, h))
        .with(transform)
        .build();
}

fn init_nav_mesh(world: &mut World) {
    // create nav mesh vertices and triangles.
    // in future this will be loaded from asset.
    let vertices: Vec<NavVec3> = vec![
        (50.0, 50.0).into(),   // 0
        (500.0, 50.0).into(),  // 1
        (500.0, 100.0).into(), // 2
        (100.0, 100.0).into(), // 3
        (100.0, 300.0).into(), // 4
        (700.0, 300.0).into(), // 5
        (700.0, 50.0).into(),  // 6
        (750.0, 50.0).into(),  // 7
        (750.0, 550.0).into(), // 8
        (50.0, 550.0).into(),  // 9
    ];
    let triangles: Vec<NavTriangle> = vec![
        (1, 2, 3).into(), // 0
        (0, 1, 3).into(), // 1
        (0, 3, 4).into(), // 2
        (0, 4, 9).into(), // 3
        (4, 8, 9).into(), // 4
        (4, 5, 8).into(), // 5
        (5, 7, 8).into(), // 6
        (5, 6, 7).into(), // 7
    ];

    let mut debug = DebugLinesComponent::default();
    for triangle in &triangles {
        let f = vertices[triangle.first as usize];
        let s = vertices[triangle.second as usize];
        let t = vertices[triangle.third as usize];
        debug.add_line(
            [f.x as f32, f.y as f32, f.z as f32].into(),
            [s.x as f32, s.y as f32, s.z as f32].into(),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        );
        debug.add_line(
            [s.x as f32, s.y as f32, s.z as f32].into(),
            [t.x as f32, t.y as f32, t.z as f32].into(),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        );
        debug.add_line(
            [t.x as f32, t.y as f32, t.z as f32].into(),
            [f.x as f32, f.y as f32, f.z as f32].into(),
            Srgba::new(0.0, 0.0, 0.0, 1.0),
        );
    }
    world
        .create_entity()
        .with(Transform::default())
        .with(debug)
        .build();

    // build a nav mesh and register it so agents can traverse it.
    let mesh = NavMesh::new(vertices, triangles).unwrap();
    world.write_resource::<NavMeshesRes>().register(mesh);
}

fn init_agent<T>(world: &mut World, x: f32, y: f32, speed: f64)
where
    T: Component + Default + Copy + Send + Sync,
{
    let mut agent = NavAgent::new((x as f64, y as f64).into());
    agent.speed = speed;
    world
        .create_entity()
        // this entity is an agent.
        .with(agent)
        // this entity will do a simple movement along path.
        .with(SimpleNavDriverTag)
        .with(DebugLinesComponent::default())
        .with(T::default())
        .build();
}

fn main() -> amethyst::Result<()> {
    amethyst::start_logger(Default::default());

    let app_root = application_root_dir()?;

    let resources = app_root.join("examples/resources");
    let display_config = resources.join("display_config.ron");

    let game_data = GameDataBuilder::default()
        .with_bundle(TransformBundle::new())?
        .with_bundle(InputBundle::<StringBindings>::new())?
        .with_bundle(
            RenderingBundle::<DefaultBackend>::new()
                .with_plugin(
                    RenderToWindow::from_config_path(display_config)
                        .with_clear([0.34, 0.36, 0.52, 1.0]),
                )
                .with_plugin(RenderDebugLines::default()),
        )?
        // nav agent maintainment system allows agents find paths to their destinations.
        .with(NavAgentMaintainSystem::default(), "nav-agent-maintain", &[])
        // simple nav driver system allows agents with `SimpleNavDriverTag` to perform simple
        // movement along path.
        .with(SimpleNavDriverSystem, "simple-nav-driver", &[])
        .with(CommandAgentsSystem, "command-agents", &[])
        .with(RenderSystem, "render", &[]);

    let mut game = Application::new(resources, MyState, game_data)?;
    game.run();

    Ok(())
}
