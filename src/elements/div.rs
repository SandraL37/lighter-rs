pub mod props;
pub mod style;

use std::sync::Arc;

use crate::{
    core::{
        arena::{
            NodeArena,
            node::{EventHandlers, NodeId, NodeKind, NodeStyle, NodeStyleBuilder},
        },
        error::*,
        event::MouseEvents,
        layout::{ContainerStyleBuilder, LayoutStyle, LeafStyleBuilder},
        reactive::bind::{DeferredBinding, HasDeferredBindings},
    },
    elements::{
        Element,
        div::{
            props::DivProps,
            style::{DivStyle, DivStyleBuilder},
        },
    },
};

/// # Div
#[derive(Default)]
pub struct Div {
    props: DivProps,
    hover_props: Option<DivProps>,
    children: Vec<Box<dyn Element>>,
    event_handlers: EventHandlers,
}

impl Div {
    pub fn style(mut self, f: impl Fn(DivProps) -> DivProps) -> Self {
        self.props = f(self.props);
        self
    }

    pub fn hover(mut self, f: impl Fn(DivProps) -> DivProps) -> Self {
        if let Some(hover_props) = self.hover_props {
            self.hover_props = Some(f(hover_props));
        } else {
            self.hover_props = Some(DivProps {
                node: self.props.node.clone(),
                layout: self.props.layout.clone(),
                div: self.props.div.clone(),
                bindings: Vec::new(),
            });
        }
        self
    }
}

pub trait ChildrenExt: Sized {
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>>;

    fn child(mut self, child: impl Element + 'static) -> Self {
        self.children_mut().push(Box::new(child));
        self
    }
}

impl HasDeferredBindings for Div {
    type Style = DivProps;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.props.bindings
    }

    fn style(&mut self) -> &mut Self::Style {
        &mut self.props
    }
}

impl LeafStyleBuilder for Div {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle {
        &mut style.layout
    }
}

impl ContainerStyleBuilder for Div {}

impl NodeStyleBuilder for Div {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle {
        &mut style.node
    }
}

impl DivStyleBuilder for Div {
    fn div_style(style: &mut Self::Style) -> &mut DivStyle {
        &mut style.div
    }
}

impl ChildrenExt for Div {
    fn children_mut(&mut self) -> &mut Vec<Box<dyn Element>> {
        &mut self.children
    }
}

impl MouseEvents for Div {
    fn event_handlers(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl std::fmt::Debug for Div {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Div")
            .field("props", &self.props)
            .field("hover", &self.hover_props)
            .field("children_len", &self.children.len())
            .finish()
    }
}

impl Element for Div {
    fn build(self: Box<Self>, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let Div {
            props,
            hover_props,
            children,
            event_handlers,
        } = *self;

        let DivProps {
            node,
            layout,
            div,
            bindings,
        } = props;

        let id = arena.create_node(
            NodeKind::Div(Arc::new(div)),
            node,
            parent,
            layout,
            event_handlers,
        )?;

        for binding in bindings {
            (binding.0)(id);
        }

        for child in children {
            child.build(arena, Some(id))?;
        }

        Ok(id)
    }
}

/// # Div
pub fn div() -> Div {
    Div {
        props: DivProps::default(),
        hover_props: Some(DivProps::default()),
        children: Vec::new(),
        event_handlers: EventHandlers::default(),
    }
}
