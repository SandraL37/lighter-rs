use std::sync::Arc;

use crate::{
    core::{
        arena::NodeArena,
        cx::{Cx, DeferredBinding, ReactivePropsExt},
        dirty::DirtyFlags,
        error::*,
        layout::{ContainerStyle, ContainerStylePropsExt},
        node::{NodeId, NodeKind, NodeProps, NodePropsExt},
        signal::Reactive,
        style::Color,
    },
    elements::Element,
};

#[derive(Debug)]
pub struct Div {
    node_props: NodeProps,
    layout_props: ContainerStyle,
    div_props: DivProps,
    children: Vec<Box<dyn Element>>,
    deferred_bindings: Vec<DeferredBinding>,
}

pub trait ChildrenExt: Sized {
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>>;

    fn child(mut self, child: impl Element + 'static) -> Self {
        self.children_mut().push(Box::new(child));
        self
    }
}

pub trait DivPropsExt:
    ReactivePropsExt + ContainerStylePropsExt + ChildrenExt + NodePropsExt
{
    fn div_props_mut(&mut self) -> &mut DivProps;

    fn bg(mut self, color: impl Into<Reactive<Color>>) -> Self {
        self.bind(
            color,
            &mut |div, color| div.div_props_mut().background_color = color,
            DirtyFlags::PAINT,
            |node, _, color| node.kind.as_div_mut().background_color = color,
        );
        self
    }

    fn rounded(mut self, radius: impl Into<Reactive<f32>>) -> Self {
        self.bind(
            radius,
            &mut |div, radius| div.div_props_mut().corner_radius = radius,
            DirtyFlags::PAINT,
            |node, _, radius| node.kind.as_div_mut().corner_radius = radius,
        );

        self
    }
}

#[derive(Debug, Clone)]
pub struct DivProps {
    pub background_color: Color,
    pub corner_radius: f32, // TODO: make it DefiniteDimension
}

impl Default for DivProps {
    fn default() -> Self {
        DivProps {
            background_color: Color::TRANSPARENT,
            corner_radius: 0.0,
        }
    }
}

impl Element for Div {
    fn build(
        self: Box<Self>,
        arena: &mut NodeArena,
        cx: &mut Cx,
        parent: Option<NodeId>,
    ) -> Result<NodeId> {
        let id = arena.create_node(
            NodeKind::Div(Arc::new(self.div_props)),
            self.node_props,
            parent,
            self.layout_props,
        )?;

        for binding in self.deferred_bindings {
            (binding.0)(id, cx)
        }

        for child in self.children {
            child.build(arena, cx, Some(id))?;
        }

        Ok(id)
    }
}

impl ReactivePropsExt for Div {
    fn deferred_bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
    }
}

impl ContainerStylePropsExt for Div {
    fn container_style_mut(&mut self) -> &mut ContainerStyle {
        &mut self.layout_props
    }
}

impl NodePropsExt for Div {
    fn node_props_mut(&mut self) -> &mut NodeProps {
        &mut self.node_props
    }
}

impl DivPropsExt for Div {
    fn div_props_mut(&mut self) -> &mut DivProps {
        &mut self.div_props
    }
}

impl ChildrenExt for Div {
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }
}

pub fn div() -> Div {
    Div {
        node_props: NodeProps::default(),
        layout_props: ContainerStyle::default(),
        children: Vec::new(),
        div_props: DivProps::default(),
        deferred_bindings: Vec::new(),
    }
}
