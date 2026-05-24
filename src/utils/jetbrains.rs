use std::{collections::BTreeMap, path::Path};

use anyhow::Context as _;

/// JetBrains IDE 配色方案
#[derive(Default)]
pub struct ColorScheme {
    /// 简单的颜色选项
    ///
    /// ```xml
    /// <option name="COLOR_NAME" value="123456" /> <!-- HEX6 -->
    /// <option name="COLOR_NAME" value="F0" /> <!-- Shorten HEX8 -->
    /// ```
    pub colors: BTreeMap<String, Option<u32>>,

    /// 带有多个字段的颜色选项
    pub attributes: BTreeMap<String, Attribute>,
}
#[derive(Clone, PartialEq, Eq)]
pub enum Attribute {
    /// 继承指定名称的属性
    ///
    /// ```xml
    /// <option name="ATTR_NAME" baseAttribute="BASE_ATTR_NAME" />
    /// ```
    Inherit(String),

    /// 带有值的Attribute
    ///
    /// ```xml
    /// <option name="ATTR_NAME">
    ///   <value>
    ///     <option name="FOREGROUND" value="0" />
    ///     <option name="BACKGROUND" value="123" />
    ///     <option name="ERROR_STRIPE_COLOR" value="456789" />
    ///     <option name="EFFECT_COLOR" value="123456" />
    ///     <option name="EFFECT_TYPE" value="1" />
    ///     <option name="FONT_TYPE" value="2" />
    ///   </value>
    /// </option>
    /// ```
    ///
    /// 其中`FOREGROUND`、`BACKGROUND`、`ERROR_STRIPE_COLOR`、`EFFECT_COLOR`是缩短版本的HEX字符串
    Value(AttributeWithValue),
}
#[derive(Default, Clone, PartialEq, Eq)]
pub struct AttributeWithValue {
    pub fg: Option<u32>,
    pub bg: Option<u32>,
    pub stripe: Option<u32>,
    pub ec: Option<u32>,
    pub et: Option<i8>,
    pub ft: Option<i8>,
}

impl ColorScheme {
    pub fn load<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let file = std::fs::read_to_string(path).context("打开文件时发生错误")?;
        Self::from_str(&file)
    }

    pub fn from_str(content: &str) -> anyhow::Result<Self> {
        let mut reader = quick_xml::Reader::from_str(content);
        reader.config_mut().trim_text(true);

        let mut this = Self::default();

        let mut state = ParseState::Root;
        loop {
            let event = reader.read_event().context("解析XML时发生错误")?;
            match event {
                quick_xml::events::Event::Start(bytes_start) => {
                    parse_xml_start_event(&mut state, bytes_start).context("解析XML时发生错误")?;
                }
                quick_xml::events::Event::End(bytes_end) => {
                    parse_xml_end_event(&mut state, bytes_end).context("解析XML时发生错误")?;
                }
                quick_xml::events::Event::Empty(bytes_start) => {
                    parse_xml_empty_event(&mut state, &mut this, bytes_start)
                        .context("解析XML时发生错误")?;
                }
                quick_xml::events::Event::Eof => break,
                _ => {}
            }
        }

        for (_, attr) in &mut this.attributes {
            if let Attribute::Value(attr) = attr {
                if attr.ec.is_none() {
                    attr.et = None;
                }
            }
        }

        Ok(this)
    }
}

impl AttributeWithValue {
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.fg.is_none()
            && self.bg.is_none()
            && self.stripe.is_none()
            && self.ec.is_none()
            && self.et.is_none()
            && self.ft.is_none()
    }
}

enum ParseState {
    Root,
    InScheme,
    InIrrelevant(usize),
    InColors,
    InAttributes,
    InAttributeOption(String),
    InAttributeOptionValue(String),
}

