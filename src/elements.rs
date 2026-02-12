use crate::core::{error::*, node::NodeId, tree::BuildCtx};

pub mod div;
pub mod text;

#[derive(Debug)]
pub enum ElementKind {
    Div(div::Div),
    Text(text::Text),
}

impl ElementKind {
    pub fn build(self, ctx: &mut BuildCtx, parent: Option<NodeId>) -> Result<NodeId> {
        match self {
            ElementKind::Div(div) => div.build(ctx, parent),
            ElementKind::Text(text) => text.build(ctx, parent),
        }
    }
}

pub trait ElementBuild {
    fn build(self, ctx: &mut BuildCtx, parent: Option<NodeId>) -> Result<NodeId>;
}

pub trait Element: Sized + ElementBuild + Into<ElementKind> {}
