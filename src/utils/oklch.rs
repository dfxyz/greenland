use std::str::FromStr;

use color::{Oklch, OpaqueColor, Srgb};

const MAX_CHROMA: f32 = 0.325;
const MAX_CHROMA_SEARCH_ITERATIONS: u32 = 9; // 0.325 / 2^9 = 0.000634765625, 九次搜索能精确到千分位

const L_FACTOR: f32 = 100.0;
const C_FACTOR: f32 = 1000.0;
const H_FACTOR: f32 = 1.0;
const A_FACTOR: f32 = 100.0;

/// OKLCH颜色的定点数表示
#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct OklchColor {
    /// 亮度的定点数表示，数值范围为`[0, 100]`
    l: u8,
    /// 色度的定点数表示，数值范围为`[0, 1000]`
    c: u16,
    /// 色相的定点数表示，数值范围为`[0, 360]`
    h: u16,
    /// 透明度的定点数表示，数值范围为`[0, 100]`
    a: u8,
}

impl Default for OklchColor {
    fn default() -> Self {
        Self::BLACK
    }
}

impl OklchColor {
    pub const MAX_CHROMA: f32 = MAX_CHROMA;

    const ZERO: Self = Self {
        l: 0,
        c: 0,
        h: 0,
        a: 0,
    };

    pub const BLACK: Self = Self {
        l: 0,
        c: 0,
        h: 0,
        a: 100,
    };

    pub const WHITE: Self = Self {
        l: 100,
        c: 0,
        h: 0,
        a: 100,
    };
}

impl OklchColor {
    /// 创建一个不透明的OKLCH颜色
    #[inline]
    pub fn new_opaque(l: f32, c: f32, h: f32) -> Self {
        Self::new_transparent(l, c, h, 1.0)
    }

    /// 创建一个透明的OKLCH颜色
    #[inline]
    pub fn new_transparent(l: f32, c: f32, h: f32, a: f32) -> Self {
        *Self::ZERO
            .clone()
            .set_l_inner(preprocess_l_f32(l))
            .set_c_inner(preprocess_c_f32(c))
            .set_h_inner(preprocess_h_f32(h))
            .set_a_inner(preprocess_a_f32(a))
            .clamp()
    }

    /// 在亮度、色相保持不变的情况下，调整色度，从而将颜色限制在sRGB空间内
    fn clamp(&mut self) -> &mut Self {
        let max_chroma = max_chroma(self.l(), self.h());
        if self.c() > max_chroma {
            self.set_c_inner(max_chroma);
        }
        self
    }
}

impl OklchColor {
    /// 使用定点数修改亮度（仅内部逻辑使用，不检查数值范围或处理色域缩限）
    #[inline]
    fn set_l_inner(&mut self, value: f32) -> &mut Self {
        self.l = (value * L_FACTOR).round() as _;
        self
    }
    /// 使用定点数修改色度（仅内部逻辑使用，不检查数值范围或处理色域缩限）
    #[inline]
    fn set_c_inner(&mut self, value: f32) -> &mut Self {
        self.c = (value * C_FACTOR).round() as _;
        self
    }
    /// 使用定点数修改色相（仅内部逻辑使用，不检查数值范围或处理色域缩限）
    #[inline]
    fn set_h_inner(&mut self, value: f32) -> &mut Self {
        self.h = (value * H_FACTOR).round() as _;
        self
    }
    /// 使用定点数修改透明度（仅内部逻辑使用，不检查数值范围或处理色域缩限）
    #[inline]
    fn set_a_inner(&mut self, value: f32) -> &mut Self {
        self.a = (value * A_FACTOR).round() as _;
        self
    }

    /// 获取亮度的定点数表示（仅用于特殊逻辑）
    pub fn get_l_inner(&self) -> u8 {
        self.l
    }
    /// 获取色度的定点数表示（仅用于特殊逻辑）
    pub fn get_c_inner(&self) -> u16 {
        self.c
    }
    /// 获取色相的定点数表示（仅用于特殊逻辑）
    pub fn get_h_inner(&self) -> u16 {
        self.h
    }
    /// 获取透明度的定点数表示（仅用于特殊逻辑）
    pub fn get_a_inner(&self) -> u8 {
        self.a
    }
}

