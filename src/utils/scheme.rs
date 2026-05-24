use std::{collections::HashMap, ops::Range, path::Path};

use anyhow::Context;
use serde::{Deserialize, Serialize};

use crate::utils::OklchColor;

/// 用于序列化、反序列化配色方案数据的结构体
#[derive(Default, Serialize, Deserialize)]
struct RawColorScheme {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    fg: Vec<RawColorEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    bg: Vec<RawColorEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    marker: Vec<RawColorEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    other: Vec<RawColorEntry>,
}
#[derive(Serialize, Deserialize)]
struct RawColorEntry {
    purposes: Vec<String>,
    oklch: String,
}

/// 配色方案数据
#[derive(Default)]
pub struct ColorScheme {
    /// 前景色颜色表
    fg: ColorTable,
    /// 背景色颜色表
    bg: ColorTable,
    /// 标记颜色表（Gutter或Stripe）
    marker: ColorTable,
    /// 其他颜色表
    other: ColorTable,
}
/// 颜色表类型
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ColorTableKind {
    /// 前景色颜色表
    #[default]
    Foreground,
    /// 背景色颜色表
    Background,
    /// 标记颜色表（Gutter或Stripe）
    Marker,
    /// 其他颜色表
    Other,
}
#[derive(Default)]
pub struct ColorTable {
    /// 颜色配置项列表
    entries: Vec<ColorEntry>,
    /// 索引表：OklchColor -> 颜色配置项列表下标
    oklch_indexes: HashMap<OklchColor, usize>,
    /// 索引表：RGB的u32整数 -> 颜色配置项列表下标
    rgba_u32_indexes: HashMap<u32, usize>,
    /// 索引表：用途 -> 颜色配置项列表下标
    purpose_indexes: HashMap<String, usize>,
}
pub struct ColorEntry {
    /// 颜色的值
    pub color: OklchColor,
    /// 颜色的用途列表
    pub purposes: Vec<String>,
    /// 颜色的OKLCH字符串表示
    pub oklch: String,
    /// 颜色的RGB字符串表示
    pub rgb: String,
    /// 颜色的HEX字符串表示
    pub hex: String,
    /// 颜色的APCA对比度
    pub apca_contrast: Option<f32>,
}

impl ColorScheme {
    /// 从TOML格式的字符串中反序列化配色方案数据
    #[inline]
    pub fn from_toml(s: &str) -> anyhow::Result<Self> {
        let raw: RawColorScheme = toml::from_str(s).context("反序列化错误")?;
        ColorScheme::try_from(raw)
    }

    /// 将配色方案数据序列化为TOML格式的字符串
    #[inline]
    fn to_toml(&self) -> anyhow::Result<String> {
        let raw = RawColorScheme::from(self);
        toml::to_string(&raw).context("序列化错误")
    }

    /// 从指定路径加载配色方案数据
    #[inline]
    pub fn load_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path).context("无法读取文件内容")?;
        Self::from_toml(&content)
    }

    /// 将配色方案数据保存到指定路径
    #[inline]
    pub fn save_to<P: AsRef<Path>>(&self, path: P) -> anyhow::Result<()> {
        let content = self.to_toml()?;
        std::fs::write(path, content).context("无法写入文件内容")
    }
}

impl TryFrom<RawColorScheme> for ColorScheme {
    type Error = anyhow::Error;

    fn try_from(raw: RawColorScheme) -> Result<Self, Self::Error> {
        Ok(Self {
            fg: ColorTable::try_from(raw.fg)?,
            bg: ColorTable::try_from(raw.bg)?,
            marker: ColorTable::try_from(raw.marker)?,
            other: ColorTable::try_from(raw.other)?,
        })
    }
}
impl TryFrom<Vec<RawColorEntry>> for ColorTable {
    type Error = anyhow::Error;

