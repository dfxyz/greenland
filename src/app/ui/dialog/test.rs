use eframe::egui::{Context, Ui};

use crate::{
    app::{App, ui::ColorPicker},
    utils::OklchColor,
};

/// 堆叠测试用的对话框
#[derive(Default)]
pub struct StackTestDialog {
    id: usize,
    level: usize,
}

impl super::Dialog for StackTestDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut Ui) -> super::DialogResponse {
        let mut response = super::DialogResponse::default();

        super::show_title(ui, format!("堆叠测试对话框 - Level {}", self.level));
        let next_level = self.level + 1;
        if ui.button("弹出下一层对话框").clicked() {
            response = (move |dialog: Box<dyn super::Dialog>,
                              app: &mut App,
                              _ctx: &Context,
                              _frame: &mut eframe::Frame| {
                app.dialog.push_boxed(dialog);
                app.dialog.push(Self {
                    level: next_level,
                    ..Default::default()
                });
            })
            .into();
        }
        if ui.button("替换成下一层对话框").clicked() {
            response = (move |_dialog: Box<dyn super::Dialog>,
                              app: &mut App,
                              _ctx: &Context,
                              _frame: &mut eframe::Frame| {
                app.dialog.push(Self {
                    level: next_level,
                    ..Default::default()
                });
            })
            .into();
        }
        if super::show_close_button(ui) {
            response = super::DialogResponse::Close;
        }

        response
    }
}

/// 颜色选择器测试用的对话框
#[derive(Default)]
pub struct ColorPickerTestDialog {
    id: usize,
    picker: ColorPicker,
    color: OklchColor,
}

impl super::Dialog for ColorPickerTestDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut Ui) -> super::DialogResponse {
        let mut response = super::DialogResponse::default();

        super::show_title(ui, "颜色选择器测试对话框");
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
        if super::show_close_button(ui) {
            response = super::DialogResponse::Close;
        }

        response
    }
}