fn parse_xml_start_event(
    state: &mut ParseState,
    bytes_start: quick_xml::events::BytesStart,
) -> anyhow::Result<()> {
    let tag_name = bytes_start.name();
    let tag_name = String::from_utf8_lossy(tag_name.as_ref());
    match state {
        ParseState::Root => {
            if tag_name != "scheme" {
                anyhow::bail!("初始的XML解析事件不是'<scheme>'，而是'<{tag_name}>'");
            }
            *state = ParseState::InScheme;
        }
        ParseState::InScheme => {
            if tag_name == "colors" {
                *state = ParseState::InColors;
            } else if tag_name == "attributes" {
                *state = ParseState::InAttributes;
            } else {
                *state = ParseState::InIrrelevant(0);
            }
        }
        ParseState::InIrrelevant(level) => {
            *state = ParseState::InIrrelevant(*level + 1);
        }
        ParseState::InColors => {
            anyhow::bail!("处理'colors'的子节点时碰到了'<{tag_name}>'，而非'<option .. />");
        }
        ParseState::InAttributes => {
            if tag_name != "option" {
                anyhow::bail!("处理'attributes'的子节点时碰到了'<{tag_name}>'，而非'<option>'");
            }
            let name = get_attr(&bytes_start, "name")
                .context("解析'attributes::option'的元素属性时发生错误")?
                .context("某个'attributes::option'节点缺少'name'属性")?;
            if get_attr(&bytes_start, "baseAttributes")
                .context("解析'attributes::option'的元素属性时发生错误")?
                .is_some()
            {
                anyhow::bail!(
                    "'attributes::option<{name}>'可能是带有值的变体，但含有'baseAttributes'属性"
                );
            }
            *state = ParseState::InAttributeOption(name);
        }
        ParseState::InAttributeOption(name) => {
            if tag_name != "value" {
                anyhow::bail!(
                    "处理'attributes::option'的子节点时碰到了'<{tag_name}>'，而非'<value>'"
                );
            }
            *state = ParseState::InAttributeOptionValue(std::mem::take(name));
        }
        ParseState::InAttributeOptionValue(_) => {
            anyhow::bail!(
                "处理'attributes::option::value'的子节点时碰到了'<{tag_name}>'，而非'<option .. />"
            );
        }
    }
    Ok(())
}

fn parse_xml_end_event(
    state: &mut ParseState,
    bytes_end: quick_xml::events::BytesEnd,
) -> anyhow::Result<()> {
    let tag_name = bytes_end.name();
    let tag_name = String::from_utf8_lossy(tag_name.as_ref());
    match state {
        ParseState::Root => anyhow::bail!("初始的XML解析事件不是'<scheme>'，而是'</{tag_name}>'"),
        ParseState::InScheme => {
            // DONE
        }
        ParseState::InIrrelevant(level) => {
            if *level == 0 {
                *state = ParseState::InScheme;
            } else {
                *state = ParseState::InIrrelevant(*level - 1);
            }
        }
        ParseState::InColors => {
            *state = ParseState::InScheme;
        }
        ParseState::InAttributes => {
            *state = ParseState::InScheme;
        }
        ParseState::InAttributeOption(_) => {
            *state = ParseState::InAttributes;
        }
        ParseState::InAttributeOptionValue(name) => {
            *state = ParseState::InAttributeOption(std::mem::take(name));
        }
    }
    Ok(())
}