    fn try_from(raw: Vec<RawColorEntry>) -> Result<Self, Self::Error> {
        let mut this = Self::default();
        for entry in raw {
            let entry = ColorEntry::try_from(entry)?;
            let index = this.entries.len();
            if this.oklch_indexes.insert(entry.color, index).is_some() {
                return Err(anyhow::anyhow!("颜色'{}'重复", entry.oklch));
            }
            let rgba_u32 = entry.color.to_rgba_u32();
            if this.rgba_u32_indexes.insert(rgba_u32, index).is_some() {
                return Err(anyhow::anyhow!(
                    "颜色'{}'对应的RGBA的u32整数'{}'重复",
                    entry.oklch,
                    rgba_u32
                ));
            }
            for purpose in &entry.purposes {
                if this
                    .purpose_indexes
                    .insert(purpose.to_owned(), index)
                    .is_some()
                {
                    return Err(anyhow::anyhow!(
                        "颜色'{}'的用途'{}'重复",
                        entry.oklch,
                        purpose
                    ));
                }
            }
            this.entries.push(entry);
        }
        Ok(this)
    }
}
impl TryFrom<RawColorEntry> for ColorEntry {
    type Error = anyhow::Error;

    #[inline]
    fn try_from(raw: RawColorEntry) -> Result<Self, Self::Error> {
        let color: OklchColor = raw
            .oklch
            .parse()
            .map_err(|_| anyhow::anyhow!("解析配置项的OKLCH字符串'{}'失败", raw.oklch))?;
        Ok(ColorEntry {
            color,
            purposes: raw.purposes,
            oklch: color.to_oklch_string(),
            rgb: color.to_rgb_string(),
            hex: color.to_hex_string(),
            apca_contrast: None,
        })
    }
}

impl From<&ColorScheme> for RawColorScheme {
    #[inline]
    fn from(value: &ColorScheme) -> Self {
        RawColorScheme {
            fg: value.fg.entries.iter().map(|entry| entry.into()).collect(),
            bg: value.bg.entries.iter().map(|entry| entry.into()).collect(),
            marker: value
                .marker
                .entries
                .iter()
                .map(|entry| entry.into())
                .collect(),
            other: value
                .other
                .entries
                .iter()
                .map(|entry| entry.into())
                .collect(),
        }
    }
}
impl From<&ColorEntry> for RawColorEntry {
    #[inline]
    fn from(value: &ColorEntry) -> Self {
        RawColorEntry {
            purposes: value.purposes.clone(),
            oklch: value.oklch.clone(),
        }
    }
}

impl ColorScheme {
    /// 获取指定类型的颜色表引用
    #[inline]
    pub fn get_table(&self, kind: ColorTableKind) -> &ColorTable {
        match kind {
            ColorTableKind::Foreground => &self.fg,
            ColorTableKind::Background => &self.bg,
            ColorTableKind::Marker => &self.marker,
            ColorTableKind::Other => &self.other,
        }
    }
    /// 获取指定类型的颜色表可变引用
    #[inline]
    pub fn get_table_mut(&mut self, kind: ColorTableKind) -> &mut ColorTable {
        match kind {
            ColorTableKind::Foreground => &mut self.fg,
            ColorTableKind::Background => &mut self.bg,
            ColorTableKind::Marker => &mut self.marker,
            ColorTableKind::Other => &mut self.other,
        }
    }

    /// 根据带有颜色表前缀的颜色用途字符串获取对应的颜色配置项
    #[inline]
    pub fn get_by_purpose(&self, purpose: &str) -> Option<&ColorEntry> {
        if let Some(purpose) = purpose.strip_prefix(ColorTableKind::Foreground.prefix()) {
            self.fg.get_by_purpose(purpose)
        } else if let Some(purpose) = purpose.strip_prefix(ColorTableKind::Background.prefix()) {
            self.bg.get_by_purpose(purpose)
        } else if let Some(purpose) = purpose.strip_prefix(ColorTableKind::Marker.prefix()) {
            self.marker.get_by_purpose(purpose)
        } else if let Some(purpose) = purpose.strip_prefix(ColorTableKind::Other.prefix()) {
            self.other.get_by_purpose(purpose)
        } else {
            None
        }
    }