impl OklchColor {
    /// 使用浮点数修改亮度
    #[inline]
    pub fn set_l(&mut self, value: f32) -> &mut Self {
        self.set_l_inner(preprocess_l_f32(value)).clamp()
    }
    /// 使用浮点数修改色度
    #[inline]
    pub fn set_c(&mut self, value: f32) -> &mut Self {
        self.set_c_inner(preprocess_c_f32(value)).clamp()
    }
    /// 使用浮点数修改色相
    #[inline]
    pub fn set_h(&mut self, value: f32) -> &mut Self {
        self.set_h_inner(preprocess_h_f32(value)).clamp()
    }
    /// 使用浮点数修改透明度
    #[inline]
    pub fn set_a(&mut self, value: f32) -> &mut Self {
        self.set_a_inner(preprocess_a_f32(value))
    }

    /// 同时使用浮点数修改亮度与色度
    #[inline]
    pub fn set_lc(&mut self, l: f32, c: f32) -> &mut Self {
        self.set_l_inner(preprocess_l_f32(l))
            .set_c_inner(preprocess_c_f32(c))
            .clamp()
    }

    /// 获取亮度的浮点数表示
    #[inline]
    pub fn l(&self) -> f32 {
        self.get_l_inner() as f32 / L_FACTOR
    }
    /// 获取色度的浮点数表示
    #[inline]
    pub fn c(&self) -> f32 {
        self.get_c_inner() as f32 / C_FACTOR
    }
    /// 获取色相的浮点数表示
    #[inline]
    pub fn h(&self) -> f32 {
        self.get_h_inner() as f32 / H_FACTOR
    }
    /// 获取透明度的浮点数表示
    #[inline]
    pub fn a(&self) -> f32 {
        self.get_a_inner() as f32 / A_FACTOR
    }
}

impl OklchColor {
    /// 判断颜色是否不透明
    #[inline]
    fn is_opaque(&self) -> bool {
        self.a >= 100
    }

    /// 将颜色转换为OKLCH字符串表示
    #[inline]
    pub fn to_oklch_string(self) -> String {
        if self.is_opaque() {
            format!("oklch({} {} {})", self.l(), self.c(), self.h())
        } else {
            format!("oklch({} {} {} {})", self.l(), self.c(), self.h(), self.a())
        }
    }

    /// 将颜色转换为RGB字符串表示
    #[inline]
    pub fn to_rgb_string(self) -> String {
        let [r, g, b] = oklch_to_srgb_u8(self.l(), self.c(), self.h());
        if self.is_opaque() {
            format!("rgb({r}, {g}, {b})")
        } else {
            format!("rgba({r}, {g}, {b}, {a})", a = self.a())
        }
    }

    /// 将颜色转换为HEX字符串表示（若颜色透明，使用HEX8格式，否则使用HEX6格式）
    #[inline]
    pub fn to_hex_string(self) -> String {
        let [r, g, b] = oklch_to_srgb_u8(self.l(), self.c(), self.h());
        if self.is_opaque() {
            format!("#{r:02X}{g:02X}{b:02X}")
        } else {
            let a = (self.a() * 255.0).round() as u8;
            format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
        }
    }

    /// 将颜色转换为HEX8字符串表示
    #[inline]
    pub fn to_hex8_string(self) -> String {
        let [r, g, b] = oklch_to_srgb_u8(self.l(), self.c(), self.h());
        let a = (self.a() * 255.0).round() as u8;
        format!("#{r:02X}{g:02X}{b:02X}{a:02X}")
    }

    /// 将颜色转换为RGB的u32表示
    #[inline]
    pub fn to_rgb_u32(self) -> u32 {
        let [r, g, b] = oklch_to_srgb_u8(self.l(), self.c(), self.h());
        (r as u32) << 16 | (g as u32) << 8 | (b as u32)
    }
    /// 将颜色转换为RGBA的u32表示
    #[inline]
    pub fn to_rgba_u32(self) -> u32 {
        let [r, g, b] = oklch_to_srgb_u8(self.l(), self.c(), self.h());
        let a = (self.a() * 255.0).round() as u8;
        (r as u32) << 24 | (g as u32) << 16 | (b as u32) << 8 | (a as u32)
    }

