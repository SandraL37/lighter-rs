use slotmap::Key;
use taffy::{compute_flexbox_layout, compute_leaf_layout};

use crate::core::{
    layout::*,
    node::{Node, NodeId, NodeKind},
    tree::NodeArena,
};

pub type EngineStyle = taffy::Style;
pub type EngineLayout = taffy::Layout;
pub type EngineCache = taffy::Cache;

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

impl NodeArena {
    fn get_node_unchecked(&self, node_id: taffy::NodeId) -> &Node {
        self.get_node(node_id.into()).expect(NON_EXISTENT_NODE_ID)
    }

    fn get_node_mut_unchecked(&mut self, node_id: taffy::NodeId) -> &mut Node {
        self.get_node_mut(node_id.into())
            .expect(NON_EXISTENT_NODE_ID)
    }
}

impl taffy::TraversePartialTree for NodeArena {
    type ChildIter<'a> = ChildIter<'a>;

    fn child_ids(&self, node_id: taffy::NodeId) -> Self::ChildIter<'_> {
        let node = self.get_node_unchecked(node_id);

        ChildIter(node.children.iter())
    }

    fn child_count(&self, parent_node_id: taffy::NodeId) -> usize {
        self.get_node_unchecked(parent_node_id).children.len()
    }

    fn get_child_id(&self, parent_node_id: taffy::NodeId, child_index: usize) -> taffy::NodeId {
        self.get_node_unchecked(parent_node_id)
            .children
            .get(child_index)
            .copied()
            .map(|child| child.into())
            .expect(NON_EXISTENT_NODE_ID)
    }
}

impl taffy::TraverseTree for NodeArena {}

impl taffy::LayoutPartialTree for NodeArena {
    type CustomIdent = String;

    type CoreContainerStyle<'a> = &'a taffy::Style;

    fn get_core_container_style(&self, node_id: taffy::NodeId) -> Self::CoreContainerStyle<'_> {
        &self.get_node_unchecked(node_id).layout.style
    }

    fn set_unrounded_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.get_node_mut_unchecked(node_id).layout.unrounded = *layout;
    }

    fn compute_child_layout(
        &mut self,
        node_id: taffy::NodeId,
        inputs: taffy::LayoutInput,
    ) -> taffy::LayoutOutput {
        taffy::compute_cached_layout(self, node_id, inputs, |tree, node_id, inputs| {
            let node = tree.get_node_unchecked(node_id);

            match &node.kind {
                NodeKind::Div(_) => compute_flexbox_layout(tree, node_id, inputs),
                NodeKind::Text(text_props) => {
                    compute_leaf_layout(
                        inputs,
                        &node.layout.style,
                        |_val, _basis| 0.0,
                        |_known_dimensions, _available_space| {
                            taffy::Size::length(text_props.content().len() as f32 * 10.0) // Placeholder text size
                        },
                    )
                }
            }
        })
    }
}

impl taffy::CacheTree for NodeArena {
    fn cache_clear(&mut self, node_id: taffy::NodeId) {
        self.get_node_mut_unchecked(node_id).layout.cache.clear();
    }

    fn cache_get(
        &self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
    ) -> Option<taffy::LayoutOutput> {
        self.get_node_unchecked(node_id).layout.cache.get(
            known_dimensions,
            available_space,
            run_mode,
        )
    }

    fn cache_store(
        &mut self,
        node_id: taffy::NodeId,
        known_dimensions: taffy::Size<Option<f32>>,
        available_space: taffy::Size<taffy::AvailableSpace>,
        run_mode: taffy::RunMode,
        layout_output: taffy::LayoutOutput,
    ) {
        self.get_node_mut_unchecked(node_id).layout.cache.store(
            known_dimensions,
            available_space,
            run_mode,
            layout_output,
        );
    }
}

impl taffy::RoundTree for NodeArena {
    fn get_unrounded_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.get_node_unchecked(node_id).layout.unrounded
    }

    fn set_final_layout(&mut self, node_id: taffy::NodeId, layout: &taffy::Layout) {
        self.get_node_mut_unchecked(node_id).layout.computed = *layout;
    }
}

impl taffy::PrintTree for NodeArena {
    fn get_debug_label(&self, node_id: taffy::NodeId) -> &'static str {
        match self.get_node_unchecked(node_id).kind {
            NodeKind::Div(_) => "Div",
            NodeKind::Text(_) => "Text",
        }
    }

    fn get_final_layout(&self, node_id: taffy::NodeId) -> taffy::Layout {
        self.get_node_unchecked(node_id).layout.computed
    }
}

impl taffy::LayoutFlexboxContainer for NodeArena {
    type FlexboxContainerStyle<'a> = &'a taffy::Style;
    type FlexboxItemStyle<'a> = &'a taffy::Style;

