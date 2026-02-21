use std::fmt::Debug;

use crate::core::{arena::NodeArena, cx::Cx, error::*, node::NodeId};

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