    /// 将颜色转换为BGR的u32表示
    #[inline]
    pub fn to_bgr_u32(self) -> u32 {
        let [r, g, b] = oklch_to_srgb_u8(self.l(), self.c(), self.h());
        (b as u32) << 16 | (g as u32) << 8 | (r as u32)
    }

    /// 将当前颜色作为文本颜色，传入的`bg`为背景颜色，计算APCA对比度
    pub fn apca_contrast(&self, bg: &Self) -> f32 {
        const MAIN_TRC: f64 = 2.4;
        const S_RCO: f64 = 0.2126729;
        const S_GCO: f64 = 0.7151522;
        const S_BCO: f64 = 0.0721750;
        const NORM_BG: f64 = 0.56;
        const NORM_TXT: f64 = 0.57;
        const REV_TXT: f64 = 0.62;
        const REV_BG: f64 = 0.65;
        const BLK_THRS: f64 = 0.022;
        const BLK_CLMP: f64 = 1.414;
        const SCALE_BOW: f64 = 1.14;
        const SCALE_WOB: f64 = 1.14;
        const LO_BOW_OFFSET: f64 = 0.027;
        const LO_WOB_OFFSET: f64 = 0.027;
        const DELTA_Y_MIN: f64 = 0.0005;
        const LO_CLIP: f64 = 0.1;

        let linearize = |v: f32| -> f64 {
            let v = v.clamp(0.0, 1.0) as f64;
            if v <= 0.04045 {
                v / 12.92
            } else {
                ((v + 0.055) / 1.055).powf(MAIN_TRC)
            }
        };

        let to_y = |rgb: &[f32; 3]| -> f64 {
            S_RCO * linearize(rgb[0]) + S_GCO * linearize(rgb[1]) + S_BCO * linearize(rgb[2])
        };

        let soft_clamp = |y: f64| -> f64 {
            if y >= BLK_THRS {
                y
            } else {
                y + (BLK_THRS - y).powf(BLK_CLMP)
            }
        };

        let [r_fg, g_fg, b_fg]: [f32; 3] = (*self).into();
        let [r_bg, g_bg, b_bg]: [f32; 3] = (*bg).into();

        let y_txt = soft_clamp(to_y(&[r_fg, g_fg, b_fg]));
        let y_bg = soft_clamp(to_y(&[r_bg, g_bg, b_bg]));

        if (y_bg - y_txt).abs() < DELTA_Y_MIN {
            return 0.0;
        }

        let lc = if y_bg > y_txt {
            let sapc = (y_bg.powf(NORM_BG) - y_txt.powf(NORM_TXT)) * SCALE_BOW;
            if sapc < LO_CLIP {
                0.0
            } else {
                sapc - LO_BOW_OFFSET
            }
        } else {
            let sapc = (y_bg.powf(REV_BG) - y_txt.powf(REV_TXT)) * SCALE_WOB;
            if sapc > -LO_CLIP {
                0.0
            } else {
                sapc + LO_WOB_OFFSET
            }
        };

        (lc * 100.0) as f32
    }
}

impl FromStr for OklchColor {
    type Err = ();

    /// 将字符串转换为OKLCH颜色
    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let dynamic = color::parse_color(s.trim()).map_err(|_| ())?;
        let [l, c, h, a] = dynamic.to_alpha_color::<Oklch>().components;
        Ok(Self::new_transparent(l, c, h, a))
    }
}

impl From<OklchColor> for [u8; 3] {
    #[inline]
    fn from(value: OklchColor) -> Self {
        oklch_to_srgb_u8(value.l(), value.c(), value.h())
    }
}
impl From<OklchColor> for [u8; 4] {
    fn from(value: OklchColor) -> Self {
        let [r, g, b] = oklch_to_srgb_u8(value.l(), value.c(), value.h());
        [r, g, b, (value.a() * 255.0).round() as u8]
    }
}
impl From<OklchColor> for [f32; 3] {
    #[inline]
    fn from(value: OklchColor) -> Self {
        oklch_to_srgb_f32(value.l(), value.c(), value.h())
    }
}
impl From<OklchColor> for [f32; 4] {
    #[inline]
    fn from(value: OklchColor) -> Self {
        let [r, g, b] = oklch_to_srgb_f32(value.l(), value.c(), value.h());
        [r, g, b, value.a()]
    }
}
impl From<OklchColor> for eframe::egui::Color32 {
    #[inline]
    fn from(value: OklchColor) -> Self {
        let [r, g, b, a]: [u8; 4] = value.into();
        eframe::egui::Color32::from_rgba_unmultiplied(r, g, b, a)
    }
}