fn parse_xml_empty_event(
    state: &mut ParseState,
    scheme: &mut ColorScheme,
    bytes_start: quick_xml::events::BytesStart,
) -> anyhow::Result<()> {
    let tag_name = bytes_start.name();
    let tag_name = String::from_utf8_lossy(tag_name.as_ref());
    match state {
        ParseState::Root => {
            anyhow::bail!("初始的XML解析事件不是'<scheme>'，而是'<{tag_name} .. />'");
        }
        ParseState::InScheme => {}
        ParseState::InIrrelevant(_) => {}
        ParseState::InColors => {
            if tag_name != "option" {
                anyhow::bail!(
                    "处理'colors'的子节点时碰到了'<{tag_name} .. />'，而非'<option .. />'"
                );
            }
            let name = get_attr(&bytes_start, "name")
                .context("解析'colors::option'的元素属性时发生错误")?
                .context("某个'colors::option'节点缺少'name'属性")?;
            let color = get_attr(&bytes_start, "value")
                .context("解析'colors::option'的元素属性时发生错误")?
                .context("某个'colors::option'节点缺少'value'属性")?;
            let value = if color.is_empty() {
                None
            } else {
                let color = shorten_hex_to_u32(&color).with_context(|| {
                    format!("'colors::option<{name}>'的'value'属性值不是有效的HEX字符串")
                })?;
                Some(color)
            };
            scheme.colors.insert(name, value);
        }
        ParseState::InAttributes => {
            if tag_name != "option" {
                anyhow::bail!(
                    "处理'attributes'的子节点时碰到了'<{tag_name} .. />'，而非'<option .. />'"
                );
            }
            let name = get_attr(&bytes_start, "name")
                .context("解析'attributes::option'的元素属性时发生错误")?
                .context("某个'attributes::option'节点缺少'name'属性")?;
            let base = get_attr(&bytes_start, "baseAttributes")
                .context("解析'attributes::option'的元素属性时发生错误")?
                .context("某个'attributes::option'节点为'<option .. />'的形式，但缺少'baseAttributes'属性")?;
            scheme.attributes.insert(name, Attribute::Inherit(base));
        }
        ParseState::InAttributeOption(name) => {
            if tag_name != "value" {
                anyhow::bail!(
                    "处理'attributes::option<{name}>'的子节点时碰到了'<{tag_name} .. />'，而非'<value />'"
                );
            }
            scheme
                .attributes
                .insert(name.clone(), Attribute::Value(Default::default()));
        }
        ParseState::InAttributeOptionValue(name) => {
            if tag_name != "option" {
                anyhow::bail!(
                    "处理'attributes::option<{name}>::value'的子节点时碰到了'<{tag_name} .. />'，而非'<option .. />'"
                );
            }
            let attr_name = get_attr(&bytes_start, "name")
                .with_context(|| {
                    format!("解析'attributes::option<{name}>::value::option'的元素属性时发生错误")
                })?
                .with_context(|| {
                    format!("某个'attributes::option<{name}>::value::option'节点缺少'name'属性")
                })?;
            if attr_name == "FOREGROUND" {
                parse_attribute_u32_value(scheme, name, bytes_start, |attr, value| {
                    attr.fg = Some(value)
                })?;
            } else if attr_name == "BACKGROUND" {
                parse_attribute_u32_value(scheme, name, bytes_start, |attr, value| {
                    attr.bg = Some(value)
                })?;
            } else if attr_name == "ERROR_STRIPE_COLOR" {
                parse_attribute_u32_value(scheme, name, bytes_start, |attr, value| {
                    attr.stripe = Some(value)
                })?;
            } else if attr_name == "EFFECT_COLOR" {
                parse_attribute_u32_value(scheme, name, bytes_start, |attr, value| {
                    attr.ec = Some(value)
                })?;
            } else if attr_name == "EFFECT_TYPE" {
                parse_attribute_i8_value(scheme, name, bytes_start, |attr, value| {
                    attr.et = Some(value)
                })?;
            } else if attr_name == "FONT_TYPE" {
                parse_attribute_i8_value(scheme, name, bytes_start, |attr, value| {
                    attr.ft = Some(value)
                })?;
            } else {
                anyhow::bail!(
                    "解析到了预期之外的节点'attributes::option<{name}>::value::option<{attr_name}>'"
                );
            }
        }
    }
    Ok(())
}

