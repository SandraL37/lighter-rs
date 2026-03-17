use std::sync::Arc;

use crate::{
    core::{
        arena::{
            NodeArena,
            node::{EventHandlers, NodeData, NodeId, NodeKind, NodeProps, NodePropsExt},
        },
        error::*,
        event::MouseEvents,
        layout::{ContainerStylePropsExt, LayoutStyle, NodeLayout},
        reactive::{
            bind::{DeferredBinding, bind_field},
            dirty::DirtyFlags,
            signal::MaybeSignal,
        },
        style::Color,
    },
    elements::Element,
};

#[derive(Debug)]
pub struct Div {
    node_props: NodeProps,
    layout_props: LayoutStyle,
    div_props: DivProps,
    children: Vec<Box<dyn Element>>,
    deferred_bindings: Vec<DeferredBinding>,
    event_handlers: EventHandlers,
}

pub trait ChildrenExt: Sized {
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>>;

    fn child(mut self, child: impl Element + 'static) -> Self {
        self.children_mut().push(Box::new(child));
        self
    }
}

fn resolve_div<'a>(data: &'a mut NodeData, layout: &'a mut NodeLayout) -> &'a mut DivProps {
    match &mut data.kind {
        NodeKind::Div(props) => Arc::make_mut(props).expect("multiple references to div props"),
        _ => unreachable!(),
    }
}

pub trait DivPropsExt: Sized {
    fn div_props_mut(&mut self) -> &mut DivProps;
    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding>;

    fn bg(mut self, color: impl Into<MaybeSignal<Color>>) -> Self {
        bind_field(
            self.div_props_mut(),
            self.bindings_mut(),
            color,
            DirtyFlags::PAINT,
            |props| &mut props.background_color,
            resolve_div,
        );
        self
    }

    fn rounded(mut self, radius: impl Into<MaybeSignal<f32>>) -> Self {
        bind_field(
            self.div_props_mut(),
            self.bindings_mut(),
            radius,
            DirtyFlags::PAINT,
            |p| &mut p.corner_radius,
            resolve_div,
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
    fn build(self: Box<Self>, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let id = arena.create_node(
            NodeKind::Div(Arc::new(self.div_props)),
            self.node_props,
            parent,
            self.layout_props,
            self.event_handlers,
        )?;

        for binding in self.deferred_bindings {
            (binding.0)(id);
        }

        for child in self.children {
            child.build(arena, Some(id))?;
        }

        Ok(id)
    }
}

impl ContainerStylePropsExt for Div {
    fn container_style_mut(&mut self) -> &mut LayoutStyle {
        &mut self.layout_props
    }

    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
    }
}

impl NodePropsExt for Div {
    fn node_props_mut(&mut self) -> &mut NodeProps {
        &mut self.node_props
    }

    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
    }
}

impl DivPropsExt for Div {
    fn div_props_mut(&mut self) -> &mut DivProps {
        &mut self.div_props
    }

    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.deferred_bindings
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
        layout_props: LayoutStyle::default(),
        children: Vec::new(),
        div_props: DivProps::default(),
        deferred_bindings: Vec::new(),
        event_handlers: EventHandlers::default(),
    }
}

impl MouseEvents for Div {
    fn event_handlers(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}
