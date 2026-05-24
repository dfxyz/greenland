use eframe::egui::{Context, Ui, ViewportCommand};

use crate::app::App;

mod color_picker;
mod dialog;
mod main_area;
mod menu;
mod toast;

pub use color_picker::*;
pub use dialog::*;
pub use toast::*;

impl eframe::App for App {
    fn ui(&mut self, ui: &mut Ui, frame: &mut eframe::Frame) {
        menu::show(self, ui, frame);
        main_area::show(self, ui, frame);
        dialog::show(self, ui, frame);
        self.toast.show(ui);
        intercept_quit_request(self, ui.ctx());
    }
}

/// 拦截处理退出请求
fn intercept_quit_request(app: &mut App, ctx: &Context) {
    if app.quit_confirmed {
        ctx.send_viewport_cmd(ViewportCommand::Close);
        return;
    }
    if ctx.input(|i| i.viewport().close_requested()) && app.scheme_dirty {
        ctx.send_viewport_cmd(ViewportCommand::CancelClose);
        app.dialog.push(ConfirmDialog::new(
            "配色方案的变更尚未保存，确定要退出吗？",
            |app, _ctx, _frame| {
                app.quit_confirmed = true;
            },
        ));
    }
}
