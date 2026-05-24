use eframe::egui::{Context, Frame, Key, Modifiers, Ui, Vec2, vec2};

use crate::{
    app::{App, request, ui::ColorPicker},
    utils::{ColorTableKind, OklchColor},
};

/// 用于添加颜色配置项或修改已有颜色配置项的对话框
pub struct ColorEntryEditDialog {
    id: usize,
    add_mode: bool,
    kind: ColorTableKind,
    index: usize,
    color: OklchColor,
    picker: ColorPicker,
    ref_fg: OklchColor,
    ref_bg: OklchColor,
}

impl ColorEntryEditDialog {
    /// 用于添加颜色配置项的对话框
    #[inline]
    pub fn add(
        kind: ColorTableKind,
        index: usize,
        color: OklchColor,
        ref_fg: OklchColor,
        ref_bg: OklchColor,
    ) -> Self {
        Self {
            id: 0,
            add_mode: true,
            kind,
            index,
            color,
            picker: Default::default(),
            ref_fg,
            ref_bg,
        }
    }

    /// 用于修改已有颜色配置项的对话框
    #[inline]
    pub fn modify(
        kind: ColorTableKind,
        index: usize,
        color: OklchColor,
        ref_fg: OklchColor,
        ref_bg: OklchColor,
    ) -> Self {
        Self {
            id: 0,
            add_mode: false,
            kind,
            index,
            color,
            picker: Default::default(),
            ref_fg,
            ref_bg,
        }
    }
}

impl super::Dialog for ColorEntryEditDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut eframe::egui::Ui) -> super::DialogResponse {
        let mut response = super::DialogResponse::default();

        super::show_title(
            ui,
            if self.add_mode {
                "添加颜色配置项"
            } else {
                "修改颜色配置项"
            },
        );
        if let Err(e) = self.picker.show(ui, &mut self.color) {
            response = (move |dialog: Box<dyn super::Dialog>,
                              app: &mut App,
                              _ctx: &Context,
                              _frame: &mut eframe::Frame| {
                app.dialog.push_boxed(dialog);
                app.dialog.push(super::AlertDialog::new(e.to_string()));
            })
            .into();
        }

        match self.kind {
            ColorTableKind::Foreground => self.show_preview_for_fg_or_bg(ui),
            ColorTableKind::Background => self.show_preview_for_fg_or_bg(ui),
            ColorTableKind::Marker => self.show_preview_for_marker(ui),
            ColorTableKind::Other => {}
        }

        if ui.input_mut(|i| i.consume_key_exact(Modifiers::NONE, Key::Enter)) {
            response =
                make_confirm_closure(self.add_mode, self.kind, self.index, self.color).into();
        }
        if let Some(confirmed) = super::show_confirm_cancel_buttons(ui) {
            if confirmed {
                let add_mode = self.add_mode;
                let kind = self.kind;
                let index = self.index;
                let color = self.color;
                response = make_confirm_closure(add_mode, kind, index, color).into();
            } else {
                response = super::DialogResponse::Close;
            }
        }
        response
    }
}

#[inline]
fn make_confirm_closure(
    add_mode: bool,
    kind: ColorTableKind,
    index: usize,
    color: OklchColor,
) -> super::BoxedDialogClosure {
    Box::new(
        move |dialog: Box<dyn super::Dialog>,
              app: &mut App,
              ctx: &Context,
              _frame: &mut eframe::Frame| {
            let result = if add_mode {
                request::add_color(app, ctx, kind, index, color)
            } else {
                request::modify_color(app, ctx, kind, index, color)
            };
            match result {
                Ok(_) => {}
                Err(e) => {
                    app.dialog.push_boxed(dialog);
                    app.dialog.push(super::AlertDialog::new(format!("{e:#}")));
                }
            }
        },
    )
}

impl ColorEntryEditDialog {
    fn show_preview_for_fg_or_bg(&self, ui: &mut Ui) {
        let (fg, bg, contrast) = if matches!(self.kind, ColorTableKind::Foreground) {
            (
                self.color,
                self.ref_bg,
                self.color.apca_contrast(&self.ref_bg),
            )
        } else {
            (
                self.ref_fg,
                self.color,
                self.ref_fg.apca_contrast(&self.color),
            )
        };
        Frame::NONE.fill(bg.into()).inner_margin(8).show(ui, |ui| {
            ui.set_min_width(super::MIN_WIDTH);
            ui.colored_label(fg, self.color.to_oklch_string());
            ui.colored_label(fg, self.color.to_rgb_string());
            ui.colored_label(fg, self.color.to_hex_string());
            ui.colored_label(fg, format!("APCA对比度：{:.2}", contrast));
        });
    }

    fn show_preview_for_marker(&self, ui: &mut Ui) {
        const HEIGHT: f32 = 32.0;
        const GUTTER_SIZE: Vec2 = vec2(4.0, 16.0);
        const STRIPE_SIZE: Vec2 = vec2(16.0, 4.0);

        let (_, rect) = ui.allocate_space(vec2(super::MIN_WIDTH, HEIGHT));
        ui.painter().rect_filled(rect, 0, self.ref_bg);
        {
            let mut rect = rect;
            rect.max.x = rect.min.x + GUTTER_SIZE.x;
            let half_height = GUTTER_SIZE.y / 2.0;
            let center_y = rect.center().y;
            rect.min.y = center_y - half_height;
            rect.max.y = center_y + half_height;
            ui.painter().rect_filled(rect, 0, self.color);
        }
        {
            let mut rect = rect;
            rect.min.x = rect.max.x - STRIPE_SIZE.x;
            let half_height = STRIPE_SIZE.y / 2.0;
            let center_y = rect.center().y;
            rect.min.y = center_y - half_height;
            rect.max.y = center_y + half_height;
            ui.painter().rect_filled(rect, 0, self.color);
        }
    }
}
