use std::fmt::Debug;

use crate::core::{
    arena::{NodeArena, node::NodeId},
    error::*,
    reactive::cx::Cx,
};

pub mod div;
pub mod text;

pub trait Element: Debug {
    fn build(
        self: Box<Self>,
        arena: &mut NodeArena,
        cx: &mut Cx,
        parent: Option<NodeId>,
    ) -> Result<NodeId>;
}
