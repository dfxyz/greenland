use eframe::egui::{
    Button, Id, Key, Label, MenuBar, Modifiers, Panel, RichText, Ui, ViewportCommand,
};

use crate::app::{App, request};

use super::ModifyRefColorDialog;

macro_rules! button_clicked {
    ($ui:ident, $text:literal $(, $shortcut:literal)?) => {
        $ui.add(Button::new($text) $(.shortcut_text($shortcut))?).clicked()
    }
}
macro_rules! key_pressed {
    ($ui:ident, $($modifiers:ident)|+, $key:ident) => {
        $ui.input_mut(|i| i.consume_key_exact($(Modifiers::$modifiers)|+, Key::$key))
    };
}

/// 绘制APP顶部菜单栏
pub fn show(app: &mut App, ui: &mut Ui, frame: &mut eframe::Frame) {
    Panel::top(Id::new("Panel::top")).show_inside(ui, |ui| {
        MenuBar::new().ui(ui, |ui| {
            ui.menu_button("文件", |ui| {
                if button_clicked!(ui, "新建", "CTRL+N") {
                    request::new_scheme(app, ui.ctx());
                }
                ui.separator();
                if button_clicked!(ui, "打开..", "CTRL+O") {
                    request::open_scheme(app, ui.ctx(), frame);
                }
                ui.separator();
                if button_clicked!(ui, "保存", "CTRL+S") {
                    request::save_scheme(app, ui.ctx(), frame);
                }
                if button_clicked!(ui, "另存为..", "CTRL+SHIFT+S") {
                    request::save_scheme_to_other_path(app, ui.ctx(), frame);
                }
                ui.separator();
                if button_clicked!(ui, "选择模板文件并导出配色方案..", "CTRL+E") {
                    request::export_scheme(app, frame);
                }
                ui.separator();
                if button_clicked!(ui, "退出", "CTRL+Q") {
                    ui.send_viewport_cmd(ViewportCommand::Close);
                }
            });
            ui.menu_button("编辑", |ui| {
                if ui
                    .add_enabled(
                        app.history.can_undo(),
                        Button::new("撤销").shortcut_text("CTRL+Z"),
                    )
                    .clicked()
                {
                    request::undo(app, ui.ctx());
                }
                if ui
                    .add_enabled(
                        app.history.can_redo(),
                        Button::new("重做").shortcut_text("CTRL+Y"),
                    )
                    .clicked()
                {
                    request::redo(app, ui.ctx());
                }
                ui.separator();
                if button_clicked!(ui, "修改参考前景色/背景色..", "CTRL+R") {
                    show_modify_ref_color_dialog(app);
                }
                ui.menu_button("选择参考前景色", |ui| {
                    let mut color_to_set = None;

                    if ui.button("默认值").clicked() {
                        color_to_set = Some(app.get_default_fg());
                    }
                    ui.separator();
                    let mut empty = true;
                    let recent_colors = app.ref_color.recent_fg_colors();
                    for color in recent_colors {
                        empty = false;
                        if ui
                            .button((RichText::new("󰝤").color(color), color.to_oklch_string()))
                            .clicked()
                        {
                            color_to_set = Some(color);
                        }
                    }
                    if empty {
                        ui.add_enabled(false, Label::new("（暂无）"));
                    }

                    if let Some(color) = color_to_set {
                        app.set_ref_fg_color(color);
                    }
                });
                ui.menu_button("选择参考背景色", |ui| {
                    let mut color_to_set = None;

                    if ui.button("默认值").clicked() {
                        color_to_set = Some(app.get_default_bg());
                    }
                    ui.separator();
                    let mut empty = true;
                    let recent_colors = app.ref_color.recent_bg_colors();
                    for color in recent_colors {
                        empty = false;
                        if ui
                            .button((RichText::new("󰝤").color(color), color.to_oklch_string()))
                            .clicked()
                        {
                            color_to_set = Some(color);
                        }
                    }
                    if empty {
                        ui.add_enabled(false, Label::new("（暂无）"));
                    }

                    if let Some(color) = color_to_set {
                        app.set_ref_bg_color(color);
                    }
                });
            });
            #[cfg(debug_assertions)]
            show_test_menu(app, ui);
        });
    });

    if app.dialog.is_empty() {
        handle_menu_shortcuts(app, ui, frame);
    }
}

