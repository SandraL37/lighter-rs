mod engine;
pub mod types;

use crate::core::{
    arena::{
        NodeArena,
        node::{NodeData, NodeId},
    },
    layout::{
        engine::compute_layout,
        types::{insets::Insets, size::Size},
    },
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
            style: style,
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
pub type AvailableSpace = taffy::AvailableSpace;
pub type FlexDirection = taffy::FlexDirection;
pub type JustifyContent = taffy::JustifyContent;
pub type AlignItems = taffy::AlignItems;
pub type FlexWrap = taffy::FlexWrap;

pub type Margin = Insets<Dimension>;
pub type Padding = Insets<Dimension>;
pub type Gap = Size<Dimension>;

pub trait ContainerStylePropsExt: Sized {
    fn container_style_mut(&mut self) -> &mut LayoutStyle;
    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding>;

    fn w(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            width,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.width,
            |_, layout| &mut layout.style,
        );
        self
    }

    fn h(mut self, height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            height,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.height,
            resolve_container_style,
        );
        self
    }

    fn size(mut self, size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.width,
            resolve_container_style,
        );
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.height,
            resolve_container_style,
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_width,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_container_style,
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_height,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_container_style,
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_container_style,
        );
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_container_style,
        );
        self
    }

    fn p(mut self, padding: impl Into<MaybeSignal<Padding>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            padding,
            DirtyFlags::LAYOUT,
            |style| &mut style.padding,
            resolve_container_style,
        );
        self
    }

    fn m(mut self, margin: impl Into<MaybeSignal<Margin>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            margin,
            DirtyFlags::LAYOUT,
            |style| &mut style.margin,
            resolve_container_style,
        );
        self
    }

    fn align(mut self, align_items: impl Into<MaybeSignal<AlignItems>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            align_items,
            DirtyFlags::LAYOUT,
            |style| &mut style.align_items,
            resolve_container_style,
        );
        self
    }

    fn justify(mut self, justify_content: impl Into<MaybeSignal<JustifyContent>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            justify_content,
            DirtyFlags::LAYOUT,
            |style| &mut style.justify_content,
            resolve_container_style,
        );
        self
    }

    fn gap_x(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            gap,
            DirtyFlags::LAYOUT,
            |style| &mut style.gap.width,
            resolve_container_style,
        );
        self
    }

    fn gap_y(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            gap,
            DirtyFlags::LAYOUT,
            |style| &mut style.gap.height,
            resolve_container_style,
        );
        self
    }

    fn gap(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            gap,
            DirtyFlags::LAYOUT,
            |style| &mut style.gap.width,
            resolve_container_style,
        );
        self
    }

    fn flex_direction(mut self, direction: impl Into<MaybeSignal<FlexDirection>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            direction,
            DirtyFlags::LAYOUT,
            |style| &mut style.flex_direction,
            resolve_container_style,
        );
        self
    }

    fn flex_wrap(mut self, wrap: impl Into<MaybeSignal<FlexWrap>>) -> Self {
        bind_field(
            self.container_style_mut(),
            self.bindings_mut(),
            wrap,
            DirtyFlags::LAYOUT,
            |style| &mut style.flex_wrap,
            resolve_container_style,
        );
        self
    }
}

pub trait LeafStylePropsExt: Sized {
    fn leaf_style_mut(&mut self) -> &mut LayoutStyle;
    fn bindings_mut(&mut self) -> &mut Vec<DeferredBinding>;

    fn w(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            width,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.width,
            resolve_leaf_style,
        );
        self
    }

    fn h(mut self, height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            height,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.height,
            resolve_leaf_style,
        );
        self
    }

    fn size(mut self, size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.width,
            resolve_leaf_style,
        );
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            size,
            DirtyFlags::LAYOUT,
            |style| &mut style.size.height,
            resolve_leaf_style,
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_width,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_leaf_style,
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_height,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_leaf_style,
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<MaybeSignal<Dimension>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.width,
            resolve_leaf_style,
        );
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            max_size,
            DirtyFlags::LAYOUT,
            |style| &mut style.max_size.height,
            resolve_leaf_style,
        );
        self
    }

    fn m(mut self, margin: impl Into<MaybeSignal<Margin>>) -> Self {
        bind_field(
            self.leaf_style_mut(),
            self.bindings_mut(),
            margin,
            DirtyFlags::LAYOUT,
            |style| &mut style.margin,
            resolve_leaf_style,
        );
        self
    }
}
