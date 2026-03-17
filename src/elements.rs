use std::fmt::Debug;

use crate::core::{
    arena::{NodeArena, node::NodeId},
    error::*,
};

pub mod div;
pub mod text;

pub trait Element: Debug {
    fn build(
        self: Box<Self>,
        arena: &mut NodeArena,
        parent: Option<NodeId>,
    ) -> Result<NodeId>;
}