#[cfg(debug_assertions)]
fn show_test_menu(app: &mut App, ui: &mut Ui) {
    ui.menu_button("测试", |ui| {
        ui.menu_button("Toast提示", |ui| {
            if ui.button("弹出INFO提示").clicked() {
                app.toast.info("测试Toast提示：INFO");
            }
            if ui.button("弹出WARN提示").clicked() {
                app.toast.warn("测试Toast提示：WARN");
            }
            if ui.button("弹出ERROR提示").clicked() {
                app.toast.error("测试Toast提示：ERROR");
            }
            ui.separator();
            if ui.button("弹出自动消失的INFO提示").clicked() {
                app.toast.info_auto_dismiss("测试Toast提示：自动消失的INFO");
            }
            if ui.button("弹出自动消失的WARN提示").clicked() {
                app.toast.warn_auto_dismiss("测试Toast提示：自动消失的WARN");
            }
            if ui.button("弹出自动消失的ERROR提示").clicked() {
                app.toast
                    .error_auto_dismiss("测试Toast提示：自动消失的ERROR");
            }
            ui.separator();
            if ui.button("弹出较长内容的提示").clicked() {
                let msg = "Do you like what you see? ".repeat(20);
                app.toast.info(msg);
            }
            if ui.button("弹出自动消失的超长内容的提示").clicked() {
                let msg = "Do you like what you see? ".repeat(20);
                app.toast.info_auto_dismiss(msg);
            }
        });
        ui.menu_button("对话框", |ui| {
            use eframe::egui::Context;

            use super::{AlertDialog, ColorPickerTestDialog, ConfirmDialog, StackTestDialog};

            if ui.button("弹出堆叠测试对话框").clicked() {
                app.dialog.push(StackTestDialog::default());
            }
            ui.separator();
            if ui.button("弹出简单提示对话框").clicked() {
                app.dialog.push(AlertDialog::new("一句话提示测试"));
            }
            if ui.button("弹出简单提示对话框（较长内容）").clicked() {
                let msg = "Do you like what you see? ".repeat(20);
                app.dialog.push(AlertDialog::new(msg));
            }
            ui.separator();
            if ui.button("弹出确认对话框").clicked() {
                app.dialog.push(ConfirmDialog::new(
                    "是否要弹出另一个对话框？",
                    move |app: &mut App, _ctx: &Context, _frame: &mut eframe::Frame| {
                        app.dialog.push(AlertDialog::new("已弹出另一个对话框"));
                    },
                ));
            }
            ui.separator();
            if ui.button("弹出颜色选择器测试对话框").clicked() {
                app.dialog.push(ColorPickerTestDialog::default());
            }
        });
        ui.menu_button("配色方案状态", |ui| {
            if ui.button("强行设置为脏状态").clicked() {
                app.mark_scheme_dirty(ui.ctx());
            }
            if ui.button("强行设置为干净状态").clicked() {
                app.mark_scheme_clean(ui.ctx());
            }
        });
    });
}

fn handle_menu_shortcuts(app: &mut App, ui: &mut Ui, frame: &mut eframe::Frame) {
    if key_pressed!(ui, CTRL, N) {
        request::new_scheme(app, ui.ctx());
    }
    if key_pressed!(ui, CTRL, O) {
        request::open_scheme(app, ui.ctx(), frame);
    }
    if key_pressed!(ui, CTRL, S) {
        request::save_scheme(app, ui.ctx(), frame);
    }
    if key_pressed!(ui, CTRL | SHIFT, S) {
        request::save_scheme_to_other_path(app, ui.ctx(), frame);
    }
    if key_pressed!(ui, CTRL, E) {
        request::export_scheme(app, frame);
    }
    if key_pressed!(ui, CTRL, Q) {
        ui.send_viewport_cmd(ViewportCommand::Close);
    }
    if key_pressed!(ui, CTRL, Z) {
        request::undo(app, ui.ctx());
    }
    if key_pressed!(ui, CTRL, Y) {
        request::redo(app, ui.ctx());
    }
    if key_pressed!(ui, CTRL, R) {
        show_modify_ref_color_dialog(app);
    }
}

#[inline]
fn show_modify_ref_color_dialog(app: &mut App) {
    app.dialog.push(ModifyRefColorDialog::new(
        app.ref_color.fg(),
        app.get_default_fg(),
        app.ref_color.bg(),
        app.get_default_bg(),
    ));
}
