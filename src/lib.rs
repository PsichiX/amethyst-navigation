extern crate oxygengine_navigation as nav;

pub mod systems;

pub mod prelude {
    pub use crate::{components::*, resources::*, systems::*};
}

pub mod components {
    pub use nav::component::*;
}

pub mod resources {
    pub use nav::resource::*;
}
