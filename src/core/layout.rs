use crate::core::layout::engine::{EngineCache, EngineLayout, EngineStyle};

pub mod engine;

#[derive(Debug, Default, Clone)]
pub struct NodeLayout {
    pub style: EngineStyle,
    pub unrounded: EngineLayout,
    pub computed: EngineLayout,
    pub cache: EngineCache,
}

impl NodeLayout {
    pub fn new(style: impl Into<EngineStyle>) -> Self {
        Self {
            style: style.into(),
            unrounded: EngineLayout::default(),
            computed: EngineLayout::default(),
            cache: EngineCache::default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Size<T> {
    pub width: T,
    pub height: T,
}

impl<T: Clone> Size<T> {
    pub const fn wh(width: T, height: T) -> Self {
        Self { width, height }
    }

    pub fn uniform(value: T) -> Self {
        Self {
            width: value.clone(),
            height: value,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect<T> {
    pub top: T,
    pub right: T,
    pub bottom: T,
    pub left: T,
}

impl<T: Clone> Rect<T> {
    pub fn uniform(value: T) -> Self {
        Self {
            top: value.clone(),
            right: value.clone(),
            bottom: value.clone(),
            left: value,
        }
    }
}

#[derive(Debug, Clone)]
pub enum LayoutStyle {
    Container(ContainerStyle),
    Leaf(LeafStyle),
}

impl Into<LayoutStyle> for ContainerStyle {
    fn into(self) -> LayoutStyle {
        LayoutStyle::Container(self)
    }
}

impl Into<LayoutStyle> for LeafStyle {
    fn into(self) -> LayoutStyle {
        LayoutStyle::Leaf(self)
    }
}

pub type Margin = Rect<DefiniteDimensionAuto>;
pub type Padding = Rect<DefiniteDimension>;
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
            margin: Margin::uniform(DefiniteDimensionAuto::Points(0.0)),
            padding: Padding::uniform(DefiniteDimension::Points(0.0)),
            gap: Gap::uniform(DefiniteDimension::Points(0.0)),
            flex_direction: FlexDirection::Row,
            justify_content: JustifyContent::Start,
            align_items: AlignItems::Stretch,
            flex_wrap: FlexWrap::NoWrap,
        }
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
            margin: Margin::uniform(DefiniteDimensionAuto::Points(0.0)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Dimension {
    Auto,
    Points(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimension {
    Points(f32),
    Percent(f32),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DefiniteDimensionAuto {
    Auto,
    Points(f32),
    Percent(f32),
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

pub trait PointsDimension {
    fn points(value: f32) -> Self;
}

impl PointsDimension for Dimension {
    #[inline(always)]
    fn points(value: f32) -> Self {
        Dimension::Points(value)
    }
}

impl PointsDimension for DefiniteDimension {
    #[inline(always)]
    fn points(value: f32) -> Self {
        DefiniteDimension::Points(value)
    }
}

impl PointsDimension for DefiniteDimensionAuto {
    #[inline(always)]
    fn points(value: f32) -> Self {
        DefiniteDimensionAuto::Points(value)
    }
}

pub fn px<T: PointsDimension>(value: f32) -> T {
    T::points(value)
}

pub trait PercentDimension {
    fn percent(value: f32) -> Self;
}

impl PercentDimension for Dimension {
    #[inline(always)]
    fn percent(value: f32) -> Self {
        Dimension::Percent(value)
    }
}

impl PercentDimension for DefiniteDimension {
    #[inline(always)]
    fn percent(value: f32) -> Self {
        DefiniteDimension::Percent(value)
    }
}

impl PercentDimension for DefiniteDimensionAuto {
    #[inline(always)]
    fn percent(value: f32) -> Self {
        DefiniteDimensionAuto::Percent(value)
    }
}

pub fn percent<T: PercentDimension>(value: f32) -> T {
    T::percent(value)
}

pub trait AutoDimension {
    fn auto() -> Self;
}

impl AutoDimension for Dimension {
    #[inline(always)]
    fn auto() -> Self {
        Dimension::Auto
    }
}

impl AutoDimension for DefiniteDimensionAuto {
    #[inline(always)]
    fn auto() -> Self {
        DefiniteDimensionAuto::Auto
    }
}

pub fn auto<T: AutoDimension>() -> T {
    T::auto()
}
