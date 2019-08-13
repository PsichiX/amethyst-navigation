# Amethyst Navigation
### Nav-mesh based 2D and 3D navigation toolset for Amethyst game engine

## Table of contents
1. [About](#about)
1. [Manifest](#manifest)
1. [Setup](#setup)
1. [Build Nav Mesh](#build-nav-mesh)
1. [Create an Agent](#create-an-agent)
1. [Set Agent destination](#set-agent-destination)
1. [Custom Agent Driver](#custom-agent-driver)

## About
At one point in your game development you may want to allow your character to
find a path in the world to move along it. This Amethyst plugin gives you
opportunity to use nav mesh technology to build a nav mesh (special mesh that
describes walkable areas), make an agent and set its destination to follow.
Your destination can be either a point in world space or another agent entity.

## Manifest
Cargo.toml:
```toml
[dependencies]
amethyst-navigation = "0.1"
```

## Setup
With `amethyst-navigation` in prior to work, you have to install two systems:
- `NavAgentMaintainSystem` to allow agents to find paths to their destinations.
- `SimpleNavDriverSystem` to allow agents to perform simple movement along paths.

```rust
use amethyst_navigation::prelude::*;

let game_data = GameDataBuilder::default()
    // ...
    // INSTALL ENGINE BUNDLES HERE
    // ...
    // nav agent maintainment system allows agents find paths to their destinations.
    .with(NavAgentMaintainSystem::default(), "nav-agent-maintain", &[])
    // simple nav driver system allows agents with `SimpleNavDriverTag` to perform simple
    // movement along path.
    .with(SimpleNavDriverSystem, "simple-nav-driver", &[]);
```

## Build Nav Mesh
Nav mesh is special kind of mesh that allows agents to find shortest paths to
traverse from one point to another, that points always lay on given navmesh.

```rust
use amethyst_navigation::prelude::*;

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

// build a nav mesh and register it so agents can traverse it.
let mesh = NavMesh::new(vertices, triangles).unwrap();
world.write_resource::<NavMeshesRes>().register(mesh);
```

## Create an Agent
Agent is a type of entity that finds and follows path between two points on
navmesh. Each agent entity must have `NavAgent` component and one driver
component. Right now there is only one driver - `SimpleNavDriverTag` marks agent
to perform simple movement along path. More drivers with different behaviours
will be provided in future, but users can easly make their own drivers.

```rust
use amethyst_navigation::prelude::*;

let mut agent = NavAgent::new((x as f64, y as f64).into());
agent.speed = speed;
world
    .create_entity()
    // this entity is an agent.
    .with(agent)
    // this entity will do a simple movement along path.
    .with(SimpleNavDriverTag)
    .build();
```

## Set Agent destination

```rust
use amethyst_navigation::prelude::*;

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
```

## Custom Agent Driver
`SimpleNavDriverSystem` has the easiest movement code you can imagine. You can
make your own driver system and change agent behaviour, for example you can add
obstacle avoidance.

NOTE: Remember to create driver component tag to mark agents to use that driver.

```rust
use amethyst_navigation::prelude::*;

pub struct SimpleNavDriverSystem;

impl<'s> System<'s> for SimpleNavDriverSystem {
    type SystemData = (
        Read<'s, Time>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, SimpleNavDriverTag>,
    );

    fn run(&mut self, (time, agents, drivers): Self::SystemData) {
        let delta_time = time.delta_seconds() as f64;
        if delta_time <= 0.0 {
            return;
        }
        for (agent, _) in (&mut agents, &drivers).join() {
            if let Some(path) = agent.path() {
                if let Some((target, _)) = NavMesh::path_target_point(
                    path,
                    agent.position,
                    agent.speed.max(agent.min_target_distance.max(0.0)) * delta_time,
                ) {
                    let diff = target - agent.position;
                    let dir = diff.normalize();
                    agent.position = agent.position
                        + dir * (agent.speed.max(0.0) * delta_time).min(diff.magnitude());
                    agent.direction = diff.normalize();
                }
            }
        }
    }
}
```