fn parse_attribute_u32_value(
    scheme: &mut ColorScheme,
    name: &str,
    bytes_start: quick_xml::events::BytesStart,
    setter: impl FnOnce(&mut AttributeWithValue, u32),
) -> anyhow::Result<()> {
    let value = get_attr(&bytes_start, "value")
        .with_context(|| {
            format!("解析'attributes::option<{name}>::value::option'的元素属性时发生错误")
        })?
        .with_context(|| {
            format!("某个'attributes::option<{name}>::value::option'节点缺少'value'属性")
        })?;
    let value = shorten_hex_to_u32(&value).with_context(|| {
        format!(
            " 'attributes::option<{name}>::value::option'节点的'value'属性值不是有效的HEX字符串"
        )
    })?;
    match scheme
        .attributes
        .entry(name.to_string())
        .or_insert(Attribute::Value(Default::default()))
    {
        Attribute::Inherit(_) => {
            anyhow::bail!("'attributes::option<{name}>'的当前值是继承自其他属性的变体")
        }
        Attribute::Value(attr) => {
            setter(attr, value);
        }
    }
    Ok(())
}

fn parse_attribute_i8_value(
    scheme: &mut ColorScheme,
    name: &str,
    bytes_start: quick_xml::events::BytesStart,
    setter: impl FnOnce(&mut AttributeWithValue, i8),
) -> anyhow::Result<()> {
    use std::str::FromStr;
    let value = get_attr(&bytes_start, "value")
        .with_context(|| {
            format!("解析'attributes::option<{name}>::value::option'的元素属性时发生错误")
        })?
        .with_context(|| {
            format!("某个'attributes::option<{name}>::value::option'节点缺少'value'属性")
        })?;
    let value = i8::from_str(&value).with_context(|| {
        format!(" 'attributes::option<{name}>::value::option'节点的'value'属性值不是有效的i8整数")
    })?;
    match scheme
        .attributes
        .entry(name.to_string())
        .or_insert(Attribute::Value(Default::default()))
    {
        Attribute::Inherit(_) => {
            anyhow::bail!("'attributes::option<{name}>'的当前值是继承自其他属性的变体")
        }
        Attribute::Value(attr) => {
            setter(attr, value);
        }
    }
    Ok(())
}

#[inline]
fn shorten_hex_to_u32(raw: &str) -> anyhow::Result<u32> {
    let value =
        u32::from_str_radix(raw, 16).map_err(|_| anyhow::anyhow!("无法解析HEX字符串: {raw}"))?;
    if raw.len() == 8 {
        Ok(value)
    } else {
        Ok((value << 8) | 0xFF)
    }
}

#[inline]
fn get_attr(
    bytes_start: &quick_xml::events::BytesStart,
    key: &str,
) -> anyhow::Result<Option<String>> {
    let result = bytes_start
        .try_get_attribute(key)
        .context("解析XML元素属性时发生错误")?;
    match result {
        Some(attr) => {
            let value = String::from_utf8(attr.value.to_vec())
                .context("XML元素的属性值不是有效的UTF-8字符串")?;
            Ok(Some(value))
        }
        None => Ok(None),
    }
}

#[derive(Default)]
pub struct Difference {
    pub removed_colors: Vec<String>,
    pub removed_attributes: Vec<String>,
    pub modified: ColorScheme,
}

impl ColorScheme {
    pub fn compare(&self, other: &Self) -> Difference {
        let mut result = Difference::default();

        for (name, this) in &self.colors {
            match other.colors.get(name) {
                Some(that) => {
                    if this != that {
                        result.modified.colors.insert(name.clone(), *that);
                    }
                }
                None => {
                    result.removed_colors.push(name.clone());
                }
            }
        }
        for (name, value) in &other.colors {
            if !self.colors.contains_key(name) {
                result.modified.colors.insert(name.clone(), *value);
            }
        }

        for (name, this) in &self.attributes {
            match other.attributes.get(name) {
                Some(that) => {
                    if this != that {
                        result
                            .modified
                            .attributes
                            .insert(name.clone(), that.clone());
                    }
                }
                None => {
                    result.removed_attributes.push(name.clone());
                }
            }
        }
        for (name, value) in &other.attributes {
            if !self.attributes.contains_key(name) {
                result
                    .modified
                    .attributes
                    .insert(name.clone(), value.clone());
            }
        }

        result
    }
}
