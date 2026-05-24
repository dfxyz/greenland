use std::{str::FromStr, sync::OnceLock};

use anyhow::Context as _;
use arboard::Clipboard;
use eframe::egui::{
    Color32, ColorImage, Context, DragValue, Mesh, Rect, Sense, Shape, Stroke, StrokeKind,
    TextEdit, TextureHandle, TextureOptions, Ui, Vec2, pos2, vec2,
};

use crate::utils::{OklchColor, oklch_to_srgb_f32, oklch_to_srgb_u8};

const HEX_EDITOR_WIDTH: f32 = 90.0;

const L_DECIMALS: usize = 2;
const C_DECIMALS: usize = 3;
const H_DECIMALS: usize = 0;
const A_DECIMALS: usize = 2;

const L_DRAG_SPEED: f32 = 0.01;
const C_DRAG_SPEED: f32 = 0.001;
const H_DRAG_SPEED: f32 = 1.0;
const A_DRAG_SPEED: f32 = 0.01;

const L_DRAG_VALUE_WIDTH: f32 = 68.0;
const C_DRAG_VALUE_WIDTH: f32 = 76.0;
const H_DRAG_VALUE_WIDTH: f32 = 64.0;
const A_DRAG_VALUE_WIDTH: f32 = 68.0;

const WIDGET_COMMON_WIDTH: f32 = 300.0;
const SLIDER_COMMON_HEIGHT: f32 = 20.0;
const SLIDER_COMMON_INDICATOR_HALF_WIDTH: f32 = 5.0;
const SLIDER_COMMON_INDICATOR_HEIGHT: f32 = 10.0;
const SLIDER_COMMON_INDICATOR_STROKE_SIZE: f32 = 1.5;

const LC_PICKER_BG_GRAY: u8 = 224;
const LC_PICKER_PADDING: f32 = 4.0;
const LC_PICKER_HEIGHT: f32 = 200.0;
const LC_PICKER_INDICATOR_OUTER_RADIUS: f32 = 8.0;
const LC_PICKER_INDICATOR_MIDDLE_RADIUS: f32 = 6.5;
const LC_PICKER_INDICATOR_INNER_RADIUS: f32 = 5.0;

const HUE_SLIDER_BG_L: f32 = 0.7502;
const HUE_SLIDER_BG_C: f32 = 0.127552;

const ALPHA_SLIDER_CHECKER_SIZE: f32 = 10.0;
const ALPHA_SLIDER_CHECKER_BG_LIGHT_GRAY: u8 = 180;
const ALPHA_SLIDER_CHECKER_BG_DARK_GRAY: u8 = 120;

/// 基于OKLCH空间的颜色选择器
#[derive(Default)]
pub struct ColorPicker {
    /// 缓存的「亮度-色度」选择控件的背景纹理；第一个元素为色相
    lc_picker_bg_texture: Option<(u16, TextureHandle)>,
    /// 缓存的「透明度」选择控件的背景遮罩纹理；第一个元素为RGB颜色的u32表示
    alpha_slider_bg_mask_texture: Option<(u32, TextureHandle)>,
    /// HEX编辑框对应的字符串缓存
    hex_editor_buf: String,
    /// 当前HEX编辑框是否获得焦点
    hex_editor_focused: bool,
}

