mod engine;

pub use engine::{ComputedLayout, LayoutCache, LayoutStyle, UnroundedLayout, compute_layout};

use crate::core::{
    cx::ReactivePropsExt,
    dirty::DirtyFlags,
    signal::{Reactive, ReadSignal},
};

#[derive(Debug, Default, Clone)]
pub struct NodeLayout {
    pub style: LayoutStyle,
    pub unrounded: UnroundedLayout,
    pub computed: ComputedLayout,
    pub cache: LayoutCache,
}

impl NodeLayout {
    pub fn new(style: impl Into<LayoutStyle>) -> Self {
        Self {
            style: style.into(),
            unrounded: ComputedLayout::default(),
            computed: ComputedLayout::default(),
            cache: LayoutCache::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T: Copy> Size<T> {
    pub const fn wh(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn uniform(value: T) -> Self {
        Self {
            width: value,
            height: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Insets<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Copy> Insets<T> {
    pub fn new(top: T, right: T, bottom: T, left: T) -> Self {
        Self {
            top,
            right,
            bottom,
            left,
        }
    }

    pub fn uniform(value: T) -> Self {
        Self {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect<T> {
    pub location: Point<T>,
    pub size: Size<T>,
}

impl<T: Copy> Rect<T> {
    pub fn new(location: Point<T>, size: Size<T>) -> Self {
        Self { location, size }
    }

    pub fn xywh(x: T, y: T, width: T, height: T) -> Self {
        Self {
            location: Point::new(x, y),
            size: Size::wh(width, height),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point<T> {
    pub x: T,
    pub y: T,
}

impl<T: Clone> Point<T> {
    pub fn new(x: T, y: T) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutKind {
    Container(ContainerStyle),
    Leaf(LeafStyle),
}

impl Into<LayoutKind> for ContainerStyle {
    fn into(self) -> LayoutKind {
        LayoutKind::Container(self)
    }
}

impl Into<LayoutKind> for LeafStyle {
    fn into(self) -> LayoutKind {
        LayoutKind::Leaf(self)
    }
}

pub type Margin = Insets<DefiniteDimensionAuto>;
pub type Padding = Insets<DefiniteDimension>;
pub type Gap = Size<DefiniteDimension>;

#[derive(Debug, Clone, Copy)]
pub struct ContainerStyle {
    pub size: Size<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,

    pub margin: Margin,
    pub padding: Padding,
    pub gap: Gap,

    pub flex_direction: FlexDirection,
    pub justify_content: JustifyContent,
    pub align_items: AlignItems,
    pub flex_wrap: FlexWrap,
}

impl Default for ContainerStyle {
    fn default() -> Self {
        Self {
            size: Size::wh(Dimension::Auto, Dimension::Auto),
            min_size: Size::wh(Dimension::Auto, Dimension::Auto),
            max_size: Size::wh(Dimension::Auto, Dimension::Auto),
            margin: Margin::uniform(DefiniteDimensionAuto::Pixels(0.0)),
            padding: Padding::uniform(DefiniteDimension::Pixels(0.0)),
            gap: Gap::uniform(DefiniteDimension::Pixels(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
        }
    }
}

pub trait ContainerStylePropsExt: ReactivePropsExt {
    fn container_style_mut(&mut self) -> &mut ContainerStyle;

    fn w(mut self, width: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            width,
            &mut |this, v| {
                this.container_style_mut().size.width = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.size.width = v.into();
            },
        );
        self
    }

    fn h(mut self, height: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            height,
            &mut |this, v| {
                this.container_style_mut().size.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.size.height = v.into();
            },
        );
        self
    }

    fn size(mut self, size: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            size,
            &mut |this, v| {
                this.container_style_mut().size.height = v;
                this.container_style_mut().size.width = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.size.height = v.into();
                layout.style.size.width = v.into();
            },
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            max_width,
            &mut |this, v| {
                this.container_style_mut().max_size.width = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.max_size.width = v.into();
            },
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            max_height,
            &mut |this, v| {
                this.container_style_mut().max_size.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.max_size.height = v.into();
            },
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            max_size,
            &mut |this, v| {
                this.container_style_mut().max_size.width = v;
                this.container_style_mut().max_size.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.max_size.width = v.into();
                layout.style.max_size.height = v.into();
            },
        );
        self
    }

    fn p(mut self, padding: impl Into<Reactive<Padding>>) -> Self {
        self.bind(
            padding,
            &mut |this, v| {
                this.container_style_mut().padding = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.padding = v.into();
            },
        );
        self
    }

    fn m(mut self, margin: impl Into<Reactive<Margin>>) -> Self {
        self.bind(
            margin,
            &mut |this, v| {
                this.container_style_mut().margin = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.margin = v.into();
            },
        );
        self
    }

    fn align(mut self, align_items: impl Into<Reactive<AlignItems>>) -> Self {
        self.bind(
            align_items,
            &mut |this, v| {
                this.container_style_mut().align_items = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.align_items = Some(v.into());
            },
        );
        self
    }

    fn justify(mut self, justify_content: impl Into<Reactive<JustifyContent>>) -> Self {
        self.bind(
            justify_content,
            &mut |this, v| {
                this.container_style_mut().justify_content = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.justify_content = Some(v.into());
            },
        );
        self
    }

    fn gap_x(mut self, gap: impl Into<Reactive<DefiniteDimension>>) -> Self {
        self.bind(
            gap,
            &mut |this, v| {
                this.container_style_mut().gap.width = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.gap.width = v.into();
            },
        );
        self
    }

    fn gap_y(mut self, gap: impl Into<Reactive<DefiniteDimension>>) -> Self {
        self.bind(
            gap,
            &mut |this, v| {
                this.container_style_mut().gap.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.gap.height = v.into();
            },
        );
        self
    }

    fn gap(mut self, gap: impl Into<Reactive<DefiniteDimension>>) -> Self {
        self.bind(
            gap,
            &mut |this, v| {
                this.container_style_mut().gap.width = v;
                this.container_style_mut().gap.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.gap.width = v.into();
                layout.style.gap.height = v.into();
            },
        );
        self
    }

    fn flex_direction(mut self, direction: impl Into<Reactive<FlexDirection>>) -> Self {
        self.bind(
            direction,
            &mut |this, v| {
                this.container_style_mut().flex_direction = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.flex_direction = v.into();
            },
        );
        self
    }

    fn flex_wrap(mut self, wrap: impl Into<Reactive<FlexWrap>>) -> Self {
        self.bind(
            wrap,
            &mut |this, v| {
                this.container_style_mut().flex_wrap = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.flex_wrap = v.into();
            },
        );
        self
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LeafStyle {
    pub size: Size<Dimension>,
    pub min_size: Size<Dimension>,
    pub max_size: Size<Dimension>,

    pub margin: Margin,
}

impl Default for LeafStyle {
    fn default() -> Self {
        Self {
            size: Size::wh(Dimension::Auto, Dimension::Auto),
            min_size: Size::wh(Dimension::Auto, Dimension::Auto),
            max_size: Size::wh(Dimension::Auto, Dimension::Auto),
            margin: Margin::uniform(DefiniteDimensionAuto::Pixels(0.0)),
        }
    }
}

pub trait LeafStylePropsExt: ReactivePropsExt {
    fn leaf_style_mut(&mut self) -> &mut LeafStyle;

    fn w(mut self, width: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            width,
            &mut |this, v| {
                this.leaf_style_mut().size.width = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.size.width = v.into();
            },
        );
        self
    }

    fn h(mut self, height: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            height,
            &mut |this, v| {
                this.leaf_style_mut().size.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.size.height = v.into();
            },
        );
        self
    }

    fn size(mut self, size: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            size,
            &mut |this, v| {
                this.leaf_style_mut().size.width = v;
                this.leaf_style_mut().size.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.size.width = v.into();
                layout.style.size.height = v.into();
            },
        );
        self
    }

    fn max_w(mut self, max_width: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            max_width,
            &mut |this, v| {
                this.leaf_style_mut().max_size.width = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.max_size.width = v.into();
            },
        );
        self
    }

    fn max_h(mut self, max_height: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            max_height,
            &mut |this, v| {
                this.leaf_style_mut().max_size.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.max_size.height = v.into();
            },
        );
        self
    }

    fn max_size(mut self, max_size: impl Into<Reactive<Dimension>>) -> Self {
        self.bind(
            max_size,
            &mut |this, v| {
                this.leaf_style_mut().max_size.width = v;
                this.leaf_style_mut().max_size.height = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.max_size.width = v.into();
                layout.style.max_size.height = v.into();
            },
        );
        self
    }

    fn m(mut self, margin: impl Into<Reactive<Margin>>) -> Self {
        self.bind(
            margin,
            &mut |this, v| {
                this.leaf_style_mut().margin = v;
            },
            DirtyFlags::LAYOUT,
            |_, layout, v| {
                layout.style.margin = v.into();
            },
        );
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Auto,
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimension {
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimensionAuto {
    Auto,
    Pixels(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AvailableSpace {
    Definite(f32),
    MinContent,
    MaxContent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JustifyContent {
    Start,
    End,
    Center,
    SpaceBetween,
    SpaceAround,
    SpaceEvenly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AlignItems {
    Start,
    End,
    Center,
    Stretch,
    Baseline,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlexWrap {
    NoWrap,
    Wrap,
    WrapReverse,
}

#[inline(always)]
pub fn px(value: f32) -> DefiniteDimension {
    DefiniteDimension::Pixels(value)
}

#[inline(always)]
pub fn percent(value: f32) -> DefiniteDimension {
    DefiniteDimension::Percent(value)
}

#[inline(always)]
pub fn auto() -> Dimension {
    Dimension::Auto
}

impl From<DefiniteDimension> for Dimension {
    #[inline(always)]
    fn from(d: DefiniteDimension) -> Self {
        match d {
            DefiniteDimension::Pixels(v) => Dimension::Pixels(v),
            DefiniteDimension::Percent(v) => Dimension::Percent(v),
        }
    }
}

impl From<DefiniteDimension> for DefiniteDimensionAuto {
    #[inline(always)]
    fn from(d: DefiniteDimension) -> Self {
        match d {
            DefiniteDimension::Pixels(v) => DefiniteDimensionAuto::Pixels(v),
            DefiniteDimension::Percent(v) => DefiniteDimensionAuto::Percent(v),
        }
    }
}

impl From<DefiniteDimension> for Reactive<Dimension> {
    fn from(d: DefiniteDimension) -> Self {
        Reactive::Static(d.into())
    }
}

impl From<ReadSignal<DefiniteDimension>> for Reactive<Dimension> {
    fn from(sig: ReadSignal<DefiniteDimension>) -> Self {
        Reactive::Dynamic(sig.map(|d| d.into()))
    }
}

impl From<DefiniteDimension> for Reactive<DefiniteDimensionAuto> {
    fn from(d: DefiniteDimension) -> Self {
        Reactive::Static(d.into())
    }
}

impl From<ReadSignal<DefiniteDimension>> for Reactive<DefiniteDimensionAuto> {
    fn from(sig: ReadSignal<DefiniteDimension>) -> Self {
        Reactive::Dynamic(sig.map(|d| d.into()))
    }
}
