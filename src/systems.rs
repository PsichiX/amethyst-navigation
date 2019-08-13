use amethyst_core::{
    ecs::{Read, ReadStorage, System, WriteStorage},
    Time,
};
pub use nav::system::NavAgentMaintainSystem;
use nav::{
    component::{NavAgent, SimpleNavDriverTag},
    system::SimpleNavDriverSystem as NavSimpleNavDriverSystem,
};

/// Simple nav driver system. It's used to apply simple movement of agents with `SimpleNavDriverTag`
/// component tag on their paths.
pub struct SimpleNavDriverSystem;

impl<'s> System<'s> for SimpleNavDriverSystem {
    type SystemData = (
        Read<'s, Time>,
        WriteStorage<'s, NavAgent>,
        ReadStorage<'s, SimpleNavDriverTag>,
    );

    fn run(&mut self, (time, agents, drivers): Self::SystemData) {
        NavSimpleNavDriverSystem::run_impl(time.delta_seconds() as f64, (agents, drivers));
    }
}
