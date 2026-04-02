use std::{collections::HashMap, sync::Arc};

use windows::core::HSTRING;

use crate::{core::render::d2d::*, elements::text::style::FontWeight};

// TODO: everything is hashmap, needs to be fixed
// TODO: invalidate when DeviceContext is invalid
pub struct D2DCache {
    d2d_device_context: D2DDeviceContext,
    dwrite_factory: DWriteFactory,
    text_format: HashMap<TextFormatKey, DWriteTextFormat>,
    text_layout: HashMap<TextLayoutKey, DWriteTextLayout>,
    solid_color_brush: HashMap<SolidColorBrushKey, D2DSolidColorBrush>,
}

impl D2DCache {
    pub fn new(dwrite_factory: DWriteFactory, d2d_device_context: D2DDeviceContext) -> Self {
        Self {
            dwrite_factory,
            d2d_device_context,
            text_format: HashMap::new(),
            text_layout: HashMap::new(),
            solid_color_brush: HashMap::new(),
        }
    }

    pub fn get_text_format(&mut self, props: &TextStyle) -> Result<DWriteTextFormat> {
        let key = TextFormatKey::from(props);

        if let Some(cached) = self.text_format.get(&key) {
            return Ok(cached.clone());
        }

        let fmt = unsafe {
            self.dwrite_factory.CreateTextFormat(
                &HSTRING::from(props.font_family.to_string()),
                None,
                &[DWRITE_FONT_AXIS_VALUE {
                    axisTag: DWRITE_FONT_AXIS_TAG_WEIGHT,
                    value: props.font_weight.0 as f32,
                }],
                props.font_size,
                &HSTRING::from(""),
            )?
        };

        self.text_format.insert(key, fmt.clone());
        Ok(fmt)
    }

    pub fn get_text_layout(
        &mut self,
        props: &TextStyle,
        max_size: Size<f32>,
    ) -> Result<DWriteTextLayout> {
        let fmt_key = TextFormatKey::from(props);
        let key = TextLayoutKey::from(props, max_size, fmt_key);

        if let Some(cached) = self.text_layout.get(&key) {
            return Ok(cached.clone());
        }

        let fmt = self.get_text_format(props)?;

        let utf16: Vec<u16> = props.content.encode_utf16().collect(); // TODO: check if better methods
        let layout: DWriteTextLayout = unsafe {
            self.dwrite_factory
                .CreateTextLayout(&utf16, &fmt, max_size.width, max_size.height)?
                .cast()?
        };

        self.text_layout.insert(key, layout.clone());

        Ok(layout)
    }

    pub fn get_solid_color_brush(&mut self, color: &Color) -> Result<D2DSolidColorBrush> {
        let key = SolidColorBrushKey::from(color);

        if let Some(cached) = self.solid_color_brush.get(&key) {
            return Ok(cached.clone());
        }

        let brush = unsafe {
            self.d2d_device_context
                .CreateSolidColorBrush(&(*color).into(), None)?
        };

        self.solid_color_brush.insert(key, brush.clone());
        Ok(brush)
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct TextFormatKey {
    font_family: Arc<str>,
    font_size_bits: u32,
    font_weight: FontWeight,
}

impl From<&TextStyle> for TextFormatKey {
    fn from(props: &TextStyle) -> Self {
        Self {
            font_family: Arc::clone(&props.font_family),
            font_size_bits: props.font_size.to_bits(),
            font_weight: props.font_weight,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct TextLayoutKey {
    content: Arc<str>,
    max_width_bits: u32,
    max_height_bits: u32,
    format: TextFormatKey,
}

impl TextLayoutKey {
    pub fn from(props: &TextStyle, bounds: Size<f32>, format: TextFormatKey) -> Self {
        Self {
            content: Arc::clone(&props.content),
            max_width_bits: bounds.width.to_bits(),
            max_height_bits: bounds.height.to_bits(),
            format,
        }
    }
}

#[derive(Hash, Eq, PartialEq, Clone)]
struct SolidColorBrushKey {
    r_bits: u32,
    g_bits: u32,
    b_bits: u32,
    a_bits: u32,
}

impl From<&Color> for SolidColorBrushKey {
    fn from(color: &Color) -> Self {
        Self {
            r_bits: color.r.to_bits(),
            g_bits: color.g.to_bits(),
            b_bits: color.b.to_bits(),
            a_bits: color.a.to_bits(),
        }
    }
}
