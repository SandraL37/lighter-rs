use std::sync::Arc;

use crate::{
    core::{cx::ReactivePropsExt, dirty::DirtyFlags, signal::Reactive, style::Transform},
    elements::{div::DivProps, text::TextProps},
};

slotmap::new_key_type! {
    pub struct NodeId;
}

#[derive(Debug)]
pub struct NodeData {
    pub kind: NodeKind,
    pub props: NodeProps,
    pub dirty: DirtyFlags,
}

#[derive(Debug, Clone)]
pub enum NodeKind {
    Div(Arc<DivProps>),
    Text(Arc<TextProps>),
}

impl NodeKind {
    pub fn as_div_mut(&mut self) -> &mut DivProps {
        if let NodeKind::Div(props) = self {
            Arc::make_mut(props)
        } else {
            unreachable!("Not a div"); // TODO: check correctness
        }
    }

    pub fn as_text_mut(&mut self) -> &mut TextProps {
        if let NodeKind::Text(props) = self {
            Arc::make_mut(props)
        } else {
            unreachable!("Not a text");
        }
    }
}

//  TODO: bench this
#[derive(Debug)]
pub struct NodeProps {
    pub opacity: f32,
    pub z_index: i32,
    pub transform: Option<Transform>,
}

impl Default for NodeProps {
    fn default() -> Self {
        NodeProps {
            opacity: 1.0,
            z_index: 0,
            transform: None, // TODO: benchmark if Box<Transform> is faster
        }
    }
}

pub trait NodePropsExt: ReactivePropsExt {
    fn node_props_mut(&mut self) -> &mut NodeProps;

    fn opacity(mut self, value: impl Into<Reactive<f32>>) -> Self {
        self.bind(
            value,
            &mut |this, v| {
                this.node_props_mut().opacity = v;
            },
            DirtyFlags::PAINT,
            |data, _, v| {
                data.props.opacity = v;
            },
        );
        self
    }

    fn z(mut self, value: impl Into<Reactive<i32>>) -> Self {
        self.bind(
            value,
            &mut |this, v| {
                this.node_props_mut().z_index = v;
            },
            DirtyFlags::PAINT,
            |data, _, v| {
                data.props.z_index = v;
            },
        );
        self
    }

    fn transform(mut self, value: impl Into<Reactive<Transform>>) -> Self {
        self.bind(
            value,
            &mut |this, v| {
                this.node_props_mut().transform = Some(v);
            },
            DirtyFlags::PAINT,
            |data, _, v| {
                data.props.transform = Some(v);
            },
        );
        self
    }
}