impl From<[u8; 3]> for OklchColor {
    #[inline]
    fn from(value: [u8; 3]) -> Self {
        let [l, c, h] = srgb_u8_to_oklch(value[0], value[1], value[2]);
        Self::new_opaque(l, c, h)
    }
}
impl From<[u8; 4]> for OklchColor {
    #[inline]
    fn from(value: [u8; 4]) -> Self {
        let [l, c, h] = srgb_u8_to_oklch(value[0], value[1], value[2]);
        Self::new_transparent(l, c, h, value[3] as f32 / 255.0)
    }
}
impl From<[f32; 3]> for OklchColor {
    #[inline]
    fn from(value: [f32; 3]) -> Self {
        let [l, c, h] = srgb_f32_to_oklch(value[0], value[1], value[2]);
        Self::new_opaque(l, c, h)
    }
}
impl From<[f32; 4]> for OklchColor {
    #[inline]
    fn from(value: [f32; 4]) -> Self {
        let [l, c, h] = srgb_f32_to_oklch(value[0], value[1], value[2]);
        Self::new_transparent(l, c, h, value[3])
    }
}
impl From<eframe::egui::Color32> for OklchColor {
    #[inline]
    fn from(value: eframe::egui::Color32) -> Self {
        let [r, g, b, a] = value.to_srgba_unmultiplied();
        let [l, c, h] = srgb_u8_to_oklch(r, g, b);
        Self::new_transparent(l, c, h, a as f32 / 255.0)
    }
}

/// 通过二分搜索，计算在特定亮度、色相下，在sRGB空间中可以达到的最大色度的近似值
fn max_chroma(l: f32, h: f32) -> f32 {
    let mut lo = 0.0_f32;
    let mut hi = MAX_CHROMA;
    for _ in 0..MAX_CHROMA_SEARCH_ITERATIONS {
        let mid = (lo + hi) / 2.0;
        if oklch_to_srgb_f32(l, mid, h)
            .iter()
            .all(|x| (0.0..=1.0).contains(x))
        {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    lo
}

/// 预处理浮点数表示的亮度值
#[inline]
fn preprocess_l_f32(l: f32) -> f32 {
    l.clamp(0.0, 1.0)
}

/// 预处理浮点数表示的色度值
#[inline]
fn preprocess_c_f32(c: f32) -> f32 {
    c.clamp(0.0, MAX_CHROMA)
}

/// 预处理浮点数表示的色相值
#[inline]
fn preprocess_h_f32(h: f32) -> f32 {
    h.rem_euclid(360.0)
}

/// 预处理浮点数表示的透明度值
#[inline]
fn preprocess_a_f32(a: f32) -> f32 {
    a.clamp(0.0, 1.0)
}

#[inline]
pub fn oklch_to_srgb_u8(l: f32, c: f32, h: f32) -> [u8; 3] {
    oklch_to_srgb_f32(l, c, h).map(|x| (x * 255.0).round() as u8)
}

#[inline]
pub fn oklch_to_srgb_f32(l: f32, c: f32, h: f32) -> [f32; 3] {
    OpaqueColor::<Oklch>::new([l, c, h])
        .convert::<Srgb>()
        .components
}

#[inline]
pub fn srgb_u8_to_oklch(r: u8, g: u8, b: u8) -> [f32; 3] {
    srgb_f32_to_oklch(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0)
}

#[inline]
pub fn srgb_f32_to_oklch(r: f32, g: f32, b: f32) -> [f32; 3] {
    OpaqueColor::<Srgb>::new([r, g, b])
        .convert::<Oklch>()
        .components
}
