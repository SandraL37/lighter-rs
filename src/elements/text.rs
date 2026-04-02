pub mod props;
pub mod style;
pub mod textcontent;

use std::sync::Arc;

use crate::{
    core::{
        arena::{
            NodeArena,
            node::{EventHandlers, NodeId, NodeKind, NodeStyle, NodeStyleBuilder},
        },
        error::*,
        event::MouseEvents,
        layout::{LayoutStyle, LeafStyleBuilder},
        reactive::bind::{DeferredBinding, HasDeferredBindings},
    },
    elements::{
        Element,
        text::{
            props::TextBuildProps,
            style::{TextStyle, TextStyleBuilder},
            textcontent::IntoTextContent,
        },
    },
};

#[derive(Debug)]
pub struct Text {
    props: TextBuildProps,
    event_handlers: EventHandlers,
}

impl HasDeferredBindings for Text {
    type Style = TextBuildProps;

    fn bindings(&mut self) -> &mut Vec<DeferredBinding> {
        &mut self.props.bindings
    }

    fn style(&mut self) -> &mut Self::Style {
        &mut self.props
    }
}

impl LeafStyleBuilder for Text {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle {
        &mut style.layout
    }
}

impl NodeStyleBuilder for Text {
    fn node_style(style: &mut Self::Style) -> &mut NodeStyle {
        &mut style.node
    }
}

impl TextStyleBuilder for Text {
    fn text_style(style: &mut Self::Style) -> &mut TextStyle {
        &mut style.text
    }
}

impl MouseEvents for Text {
    fn event_handlers(&mut self) -> &mut EventHandlers {
        &mut self.event_handlers
    }
}

impl Element for Text {
    fn build(self: Box<Self>, arena: &mut NodeArena, parent: Option<NodeId>) -> Result<NodeId> {
        let Text {
            props,
            event_handlers,
        } = *self;

        let TextBuildProps {
            node,
            layout,
            text,
            bindings,
        } = props;

        let id = arena.create_node(
            NodeKind::Text(Arc::new(text)),
            node,
            parent,
            layout,
            event_handlers,
        )?;

        for binding in bindings {
            (binding.0)(id);
        }

        Ok(id)
    }
}

pub fn text(content: impl IntoTextContent) -> Text {
    (Text {
        props: TextBuildProps::default(),
        event_handlers: EventHandlers::default(),
    })
    .content(content)
}
