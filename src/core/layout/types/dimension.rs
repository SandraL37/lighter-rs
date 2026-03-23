pub type Dimension = taffy::Dimension;
pub type DefiniteDimension = taffy::LengthPercentage;
pub type DefiniteDimensionAuto = taffy::LengthPercentageAuto;

use crate::core::reactive::signal::MaybeSignal;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LengthUnit {
    Pixels,
    Percent,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Length {
    value: f32,
    unit: LengthUnit,
}

trait FromLengthValue {
    fn from_length_value(value: f32, unit: LengthUnit) -> Self;
}

impl FromLengthValue for Dimension {
    #[inline(always)]
    fn from_length_value(value: f32, unit: LengthUnit) -> Self {
        match unit {
            LengthUnit::Pixels => Dimension::length(value),
            LengthUnit::Percent => Dimension::percent(value),
        }
    }
}

impl FromLengthValue for DefiniteDimension {
    #[inline(always)]
    fn from_length_value(value: f32, unit: LengthUnit) -> Self {
        match unit {
            LengthUnit::Pixels => DefiniteDimension::length(value),
            LengthUnit::Percent => DefiniteDimension::percent(value),
        }
    }
}

impl FromLengthValue for DefiniteDimensionAuto {
    #[inline(always)]
    fn from_length_value(value: f32, unit: LengthUnit) -> Self {
        match unit {
            LengthUnit::Pixels => DefiniteDimensionAuto::length(value),
            LengthUnit::Percent => DefiniteDimensionAuto::percent(value),
        }
    }
}

impl From<Length> for Dimension {
    fn from(value: Length) -> Self {
        Self::from_length_value(value.value, value.unit)
    }
}

impl From<Length> for DefiniteDimension {
    fn from(value: Length) -> Self {
        Self::from_length_value(value.value, value.unit)
    }
}

impl From<Length> for DefiniteDimensionAuto {
    fn from(value: Length) -> Self {
        Self::from_length_value(value.value, value.unit)
    }
}

impl<T> From<Length> for MaybeSignal<T>
where
    T: FromLengthValue + 'static,
{
    fn from(value: Length) -> Self {
        MaybeSignal::Static(T::from_length_value(value.value, value.unit))
    }
}

#[must_use]
pub fn px(value: f32) -> Length {
    Length {
        value,
        unit: LengthUnit::Pixels,
    }
}

#[must_use]
pub fn percent(value: f32) -> Length {
    Length {
        value,
        unit: LengthUnit::Percent,
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AutoValue;

trait FromAutoValue {
    fn from_auto_value() -> Self;
}

impl FromAutoValue for Dimension {
    #[inline(always)]
    fn from_auto_value() -> Self {
        Dimension::auto()
    }
}

impl FromAutoValue for DefiniteDimensionAuto {
    #[inline(always)]
    fn from_auto_value() -> Self {
        DefiniteDimensionAuto::auto()
    }
}

impl From<AutoValue> for Dimension {
    fn from(_: AutoValue) -> Self {
        Self::from_auto_value()
    }
}

impl From<AutoValue> for DefiniteDimensionAuto {
    fn from(_: AutoValue) -> Self {
        Self::from_auto_value()
    }
}

impl<T> From<AutoValue> for MaybeSignal<T>
where
    T: FromAutoValue + 'static,
{
    fn from(_: AutoValue) -> Self {
        MaybeSignal::Static(T::from_auto_value())
    }
}

#[must_use]
pub fn auto() -> AutoValue {
    AutoValue
}
