use slotmap::Key;
use taffy::{compute_flexbox_layout, compute_leaf_layout};

use crate::core::{
    arena::node::{NodeId, NodeKind},
    layout::{types::point::Point, *},
    reactive::dirty::DirtyFlags,
};

pub fn compute_layout<R: Renderer>(
    layout_context: &mut LayoutContext<R>,
    root: NodeId,
    available_space: Size<AvailableSpace>,
) {
    taffy::compute_root_layout(layout_context, root.into(), available_space.into());
    taffy::round_layout(layout_context, root.into());
}

impl From<NodeId> for taffy::NodeId {
    fn from(value: NodeId) -> Self {
        taffy::NodeId::from(value.data().as_ffi())
    }
}

impl From<taffy::NodeId> for NodeId {
    fn from(value: taffy::NodeId) -> Self {
        NodeId::from(slotmap::KeyData::from_ffi(value.into()))
    }
}

pub struct ChildIter<'a>(std::slice::Iter<'a, NodeId>);
impl Iterator for ChildIter<'_> {
    type Item = taffy::NodeId;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().copied().map(|node| node.into())
    }
}

const NON_EXISTENT_NODE_ID: &str =
    "Taffy Layout Engine unexpected error: impossible to get a node because it does not exist.";

impl<'a, R: Renderer> taffy::TraversePartialTree for LayoutContext<'a, R> {
    type ChildIter<'b>
        = ChildIter<'b>
    where
        Self: 'b;

    fn child_ids(&self, node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        let node = self.get_children(node_id);

        ChildIter(node.iter())
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.get_children(parent_node_id).len()
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        self.get_child_id(parent_node_id, child_index).into()
    }
}

impl<'a, R: Renderer> taffy::TraverseTree for LayoutContext<'a, R> {}

impl<'a, R: Renderer> taffy::LayoutPartialTree for LayoutContext<'a, R> {
    type CustomIdent = String;

    type CoreContainerStyle<'b>
        = &'b taffy::Style
    where
        Self: 'b;

    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        &self.get_layout(node_id).style
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.get_layout_mut(node_id).unrounded = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        taffy::compute_cached_layout(self, node_id, inputs, |layout_context, node_id, inputs| {
            layout_context
                .arena
                .mark_clean(node_id.into(), DirtyFlags::LAYOUT)
                .expect(NON_EXISTENT_NODE_ID);

            let node_kind = layout_context.get_data(node_id).kind.clone(); // TODO: check if this clone is cheap
            let style = layout_context.get_layout(node_id).style.clone(); // TODO: fix this

            match node_kind {
                NodeKind::Div(_) => compute_flexbox_layout(layout_context, node_id, inputs),
                NodeKind::Text(text_props) => compute_leaf_layout(
                    inputs,
                    &style,
                    |_val, _basis| 0.0,
                    |_known_dimensions, available_space| {
                        layout_context
                            .renderer
                            .measure_text(
                                &text_props,
                                Size::wh(
                                    available_space.width.into(),
                                    available_space.height.into(),
                                ),
                            )
                            .expect("Layout engine error while measuring text") // TODO: handle error
                            .into()
                    },
                ),
            }
        })
    }
}

impl<'a, R: Renderer> taffy::CacheTree for LayoutContext<'a, R> {
    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.get_layout(node_id)
            .cache
            .get(known_dimensions, available_space, run_mode)
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.get_layout_mut(node_id).cache.store(
            known_dimensions,
            available_space,
            run_mode,
            layout_output,
        );
    }

    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.get_layout_mut(node_id).cache.clear();
    }
}

impl<'a, R: Renderer> taffy::RoundTree for LayoutContext<'a, R> {
    fn get_unrounded_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.get_layout(node_id).unrounded
    }

    fn set_final_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.get_layout_mut(node_id).computed = *layout;
    }
}

impl<'a, R: Renderer> taffy::LayoutFlexboxContainer for LayoutContext<'a, R> {
    type FlexboxContainerStyle<'b>
        = &'b taffy::Style
    where
        Self: 'b;
    type FlexboxItemStyle<'b>
        = &'b taffy::Style
    where
        Self: 'b;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        &self.get_layout(node_id).style
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        &self.get_layout(child_node_id).style
    }
}

impl<T: Into<U>, U> From<Insets<T>> for taffy::Rect<U> {
    #[inline(always)]
    fn from(insets: Insets<T>) -> Self {
        Self {
            left: insets.left.into(),
            right: insets.right.into(),
            top: insets.top.into(),
            bottom: insets.bottom.into(),
        }
    }
}

impl<T: Into<U>, U> From<Point<T>> for taffy::Point<U> {
    #[inline(always)]
    fn from(point: Point<T>) -> Self {
        Self {
            x: point.x.into(),
            y: point.y.into(),
        }
    }
}

impl<T: Into<U>, U> From<taffy::Point<T>> for Point<U> {
    #[inline(always)]
    fn from(point: taffy::Point<T>) -> Self {
        Self {
            x: point.x.into(),
            y: point.y.into(),
        }
    }
}

impl<T: Into<U>, U> From<Size<T>> for taffy::Size<U> {
    #[inline(always)]
    fn from(rect: Size<T>) -> Self {
        Self {
            width: rect.width.into(),
            height: rect.height.into(),
        }
    }
}
