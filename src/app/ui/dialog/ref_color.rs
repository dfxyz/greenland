use eframe::egui::{Align, Context, Frame, Key, Layout, Modifiers, Ui};

use crate::{
    app::{App, ui::ColorPicker},
    utils::OklchColor,
};

/// 修改参考前景色/背景色的对话框
pub struct ModifyRefColorDialog {
    id: usize,
    current_fg: OklchColor,
    default_fg: OklchColor,
    editing_fg: OklchColor,
    fg_picker: ColorPicker,
    default_bg: OklchColor,
    current_bg: OklchColor,
    editing_bg: OklchColor,
    bg_picker: ColorPicker,
}

impl ModifyRefColorDialog {
    pub fn new(
        current_fg: OklchColor,
        default_fg: OklchColor,
        current_bg: OklchColor,
        default_bg: OklchColor,
    ) -> Self {
        Self {
            id: 0,
            current_fg,
            default_fg,
            editing_fg: current_fg,
            fg_picker: ColorPicker::default(),
            current_bg,
            default_bg,
            editing_bg: current_bg,
            bg_picker: ColorPicker::default(),
        }
    }
}

impl super::Dialog for ModifyRefColorDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut Ui) -> super::DialogResponse {
        let mut response = super::DialogResponse::default();

        super::show_title(ui, "修改参考前景色/背景色");

        ui.columns(2, |ui| {
            if let Some(r) = show_picker_column(
                &mut ui[0],
                "参考前景色",
                &mut self.editing_fg,
                &self.default_fg,
                &mut self.fg_picker,
            ) {
                response = r;
            }

            if let Some(r) = show_picker_column(
                &mut ui[1],
                "参考背景色",
                &mut self.editing_bg,
                &self.default_bg,
                &mut self.bg_picker,
            ) {
                response = r;
            }
        });

        ui.add_space(12.0);

        show_color_preview(ui, "修改前", self.current_fg, self.current_bg);
        show_color_preview(ui, "修改后", self.editing_fg, self.editing_bg);

        if ui.input_mut(|i| i.consume_key_exact(Modifiers::NONE, Key::Enter)) {
            response = make_confirm_closure(self.editing_fg, self.editing_bg).into();
        }
        if let Some(confirmed) = super::show_confirm_cancel_buttons(ui) {
            if confirmed {
                let fg = self.editing_fg;
                let bg = self.editing_bg;
                response = make_confirm_closure(fg, bg).into();
            } else {
                response = super::DialogResponse::Close;
            }
        }
        response
    }
}

#[inline]
fn make_confirm_closure(fg: OklchColor, bg: OklchColor) -> super::BoxedDialogClosure {
    Box::new(
        move |_dialog: Box<dyn super::Dialog>,
              app: &mut App,
              _ctx: &Context,
              _frame: &mut eframe::Frame| {
            app.set_ref_fg_color(fg);
            app.set_ref_bg_color(bg);
        },
    )
}

fn show_picker_column(
    ui: &mut Ui,
    title: &str,
    editing: &mut OklchColor,
    default: &OklchColor,
    picker: &mut ColorPicker,
) -> Option<super::DialogResponse> {
    ui.horizontal(|ui| {
        ui.label(title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let restore_button = ui.small_button("󰦛");
            if restore_button.clicked() {
                *editing = *default;
            }
            restore_button.on_hover_text("重置为默认值");
        });
    });
    if let Err(e) = picker.show(ui, editing) {
        return Some(
            (move |dialog: Box<dyn super::Dialog>,
                   app: &mut App,
                   _ctx: &Context,
                   _frame: &mut eframe::Frame| {
                app.dialog.push_boxed(dialog);
                app.dialog.push(super::AlertDialog::new(format!("{e:#}")));
            })
            .into(),
        );
    }

    None
}

fn show_color_preview(ui: &mut Ui, title: &str, fg: OklchColor, bg: OklchColor) {
    ui.label(title);
    let contrast = fg.apca_contrast(&bg);
    Frame::NONE.fill(bg.into()).inner_margin(8).show(ui, |ui| {
        ui.set_min_width(ui.available_width());
        ui.colored_label(
            fg,
            format!(
                "前景色：{} | {} | {}",
                fg.to_oklch_string(),
                fg.to_rgb_string(),
                fg.to_hex_string()
            ),
        );
        ui.colored_label(
            fg,
            format!(
                "背景色：{} | {} | {}",
                bg.to_oklch_string(),
                bg.to_rgb_string(),
                bg.to_hex_string()
            ),
        );
        ui.colored_label(fg, format!("APCA对比度：{contrast:.2}"));
    });
}