    /// 根据RGBA的u32整数获取对应的颜色表前缀与颜色配置项
    #[inline]
    pub fn get_by_rgba_u32(&self, rgba_u32: u32) -> Vec<(&'static str, &ColorEntry)> {
        let mut result = Vec::new();
        if let Some(entry) = self.fg.get_by_rgba_u32(rgba_u32) {
            result.push((ColorTableKind::Foreground.prefix(), entry));
        }
        if let Some(entry) = self.bg.get_by_rgba_u32(rgba_u32) {
            result.push((ColorTableKind::Background.prefix(), entry));
        }
        if let Some(entry) = self.marker.get_by_rgba_u32(rgba_u32) {
            result.push((ColorTableKind::Marker.prefix(), entry));
        }
        if let Some(entry) = self.other.get_by_rgba_u32(rgba_u32) {
            result.push((ColorTableKind::Other.prefix(), entry));
        }
        result
    }
}

impl ColorTableKind {
    #[inline]
    pub fn desc(&self) -> &str {
        match self {
            ColorTableKind::Foreground => "前景色配置表",
            ColorTableKind::Background => "背景色配置表",
            ColorTableKind::Marker => "标记颜色配置表",
            ColorTableKind::Other => "其他颜色配置表",
        }
    }

    /// 获取指定类型的颜色表对应的完整颜色用途字符串的前缀
    #[inline]
    pub fn prefix(&self) -> &str {
        match self {
            ColorTableKind::Foreground => "fg.",
            ColorTableKind::Background => "bg.",
            ColorTableKind::Marker => "marker.",
            ColorTableKind::Other => "other.",
        }
    }
}

impl ColorTable {
    /// 判断当前颜色表是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// 获取当前颜色表中配置项的数量
    #[inline]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// 获取当前颜色表中所有配置项的迭代器
    #[inline]
    pub fn iter(&self) -> impl Iterator<Item = &ColorEntry> {
        self.entries.iter()
    }

    /// 判断当前颜色表中是否存在指定的颜色用途
    #[inline]
    pub fn has_purpose(&self, purpose: &str) -> bool {
        self.purpose_indexes.contains_key(purpose)
    }

    /// 根据颜色用途字符串获取对应的颜色配置项
    #[inline]
    pub fn get_by_purpose(&self, purpose: &str) -> Option<&ColorEntry> {
        self.purpose_indexes
            .get(purpose)
            .map(|index| &self.entries[*index])
    }

    /// 根据RGBA的u32整数获取对应的颜色配置项
    #[inline]
    pub fn get_by_rgba_u32(&self, rgba_u32: u32) -> Option<&ColorEntry> {
        self.rgba_u32_indexes
            .get(&rgba_u32)
            .map(|index| &self.entries[*index])
    }
}

impl ColorTable {
    /// 更新当前颜色表中所有配置项的APCA对比度
    /// - `other`：用于对比的颜色
    /// - `other_as_bg`：是否将`other`作为背景颜色进行对比
    /// - `force`：是否强制更新所有配置项的APCA对比度，若为`false`，则仅更新未计算过APCA对比度的配置项
    #[inline]
    pub fn update_apca_contrast(&mut self, other: OklchColor, other_as_bg: bool, force: bool) {
        for entry in &mut self.entries {
            if !force && entry.apca_contrast.is_some() {
                continue;
            }
            entry.apca_contrast = if other_as_bg {
                Some(entry.color.apca_contrast(&other))
            } else {
                Some(other.apca_contrast(&entry.color))
            }
        }
    }

