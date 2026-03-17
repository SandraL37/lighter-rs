mod engine;
pub mod types;

use crate::core::{
    arena::{
        NodeArena,
        node::{NodeData, NodeId},
    },
    layout::{engine::compute_layout, types::size::Size},
    reactive::{
        bind::{DeferredBinding, bind_field},
        dirty::DirtyFlags,
        signal::MaybeSignal,
    },
    render::Renderer,
};

pub struct LayoutContext<'a, R: Renderer> {
    pub root: NodeId,
    pub arena: &'a mut NodeArena,
    pub renderer: &'a mut R,
}

impl<'a, R: Renderer> LayoutContext<'a, R> {
    pub fn get_children(&self, node_id: impl Into<NodeId>) -> &Vec<NodeId> {
        self.arena.get_children(node_id.into()).expect(
            "Layout engine error: Malformed NodeArena. Tried to access children of a dropped node.",
        )
    }

    pub fn get_child_id(&self, parent_node_id: impl Into<NodeId>, child_index: usize) -> NodeId {
        *self.get_children(parent_node_id).get(child_index).expect(
            "Layout engine error: Malformed NodeArena. Tried to access a dropped child of a node.",
        )
    }

    pub fn get_layout(&self, node_id: impl Into<NodeId>) -> &NodeLayout {
        self.arena
            .get_layout(node_id.into())
            .expect(
                "Layout engine error: Malformed NodeArena. Tried to access the layout of a dropped node."
            )
    }

    pub fn get_layout_mut(&mut self, node_id: impl Into<NodeId>) -> &mut NodeLayout {
        self.arena
            .get_layout_mut(node_id.into())
            .expect(
                "Layout engine error: Malformed NodeArena. Tried to access the layout of a dropped node."
            )
    }

    pub fn get_data(&self, node_id: impl Into<NodeId>) -> &NodeData {
        self.arena.get_data(node_id.into()).expect(
            "Layout engine error: Malformed NodeArena. Tried to access the data of a dropped node.",
        )
    }

    pub fn compute_layout(&mut self, available_space: Size<AvailableSpace>) {
        compute_layout(self, self.root, available_space);
    }
}

#[derive(Debug, Default, Clone)]
pub struct NodeLayout {
    pub style: LayoutStyle,
    pub unrounded: UnroundedLayout,
    pub computed: ComputedLayout,
    pub cache: LayoutCache,
}

impl NodeLayout {
    pub fn new(style: LayoutStyle) -> Self {
        Self {
            style,
            unrounded: ComputedLayout::default(),
            computed: ComputedLayout::default(),
            cache: LayoutCache::default(),
        }
    }
}

pub type LayoutStyle = taffy::Style;
pub type ComputedLayout = taffy::Layout;
pub type UnroundedLayout = taffy::Layout;
pub type LayoutCache = taffy::Cache;

pub type Dimension = taffy::Dimension;
pub type DefiniteDimension = taffy::LengthPercentage;
pub type AvailableSpace = taffy::AvailableSpace;
pub type FlexDirection = taffy::FlexDirection;
pub type JustifyContent = taffy::JustifyContent;
pub type AlignItems = taffy::AlignItems;
pub type FlexWrap = taffy::FlexWrap;

/// Construct a `Dimension` as a percentage of the available space.
pub fn percent(v: f32) -> Dimension {
    taffy::style_helpers::percent(v)
}
/// Construct a `Dimension` as a fixed pixel length.
pub fn px(v: f32) -> Dimension {
    taffy::style_helpers::length(v)
}
/// Construct a `Dimension` that sizes to content (`auto`).
pub fn auto() -> Dimension {
    taffy::style_helpers::auto()
}

pub type Margin = taffy::Rect<taffy::LengthPercentageAuto>;
pub type Padding = taffy::Rect<taffy::LengthPercentage>;
pub type Gap = Size<DefiniteDimension>;

pub trait ContainerStylePropsExt: Sized {
    fn container_ctx(&mut self) -> (&mut LayoutStyle, &mut Vec<DeferredBinding>);

