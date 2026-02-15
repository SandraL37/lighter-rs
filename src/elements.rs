use crate::core::{arena::NodeArena, error::*, node::NodeId};

pub mod div;
pub mod text;

#[derive(Debug)]
pub enum ElementKind {
    Div(div::Div),
    Text(text::Text),
}

impl ElementKind {
    pub fn build(self, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        match self {
            ElementKind::Div(div) => div.build(arena, parent),
            ElementKind::Text(text) => text.build(arena, parent),
        }
    }
}

pub trait ElementBuild {
    fn build(self, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId>;
}

pub trait Element: Sized + ElementBuild + Into<ElementKind> {}