    /// 重建指定下标范围内的颜色配置项的索引（仅用于插入、移动或删除颜色配置项等下标变化但颜色数值保持不变的场景）
    #[inline]
    fn rebuild_indexes(&mut self, range: Range<usize>) {
        for i in range {
            let entry = &self.entries[i];

            self.oklch_indexes.insert(entry.color, i);

            let rgba_u32 = entry.color.to_rgba_u32();
            self.rgba_u32_indexes.insert(rgba_u32, i);

            for purpose in &entry.purposes {
                self.purpose_indexes.insert(purpose.clone(), i);
            }
        }
    }

    /// 添加一个新的颜色配置项
    pub fn add(
        &mut self,
        index: usize,
        color: OklchColor,
        purposes: Vec<String>,
    ) -> anyhow::Result<()> {
        if index > self.entries.len() {
            anyhow::bail!("颜色配置项的下标超出有效范围");
        }
        let oklch = color.to_oklch_string();
        if self.oklch_indexes.contains_key(&color) {
            anyhow::bail!("已定义相同数值的颜色配置项");
        }
        let rgba_u32 = color.to_rgba_u32();
        if self.rgba_u32_indexes.contains_key(&rgba_u32) {
            anyhow::bail!("已存在RGBA数值相同的颜色配置项");
        }
        for purpose in &purposes {
            if self.purpose_indexes.contains_key(purpose) {
                anyhow::bail!("颜色用途'{}'已被占用", purpose);
            }
        }

        self.entries.insert(
            index,
            ColorEntry {
                color,
                purposes,
                oklch,
                rgb: color.to_rgb_string(),
                hex: color.to_hex_string(),
                apca_contrast: None,
            },
        );
        self.rebuild_indexes(index..index + 1);

        Ok(())
    }