    fn w(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.size.width,
            bindings,
            width,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.width = val,
        );
        self
    }

    fn h(mut self, height: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.size.height,
            bindings,
            height,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.height = val,
        );
        self
    }

    fn size(mut self, size: impl Into<MaybeSignal<Dimension>>) -> Self {
        let size: MaybeSignal<Dimension> = size.into();
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.size.width,
            bindings,
            size.clone(),
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.width = val,
        );
        bind_field(
            &mut style.size.height,
            bindings,
            size,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.height = val,
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.max_size.width,
            bindings,
            max_width,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.width = val,
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.max_size.height,
            bindings,
            max_height,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.height = val,
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<MaybeSignal<Dimension>>) -> Self {
        let max_size: MaybeSignal<Dimension> = max_size.into();
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.max_size.width,
            bindings,
            max_size.clone(),
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.width = val,
        );
        bind_field(
            &mut style.max_size.height,
            bindings,
            max_size,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.height = val,
        );
        self
    }

    fn p(mut self, padding: impl Into<MaybeSignal<Padding>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.padding,
            bindings,
            padding,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.padding = val,
        );
        self
    }

    fn m(mut self, margin: impl Into<MaybeSignal<Margin>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.margin,
            bindings,
            margin,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.margin = val,
        );
        self
    }

    fn align(mut self, align_items: impl Into<MaybeSignal<AlignItems>>) -> Self {
        let value = align_items.into().map(Some);
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.align_items,
            bindings,
            value,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.align_items = val,
        );
        self
    }

    fn justify(mut self, justify_content: impl Into<MaybeSignal<JustifyContent>>) -> Self {
        let value = justify_content.into().map(Some);
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.justify_content,
            bindings,
            value,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.justify_content = val,
        );
        self
    }

    fn gap_x(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.gap.width,
            bindings,
            gap,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.gap.width = val,
        );
        self
    }

    fn gap_y(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.gap.height,
            bindings,
            gap,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.gap.height = val,
        );
        self
    }

    fn gap(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        let gap: MaybeSignal<DefiniteDimension> = gap.into();
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.gap.width,
            bindings,
            gap.clone(),
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.gap.width = val,
        );
        bind_field(
            &mut style.gap.height,
            bindings,
            gap,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.gap.height = val,
        );
        self
    }

    fn flex_direction(mut self, direction: impl Into<MaybeSignal<FlexDirection>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.flex_direction,
            bindings,
            direction,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.flex_direction = val,
        );
        self
    }

    fn flex_wrap(mut self, wrap: impl Into<MaybeSignal<FlexWrap>>) -> Self {
        let (style, bindings) = self.container_ctx();
        bind_field(
            &mut style.flex_wrap,
            bindings,
            wrap,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.flex_wrap = val,
        );
        self
    }
}

pub trait LeafStylePropsExt: Sized {
    fn leaf_ctx(&mut self) -> (&mut LayoutStyle, &mut Vec<DeferredBinding>);

    fn w(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.leaf_ctx();
        bind_field(
            &mut style.size.width,
            bindings,
            width,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.width = val,
        );
        self
    }

    fn h(mut self, height: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.leaf_ctx();
        bind_field(
            &mut style.size.height,
            bindings,
            height,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.height = val,
        );
        self
    }

    fn size(mut self, size: impl Into<MaybeSignal<Dimension>>) -> Self {
        let size: MaybeSignal<Dimension> = size.into();
        let (style, bindings) = self.leaf_ctx();
        bind_field(
            &mut style.size.width,
            bindings,
            size.clone(),
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.width = val,
        );
        bind_field(
            &mut style.size.height,
            bindings,
            size,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.size.height = val,
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.leaf_ctx();
        bind_field(
            &mut style.max_size.width,
            bindings,
            max_width,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.width = val,
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        let (style, bindings) = self.leaf_ctx();
        bind_field(
            &mut style.max_size.height,
            bindings,
            max_height,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.height = val,
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<MaybeSignal<Dimension>>) -> Self {
        let max_size: MaybeSignal<Dimension> = max_size.into();
        let (style, bindings) = self.leaf_ctx();
        bind_field(
            &mut style.max_size.width,
            bindings,
            max_size.clone(),
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.width = val,
        );
        bind_field(
            &mut style.max_size.height,
            bindings,
            max_size,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.max_size.height = val,
        );
        self
    }

    fn m(mut self, margin: impl Into<MaybeSignal<Margin>>) -> Self {
        let (style, bindings) = self.leaf_ctx();
        bind_field(
            &mut style.margin,
            bindings,
            margin,
            DirtyFlags::LAYOUT,
            |_, layout, val| layout.style.margin = val,
        );
        self
    }
}
