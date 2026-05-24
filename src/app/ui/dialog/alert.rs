use std::borrow::Cow;

use eframe::egui::Ui;

/// 简单的提示对话框，只有标题、提示文本与一个关闭按钮
pub struct AlertDialog {
    id: usize,
    message: Cow<'static, str>,
}

impl AlertDialog {
    pub fn new<S>(message: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            id: 0,
            message: message.into(),
        }
    }
}

impl super::Dialog for AlertDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut Ui) -> super::DialogResponse {
        let mut response = super::DialogResponse::default();

        super::show_title(ui, "提示");
        ui.label(self.message.as_ref());
        if super::show_close_button(ui) {
            response = super::DialogResponse::Close;
        }

        response
    }
}