    /// 修改某个颜色配置项的颜色数值；若未发生错误，颜色数值发生变化时，返回旧的颜色数值；否则返回`None`
    pub fn modify_color(
        &mut self,
        index: usize,
        color: OklchColor,
    ) -> anyhow::Result<Option<OklchColor>> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("颜色配置项的下标超出有效范围"))?;
        let old_color = entry.color;
        if old_color == color {
            return Ok(None);
        }
        let oklch = color.to_oklch_string();
        if let Some(i) = self.oklch_indexes.get(&color) {
            if *i == index {
                return Ok(None);
            }
            anyhow::bail!("已定义相同数值的颜色配置项");
        }
        let rgba_u32 = color.to_rgba_u32();
        if let Some(i) = self.rgba_u32_indexes.get(&rgba_u32) {
            if *i == index {
                return Ok(None);
            }
            anyhow::bail!("已存在RGBA数值相同的颜色配置项");
        }

        entry.color = color;
        entry.oklch = oklch;
        entry.rgb = color.to_rgb_string();
        entry.hex = color.to_hex_string();
        entry.apca_contrast = None;
        self.oklch_indexes.remove(&old_color);
        self.rgba_u32_indexes.remove(&old_color.to_rgba_u32());
        self.oklch_indexes.insert(color, index);
        self.rgba_u32_indexes.insert(rgba_u32, index);

        Ok(Some(old_color))
    }

    /// 调整某个颜色配置项的排序：将下标为`from`的配置项移动到下标`to`之前
    ///
    /// 以`[A, B, C, D, E]`为例：
    /// - `from=1, to=2`，实际无变化
    /// - `from=1, to=3` 会得到 `[A, C, D, B, E]`
    /// - `from=2, to=1` 会得到 `[A, C, B, D, E]`
    /// - `from=2, to=2`，实际无变化
    pub fn move_color(&mut self, from: usize, to: usize) -> anyhow::Result<bool> {
        if from >= self.entries.len() || to > self.entries.len() {
            anyhow::bail!("颜色配置项的下标超出有效范围");
        }
        if from == to || from + 1 == to {
            return Ok(false);
        }

        let entry = self.entries.remove(from);
        let insert_at = if from < to { to - 1 } else { to }; // from小于to时，先删后插需要调整插入位置
        self.entries.insert(insert_at, entry);
        let range = if from < to {
            from..(to + 1).min(self.entries.len())
        } else {
            to..from + 1
        };
        self.rebuild_indexes(range);

        Ok(true)
    }

    /// 删除某个颜色配置项
    pub fn remove(&mut self, index: usize) -> anyhow::Result<ColorEntry> {
        if index >= self.entries.len() {
            anyhow::bail!("颜色配置项的下标超出有效范围");
        }

        let entry = self.entries.remove(index);
        self.oklch_indexes.remove(&entry.color);
        self.rgba_u32_indexes.remove(&entry.color.to_rgba_u32());
        for purpose in &entry.purposes {
            self.purpose_indexes.remove(purpose);
        }
        self.rebuild_indexes(index..self.entries.len());

        Ok(entry)
    }

    /// 为某个颜色配置项添加用途字符串
    pub fn add_purpose(
        &mut self,
        index: usize,
        purpose_index: usize,
        purpose: String,
    ) -> anyhow::Result<()> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("颜色配置项的下标超出有效范围"))?;
        if purpose_index > entry.purposes.len() {
            anyhow::bail!("颜色用途的下标超出有效范围");
        }
        if self.purpose_indexes.contains_key(&purpose) {
            anyhow::bail!("颜色用途字符串已被其他颜色配置项使用");
        }

        entry.purposes.insert(purpose_index, purpose.clone());
        self.purpose_indexes.insert(purpose, index);

        Ok(())
    }

    /// 修改某个颜色配置项的颜色用途字符串；成功时，若颜色用途字符串真正发生变化，返回修改前的旧字符串，否则返回None
    pub fn modify_purpose(
        &mut self,
        index: usize,
        purpose_index: usize,
        purpose: String,
    ) -> anyhow::Result<Option<String>> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("颜色配置项的下标超出有效范围"))?;
        let old_purpose = entry
            .purposes
            .get(purpose_index)
            .ok_or_else(|| anyhow::anyhow!("颜色用途的下标超出有效范围"))?;
        if *old_purpose == purpose {
            return Ok(None);
        }
        if let Some(i) = self.purpose_indexes.get(&purpose) {
            if *i == index {
                return Ok(None);
            }
            anyhow::bail!("颜色用途字符串已被其他颜色配置项使用");
        }

        self.purpose_indexes.remove(old_purpose);
        self.purpose_indexes.insert(purpose.clone(), index);
        let old_purpose = std::mem::replace(&mut entry.purposes[purpose_index], purpose);

        Ok(Some(old_purpose))
    }

    /// 调整某个颜色配置项的颜色用途字符串的排序：将下标为`from`的用途字符串移动到下标`to`之前
    pub fn move_purpose(&mut self, index: usize, from: usize, to: usize) -> anyhow::Result<bool> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("颜色配置项的下标超出有效范围"))?;
        if from >= entry.purposes.len() || to > entry.purposes.len() {
            anyhow::bail!("颜色用途的下标超出有效范围");
        }
        if from == to || from + 1 == to {
            return Ok(false);
        }

        let purpose = entry.purposes.remove(from);
        let insert_at = if from < to { to - 1 } else { to }; // from小于to时，先删后插需要调整插入位置
        entry.purposes.insert(insert_at, purpose);

        Ok(true)
    }

    /// 删除某个颜色配置项的用途；成功时返回被删除的颜色用途字符串
    pub fn remove_purpose(&mut self, index: usize, purpose_index: usize) -> anyhow::Result<String> {
        let entry = self
            .entries
            .get_mut(index)
            .ok_or_else(|| anyhow::anyhow!("颜色配置项的下标超出有效范围"))?;
        if purpose_index >= entry.purposes.len() {
            anyhow::bail!("颜色用途的下标超出有效范围");
        }

        let purpose = entry.purposes.remove(purpose_index);
        self.purpose_indexes.remove(&purpose);

        Ok(purpose)
    }
}
