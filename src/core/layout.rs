mod engine;
pub mod types;

use crate::core::{
    arena::{
        NodeArena,
        node::{NodeData, NodeId},
    },
    layout::{
        engine::compute_layout,
        types::{
            alignment::{AlignItems, JustifyContent},
            dimension::{DefiniteDimension, DefiniteDimensionAuto, Dimension},
            flex::{FlexDirection, FlexWrap},
            size::Size,
        },
    },
    reactive::{bind::HasDeferredBindings, dirty::DirtyFlags, signal::MaybeSignal},
    render::Renderer,
};

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

pub type AvailableSpace = taffy::AvailableSpace;

pub trait ContainerStyleBuilder: LeafStyleBuilder {
    fn pb(mut self, bottom: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).padding.bottom,
            bottom,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.padding.bottom = val;
            },
        );
        self
    }

    fn pt(mut self, top: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).padding.top,
            top,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.padding.top = val;
            },
        );
        self
    }

    fn pl(mut self, left: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).padding.left,
            left,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.padding.left = val;
            },
        );
        self
    }

    fn pr(mut self, right: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).padding.right,
            right,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.padding.right = val;
            },
        );
        self
    }

    #[inline]
    fn py(self, padding: impl Into<MaybeSignal<DefiniteDimension>> + Copy) -> Self {
        self.pt(padding).pb(padding)
    }

    #[inline]
    fn px(self, padding: impl Into<MaybeSignal<DefiniteDimension>> + Copy) -> Self {
        self.pl(padding).pr(padding)
    }

    #[inline]
    fn p(self, padding: impl Into<MaybeSignal<DefiniteDimension>> + Copy) -> Self {
        self.px(padding).py(padding)
    }

    fn align(mut self, align_items: impl Into<MaybeSignal<Option<AlignItems>>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).align_items,
            align_items,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.align_items = val;
            },
        );
        self
    }

    #[inline(always)]
    fn items_start(self) -> Self {
        self.align(Some(AlignItems::Start))
    }

    #[inline(always)]
    fn items_end(self) -> Self {
        self.align(Some(AlignItems::End))
    }

    #[inline(always)]
    fn items_center(self) -> Self {
        self.align(Some(AlignItems::Center))
    }

    fn justify(mut self, justify_content: impl Into<MaybeSignal<Option<JustifyContent>>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).justify_content,
            justify_content,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.justify_content = val;
            },
        );
        self
    }

    // TODO: Cover all cases
    #[inline(always)]
    fn justify_start(self) -> Self {
        self.justify(Some(JustifyContent::Start))
    }

    #[inline(always)]
    fn justify_end(self) -> Self {
        self.justify(Some(JustifyContent::End))
    }

    #[inline(always)]
    fn justify_center(self) -> Self {
        self.justify(Some(JustifyContent::Center))
    }

    fn gap_x(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).gap.width,
            gap,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.gap.width = val;
            },
        );
        self
    }

    fn gap_y(mut self, gap: impl Into<MaybeSignal<DefiniteDimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).gap.height,
            gap,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.gap.height = val;
            },
        );
        self
    }

    #[inline]
    fn gap(self, gap: impl Into<MaybeSignal<DefiniteDimension>> + Copy) -> Self {
        self.gap_x(gap).gap_y(gap)
    }

    fn flex_direction(mut self, direction: impl Into<MaybeSignal<FlexDirection>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).flex_direction,
            direction,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.flex_direction = val;
            },
        );
        self
    }

    fn flex_wrap(mut self, wrap: impl Into<MaybeSignal<FlexWrap>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).flex_wrap,
            wrap,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.flex_wrap = val;
            },
        );
        self
    }
}

pub trait LeafStyleBuilder: HasDeferredBindings + Sized {
    fn layout_style(style: &mut Self::Style) -> &mut LayoutStyle;

    fn w(mut self, width: impl Into<MaybeSignal<Dimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).size.width,
            width,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.size.width = val;
            },
        );
        self
    }

    fn h(mut self, height: impl Into<MaybeSignal<Dimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).size.height,
            height,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.size.height = val;
            },
        );
        self
    }

    #[inline]
    fn size(self, size: impl Into<MaybeSignal<Dimension>> + Copy) -> Self {
        self.w(size).h(size)
    }

    fn max_w(mut self, max_width: impl Into<MaybeSignal<Dimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).max_size.width,
            max_width,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.max_size.width = val;
            },
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<MaybeSignal<Dimension>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).max_size.height,
            max_height,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.max_size.height = val;
            },
        );
        self
    }

    fn max_size(self, max_size: impl Into<MaybeSignal<Dimension>> + Copy) -> Self {
        self.max_w(max_size).max_h(max_size)
    }

    fn mb(mut self, bottom: impl Into<MaybeSignal<DefiniteDimensionAuto>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).margin.bottom,
            bottom,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.margin.bottom = val;
            },
        );
        self
    }

    fn mt(mut self, top: impl Into<MaybeSignal<DefiniteDimensionAuto>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).margin.top,
            top,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.margin.top = val;
            },
        );
        self
    }

    fn ml(mut self, left: impl Into<MaybeSignal<DefiniteDimensionAuto>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).margin.left,
            left,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.margin.left = val;
            },
        );
        self
    }

    fn mr(mut self, right: impl Into<MaybeSignal<DefiniteDimensionAuto>>) -> Self {
        self.bind(
            |style| &mut Self::layout_style(style).margin.right,
            right,
            DirtyFlags::LAYOUT,
            |_, layout, val| {
                layout.style.margin.right = val;
            },
        );
        self
    }

    #[inline]
    fn my(self, margin: impl Into<MaybeSignal<DefiniteDimensionAuto>> + Copy) -> Self {
        self.mt(margin).mb(margin)
    }

    #[inline]
    fn mx(self, margin: impl Into<MaybeSignal<DefiniteDimensionAuto>> + Copy) -> Self {
        self.ml(margin).mr(margin)
    }

    #[inline]
    fn m(self, margin: impl Into<MaybeSignal<DefiniteDimensionAuto>> + Copy) -> Self {
        self.mx(margin).my(margin)
    }
}

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