impl ColorPicker {
    /// 绘制颜色选择器控件
    pub fn show(&mut self, ui: &mut Ui, color: &mut OklchColor) -> anyhow::Result<()> {
        let mut result = Ok(());
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                self.show_color_square(ui, color);
                self.show_hex_editor(ui, color);
                self.show_copy_button(ui, color);
                result = self.show_paste_button(ui, color);
            });
            Self::show_drag_values(ui, color);
            self.show_lc_picker(ui, color);
            Self::show_hue_slider(ui, color);
            self.show_alpha_slider(ui, color);
        });
        result
    }

    fn show_color_square(&mut self, ui: &mut Ui, color: &OklchColor) {
        let height = ui.spacing().interact_size.y;
        let (rect, rsp) = ui.allocate_exact_size(Vec2::splat(height), Sense::empty());
        let painter = ui.painter();
        painter.rect_filled(rect.expand(-1.0), 0.0, *color);
        if rsp.hovered() {
            painter.rect_stroke(
                rect,
                0.0,
                ui.style().visuals.widgets.hovered.bg_stroke,
                StrokeKind::Inside,
            );
        }
    }

    fn show_hex_editor(&mut self, ui: &mut Ui, color: &mut OklchColor) {
        if !self.hex_editor_focused {
            self.hex_editor_buf = color.to_hex_string();
        }
        let hex_editor = TextEdit::singleline(&mut self.hex_editor_buf)
            .desired_width(HEX_EDITOR_WIDTH)
            .show(ui);
        self.hex_editor_focused = hex_editor.response.has_focus();
        if hex_editor.response.lost_focus() {
            if self.hex_editor_buf != color.to_hex_string()
                && let Ok(new_color) = OklchColor::from_str(&self.hex_editor_buf)
            {
                *color = new_color;
            }
            self.hex_editor_buf = color.to_hex_string();
        }
    }

    fn show_copy_button(&mut self, ui: &mut Ui, color: &OklchColor) {
        ui.menu_button("复制..", |ui| {
            let text = color.to_oklch_string();
            if ui.button(&text).clicked() {
                ui.copy_text(text);
            }
            let text = color.to_rgb_string();
            if ui.button(&text).clicked() {
                ui.copy_text(text);
            }
            let text = color.to_hex_string();
            if ui.button(&text).clicked() {
                ui.copy_text(text);
            }
        });
    }

    fn show_paste_button(&mut self, ui: &mut Ui, color: &mut OklchColor) -> anyhow::Result<()> {
        if ui.button("粘贴").clicked() {
            let mut clipboard = Clipboard::new().context("无法访问剪贴板")?;
            let content = clipboard.get_text().context("无法获取剪贴板中的内容")?;
            let new_color = OklchColor::from_str(&content)
                .map_err(|_| anyhow::anyhow!("无法将剪贴板中的内容解析为颜色表达式"))?;
            *color = new_color;
        }
        Ok(())
    }

    fn show_drag_values(ui: &mut Ui, color: &mut OklchColor) {
        ui.horizontal(|ui| {
            let height = ui.spacing().interact_size.y;
            macro_rules! drag_value {
                ($name:ident, $width:expr, $prefix:literal, $range:expr, $speed:expr, $decimals:expr, $setter:ident) => {
                    {
                        let current = color.$name();
                        let mut $name = current;
                        ui.add_sized(vec2($width, height),
                            DragValue::new(&mut $name)
                                .prefix($prefix)
                                .range($range)
                                .speed($speed)
                                .min_decimals($decimals)
                                .max_decimals($decimals),
                        );
                        if $name != current {
                            color.$setter($name);
                        }
                    }
                };
            }
            drag_value!(l, L_DRAG_VALUE_WIDTH, "L ", 0.0f32..=1.0, L_DRAG_SPEED, L_DECIMALS, set_l);
            drag_value!(c, C_DRAG_VALUE_WIDTH, "C ", 0.0f32..=OklchColor::MAX_CHROMA, C_DRAG_SPEED, C_DECIMALS, set_c );
            drag_value!(h, H_DRAG_VALUE_WIDTH, "H ", 0.0f32..=360.0, H_DRAG_SPEED, H_DECIMALS, set_h);
            drag_value!(a, A_DRAG_VALUE_WIDTH, "A ", 0.0f32..=1.0, A_DRAG_SPEED, A_DECIMALS, set_a);
        });
    }

    fn prepare_lc_picker_bg_texture(&mut self, ctx: &Context, color: &OklchColor) -> TextureHandle {
        let h = color.h();
        let h_inner = color.get_h_inner();
        if let Some((cached_h, texture)) = &self.lc_picker_bg_texture
            && *cached_h == h_inner
        {
            return texture.clone();
        }
        let n = WIDGET_COMMON_WIDTH as usize;
        let ss = n * 2; // 两倍超级采样
        let [bg_r, bg_g, bg_b, _] = Color32::from_gray(LC_PICKER_BG_GRAY).to_srgba_unmultiplied();
        let [bg_r, bg_g, bg_b] = [bg_r as f32, bg_g as f32, bg_b as f32];
        let mut pixels = Vec::with_capacity(n * n);
        for py in 0..n {
            for px in 0..n {
                let mut r_acc = 0.0f32;
                let mut g_acc = 0.0f32;
                let mut b_acc = 0.0f32;
                for sy in 0..2 {
                    for sx in 0..2 {
                        let spx = px * 2 + sx;
                        let spy = py * 2 + sy;
                        let c = (spx as f32 / (ss as f32 - 1.0)) * OklchColor::MAX_CHROMA;
                        let l = 1.0 - (spy as f32 / (ss as f32 - 1.0));
                        let [rf, gf, bf] = oklch_to_srgb_f32(l, c, h);
                        if (0.0..=1.0).contains(&rf)
                            && (0.0..=1.0).contains(&gf)
                            && (0.0..=1.0).contains(&bf)
                        {
                            r_acc += rf * 255.0;
                            g_acc += gf * 255.0;
                            b_acc += bf * 255.0;
                        } else {
                            r_acc += bg_r;
                            g_acc += bg_g;
                            b_acc += bg_b;
                        }
                    }
                }
                pixels.push(Color32::from_rgb(
                    (r_acc / 4.0).round() as u8,
                    (g_acc / 4.0).round() as u8,
                    (b_acc / 4.0).round() as u8,
                ));
            }
        }
        let color_image = ColorImage::new([n, n], pixels);
        let texture = ctx.load_texture(
            "OklchColorPicker::LcPickerBgTexture",
            color_image,
            TextureOptions::LINEAR,
        );
        self.lc_picker_bg_texture = Some((h_inner, texture.clone()));
        texture
    }

    fn show_lc_picker(&mut self, ui: &mut Ui, color: &mut OklchColor) {
        ui.add_space(LC_PICKER_PADDING);

        let (rect, rsp) = ui.allocate_exact_size(
            vec2(WIDGET_COMMON_WIDTH, LC_PICKER_HEIGHT),
            Sense::click_and_drag(),
        );

        // 填充背景
        {
            let texture = self.prepare_lc_picker_bg_texture(ui, color);
            let mut mesh = Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().add(Shape::Mesh(mesh.into()));
        }

        // 处理悬浮事件
        if rsp.hovered() {
            ui.painter().rect_stroke(
                rect,
                0.0,
                ui.style().visuals.widgets.hovered.bg_stroke,
                StrokeKind::Outside,
            );
        }

        // 处理拖拽、点击事件
        if (rsp.dragged() || rsp.clicked())
            && let Some(pos) = rsp.interact_pointer_pos()
        {
            let relative = ((pos - rect.min) / rect.size()).clamp(Vec2::ZERO, Vec2::splat(1.0));
            color.set_lc(1.0 - relative.y, relative.x * OklchColor::MAX_CHROMA);
        }

        // 绘制提示器
        {
            let cx = rect.min.x + (color.c() / OklchColor::MAX_CHROMA) * rect.width();
            let cy = rect.min.y + (1.0 - color.l()) * rect.height();
            let center = pos2(cx, cy);
            let painter = ui.painter();
            painter.circle_filled(center, LC_PICKER_INDICATOR_OUTER_RADIUS, Color32::BLACK);
            painter.circle_filled(center, LC_PICKER_INDICATOR_MIDDLE_RADIUS, Color32::WHITE);
            painter.circle_filled(center, LC_PICKER_INDICATOR_INNER_RADIUS, *color);
        }

        ui.add_space(LC_PICKER_PADDING);
    }

    fn prepare_hue_slider_bg_texture(ctx: &Context) -> TextureHandle {
        static TEXTURE: OnceLock<TextureHandle> = OnceLock::new();
        TEXTURE
            .get_or_init(|| {
                let w = WIDGET_COMMON_WIDTH as usize;
                let mut pixels = Vec::with_capacity(w);
                for px in 0..w {
                    let h = px as f32 / (w as f32 - 1.0) * 360.0;
                    let [r, g, b] = oklch_to_srgb_u8(HUE_SLIDER_BG_L, HUE_SLIDER_BG_C, h);
                    pixels.push(Color32::from_rgb(r, g, b));
                }
                let color_image = ColorImage::new([w, 1], pixels);
                ctx.load_texture(
                    "OklchColorPicker::HueSliderBgTexture",
                    color_image,
                    TextureOptions::LINEAR,
                )
            })
            .clone()
    }

    fn show_hue_slider(ui: &mut Ui, color: &mut OklchColor) {
        let (rect, rsp) = ui.allocate_exact_size(
            vec2(WIDGET_COMMON_WIDTH, SLIDER_COMMON_HEIGHT),
            Sense::click_and_drag(),
        );

        // 填充背景
        {
            let texture = Self::prepare_hue_slider_bg_texture(ui);
            let mut mesh = Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().add(Shape::Mesh(mesh.into()));
        }

        // 处理悬浮事件
        if rsp.hovered() {
            ui.painter().rect_stroke(
                rect,
                0.0,
                ui.style().visuals.widgets.hovered.bg_stroke,
                StrokeKind::Outside,
            );
        }

        // 处理拖拽、点击事件
        if (rsp.dragged() || rsp.clicked())
            && let Some(pos) = rsp.interact_pointer_pos()
        {
            let relative = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            color.set_h(relative * 360.0);
        }

        // 绘制提示器
        {
            let thumb_x = rect.min.x + (color.h() / 360.0) * rect.width();
            let thumb_y = rect.max.y - SLIDER_COMMON_INDICATOR_HEIGHT;
            ui.painter().add(Shape::convex_polygon(
                vec![
                    pos2(thumb_x, thumb_y),
                    pos2(thumb_x - SLIDER_COMMON_INDICATOR_HALF_WIDTH, rect.max.y),
                    pos2(thumb_x + SLIDER_COMMON_INDICATOR_HALF_WIDTH, rect.max.y),
                ],
                OklchColor::new_opaque(HUE_SLIDER_BG_L, HUE_SLIDER_BG_C, color.h()),
                Stroke::new(SLIDER_COMMON_INDICATOR_STROKE_SIZE, Color32::BLACK),
            ));
        }
    }

    fn prepare_alpha_slider_checker_bg_texture(ctx: &Context) -> TextureHandle {
        static TEXTURE: OnceLock<TextureHandle> = OnceLock::new();
        TEXTURE
            .get_or_init(|| {
                let light = Color32::from_gray(ALPHA_SLIDER_CHECKER_BG_LIGHT_GRAY);
                let dark = Color32::from_gray(ALPHA_SLIDER_CHECKER_BG_DARK_GRAY);
                let color_image = ColorImage::new([2, 2], vec![light, dark, dark, light]);
                ctx.load_texture(
                    "OklchColorPicker::AlphaSliderCheckerBgTexture",
                    color_image,
                    TextureOptions::NEAREST_REPEAT,
                )
            })
            .clone()
    }

    fn prepare_alpha_slider_bg_mask_texture(
        &mut self,
        ctx: &Context,
        color: &OklchColor,
    ) -> TextureHandle {
        let rgb_u32 = color.to_rgb_u32();
        if let Some((cached_rgb_u32, texture)) = &self.alpha_slider_bg_mask_texture
            && *cached_rgb_u32 == rgb_u32
        {
            return texture.clone();
        }
        let [r8, g8, b8]: [u8; 3] = (*color).into();
        let w = WIDGET_COMMON_WIDTH as usize;
        let mut pixels = Vec::with_capacity(w);
        for px in 0..w {
            let alpha = px as f32 / (w as f32 - 1.0);
            let a8 = (alpha * 255.0).round() as u8;
            pixels.push(Color32::from_rgba_unmultiplied(r8, g8, b8, a8));
        }
        let color_image = ColorImage::new([w, 1], pixels);
        let texture = ctx.load_texture(
            "OklchColorPicker::AlphaSliderBgMaskTexture",
            color_image,
            TextureOptions::LINEAR,
        );
        self.alpha_slider_bg_mask_texture = Some((rgb_u32, texture.clone()));
        texture
    }

    fn show_alpha_slider(&mut self, ui: &mut Ui, color: &mut OklchColor) {
        let (rect, rsp) = ui.allocate_exact_size(
            vec2(WIDGET_COMMON_WIDTH, SLIDER_COMMON_HEIGHT),
            Sense::click_and_drag(),
        );

        // 填充棋盘图案背景
        {
            let texture = Self::prepare_alpha_slider_checker_bg_texture(ui);
            let mut mesh = Mesh::with_texture(texture.id());
            let columns = rect.width() / ALPHA_SLIDER_CHECKER_SIZE;
            let rows = rect.height() / ALPHA_SLIDER_CHECKER_SIZE;
            let uv = Rect::from_min_max(pos2(0.0, 0.0), pos2(columns, rows));
            mesh.add_rect_with_uv(rect, uv, Color32::WHITE);
            ui.painter().add(Shape::Mesh(mesh.into()));
        }

        // 填充遮罩纹理
        {
            let texture = self.prepare_alpha_slider_bg_mask_texture(ui, color);
            let mut mesh = Mesh::with_texture(texture.id());
            mesh.add_rect_with_uv(
                rect,
                Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
                Color32::WHITE,
            );
            ui.painter().add(Shape::Mesh(mesh.into()));
        }

        // 处理悬浮事件
        if rsp.hovered() {
            ui.painter().rect_stroke(
                rect,
                0.0,
                ui.style().visuals.widgets.hovered.bg_stroke,
                StrokeKind::Outside,
            );
        }

        // 处理拖拽、点击事件
        if (rsp.dragged() || rsp.clicked())
            && let Some(pos) = rsp.interact_pointer_pos()
        {
            let relative = ((pos.x - rect.min.x) / rect.width()).clamp(0.0, 1.0);
            color.set_a(relative);
        }

        // 绘制提示器
        {
            let thumb_x = rect.min.x + (color.a()) * rect.width();
            let thumb_y = rect.max.y - SLIDER_COMMON_INDICATOR_HEIGHT;
            ui.painter().add(Shape::convex_polygon(
                vec![
                    pos2(thumb_x, thumb_y),
                    pos2(thumb_x - SLIDER_COMMON_INDICATOR_HALF_WIDTH, rect.max.y),
                    pos2(thumb_x + SLIDER_COMMON_INDICATOR_HALF_WIDTH, rect.max.y),
                ],
                Color32::WHITE,
                Stroke::new(SLIDER_COMMON_INDICATOR_STROKE_SIZE, Color32::BLACK),
            ));
        }
    }
}
