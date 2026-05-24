use std::borrow::Cow;

use eframe::egui::{Context, Ui};

use crate::app::App;

type BoxedConfirmDialogClosure = Box<dyn FnMut(&mut App, &Context, &mut eframe::Frame)>;

/// 通用的确认对话框
pub struct ConfirmDialog {
    id: usize,
    message: Cow<'static, str>,
    on_confirm: Option<BoxedConfirmDialogClosure>,
}

impl ConfirmDialog {
    pub fn new<S, F>(message: S, on_confirm: F) -> Self
    where
        S: Into<Cow<'static, str>>,
        F: FnMut(&mut App, &Context, &mut eframe::Frame) + 'static,
    {
        Self {
            id: 0,
            message: message.into(),
            on_confirm: Some(Box::new(on_confirm)),
        }
    }
}

impl super::Dialog for ConfirmDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut Ui) -> super::DialogResponse {
        super::show_title(ui, "确认");
        ui.label(self.message.as_ref());
        match super::show_confirm_cancel_buttons(ui) {
            Some(confirmed) => {
                if confirmed {
                    let mut on_confirm = self
                        .on_confirm
                        .take()
                        .expect("同一个ConfirmDialog实例被反复确认");
                    super::DialogResponse::CloseWithClosure(Box::new(
                        move |_dialog: Box<dyn super::Dialog>,
                              app: &mut App,
                              ctx: &Context,
                              frame: &mut eframe::Frame| {
                            on_confirm(app, ctx, frame);
                        },
                    ))
                } else {
                    super::DialogResponse::Close
                }
            }
            None => super::DialogResponse::KeepOpen,
        }
    }
}