    fn get_flexbox_container_style(
        &self,
        node_id: taffy::NodeId,
    ) -> Self::FlexboxContainerStyle<'_> {
        &self.get_node_unchecked(node_id).layout.style
    }

    fn get_flexbox_child_style(&self, child_node_id: taffy::NodeId) -> Self::FlexboxItemStyle<'_> {
        &self.get_node_unchecked(child_node_id).layout.style
    }
}

impl From<Dimension> for taffy::Dimension {
    #[inline(always)]
    fn from(dimension: Dimension) -> Self {
        match dimension {
            Dimension::Auto => taffy::Dimension::auto(),
            Dimension::Points(points) => taffy::Dimension::length(points),
            Dimension::Percent(percent) => taffy::Dimension::percent(percent),
        }
    }
}

impl From<DefiniteDimension> for taffy::LengthPercentage {
    #[inline(always)]
    fn from(dimension: DefiniteDimension) -> Self {
        match dimension {
            DefiniteDimension::Points(points) => taffy::LengthPercentage::length(points),
            DefiniteDimension::Percent(percent) => taffy::LengthPercentage::percent(percent),
        }
    }
}

impl From<DefiniteDimensionAuto> for taffy::LengthPercentageAuto {
    #[inline(always)]
    fn from(dimension: DefiniteDimensionAuto) -> Self {
        match dimension {
            DefiniteDimensionAuto::Auto => taffy::LengthPercentageAuto::auto(),
            DefiniteDimensionAuto::Points(points) => taffy::LengthPercentageAuto::length(points),
            DefiniteDimensionAuto::Percent(percent) => {
                taffy::LengthPercentageAuto::percent(percent)
            }
        }
    }
}

impl<T: Into<U>, U> From<Rect<T>> for taffy::Rect<U> {
    #[inline(always)]
    fn from(rect: Rect<T>) -> Self {
        Self {
            left: rect.left.into(),
            right: rect.right.into(),
            top: rect.top.into(),
            bottom: rect.bottom.into(),
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

impl From<FlexDirection> for taffy::FlexDirection {
    #[inline(always)]
    fn from(direction: FlexDirection) -> Self {
        match direction {
            FlexDirection::Row => taffy::FlexDirection::Row,
            FlexDirection::Column => taffy::FlexDirection::Column,
            FlexDirection::RowReverse => taffy::FlexDirection::RowReverse,
            FlexDirection::ColumnReverse => taffy::FlexDirection::ColumnReverse,
        }
    }
}

impl From<JustifyContent> for taffy::JustifyContent {
    #[inline(always)]
    fn from(justify_content: JustifyContent) -> Self {
        match justify_content {
            JustifyContent::Start => taffy::JustifyContent::FlexStart,
            JustifyContent::Center => taffy::JustifyContent::Center,
            JustifyContent::End => taffy::JustifyContent::FlexEnd,
            JustifyContent::SpaceBetween => taffy::JustifyContent::SpaceBetween,
            JustifyContent::SpaceAround => taffy::JustifyContent::SpaceAround,
            JustifyContent::SpaceEvenly => taffy::JustifyContent::SpaceEvenly,
        }
    }
}

impl From<AlignItems> for taffy::AlignItems {
    #[inline(always)]
    fn from(align_items: AlignItems) -> Self {
        match align_items {
            AlignItems::Start => taffy::AlignItems::FlexStart,
            AlignItems::Center => taffy::AlignItems::Center,
            AlignItems::End => taffy::AlignItems::FlexEnd,
            AlignItems::Stretch => taffy::AlignItems::Stretch,
            AlignItems::Baseline => taffy::AlignItems::Baseline,
        }
    }
}

impl From<FlexWrap> for taffy::FlexWrap {
    #[inline(always)]
    fn from(flex_wrap: FlexWrap) -> Self {
        match flex_wrap {
            FlexWrap::NoWrap => taffy::FlexWrap::NoWrap,
            FlexWrap::Wrap => taffy::FlexWrap::Wrap,
            FlexWrap::WrapReverse => taffy::FlexWrap::WrapReverse,
        }
    }
}

impl From<LayoutStyle> for taffy::Style {
    fn from(style: LayoutStyle) -> Self {
        match style {
            LayoutStyle::Container(container) => Self {
                size: container.size.into(),
                min_size: container.min_size.into(),
                max_size: container.max_size.into(),
                padding: container.padding.into(),
                margin: container.margin.into(),
                gap: container.gap.into(),
                flex_direction: container.flex_direction.into(),
                justify_content: Some(container.justify_content.into()), // TODO: check if this should be made option or not
                align_items: Some(container.align_items.into()),
                flex_wrap: container.flex_wrap.into(),
                ..Default::default()
            },
            LayoutStyle::Leaf(leaf) => Self {
                size: leaf.size.into(),
                min_size: leaf.min_size.into(),
                max_size: leaf.max_size.into(),
                margin: leaf.margin.into(),
                ..Default::default()
            },
        }
    }
}
