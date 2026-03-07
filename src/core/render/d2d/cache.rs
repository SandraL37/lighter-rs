use crate::{core::render::d2d::*, elements::text::FontWeight};

// TODO: everything is hashmap, needs to be fixed
// TODO: invalidate when DeviceContext is invalid
pub struct D2DCache {
    d2d_device_context: D2DDeviceContext,
    dwrite_factory: DWriteFactory,
    text_format: HashMap<TextFormatKey, DWriteTextFormat>,
    solid_color_brush: HashMap<SolidColorBrushKey, D2DSolidColorBrush>,
}

impl D2DCache {
    pub fn new(dwrite_factory: DWriteFactory, d2d_device_context: D2DDeviceContext) -> Self {
        Self {
            dwrite_factory,
            d2d_device_context,
            text_format: HashMap::new(),
            solid_color_brush: HashMap::new(),
        }
    }

    pub fn get_text_format(&mut self, props: &TextProps) -> Result<DWriteTextFormat> {
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

impl From<&TextProps> for TextFormatKey {
    fn from(props: &TextProps) -> Self {
        Self {
            font_family: Arc::clone(&props.font_family),
            font_size_bits: props.font_size.to_bits(),
            font_weight: props.font_weight,
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
