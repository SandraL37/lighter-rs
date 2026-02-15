use crate::{
    core::{dirty::DirtyFlags, layout::NodeLayout, style::Transform},
    elements::{div::DivProps, text::TextProps},
};

slotmap::new_key_type! {
    pub struct NodeId;
}

#[derive(Debug)]
pub struct Node {
    pub kind: NodeKind,

    pub props: NodeProps,

    pub children: Vec<NodeId>,
    pub parent: Option<NodeId>,

    pub dirty: DirtyFlags,

    pub layout: NodeLayout,
}

#[derive(Debug)]
pub enum NodeKind {
    Div(DivProps),
    Text(TextProps),
}

#[derive(Debug)]
pub struct NodeProps {
    pub opacity: f32,
    pub z_index: i32,
    pub transform: Transform,
}

impl Default for NodeProps {
    fn default() -> Self {
        NodeProps {
            opacity: 1.0,
            z_index: 0,
            transform: Transform::IDENTITY,
        }
    }
}

pub trait NodePropsExt: Sized {
    fn props_mut(&mut self) -> &mut NodeProps;

    fn opacity(mut self, value: f32) -> Self {
        self.props_mut().opacity = value;
        self
    }

    fn z(mut self, z: i32) -> Self {
        self.props_mut().z_index = z;
        self
    }

    fn transform(mut self, t: Transform) -> Self {
        self.props_mut().transform = t;
        self
    }
}
