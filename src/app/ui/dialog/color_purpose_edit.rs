use eframe::egui::{Context, Key, Modifiers, TextEdit};

use crate::{
    app::{App, request},
    utils::ColorTableKind,
};

/// 用于添加颜色用途标签或编辑已有颜色用途标签的对话框
pub struct ColorPurposeEditDialog {
    id: usize,
    add_mode: bool,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: String,
    first_show: bool,
}

impl ColorPurposeEditDialog {
    /// 用于添加颜色用途标签的对话框
    #[inline]
    pub fn add(kind: ColorTableKind, index: usize, purpose_index: usize) -> Self {
        Self {
            id: 0,
            add_mode: true,
            kind,
            index,
            purpose_index,
            purpose: String::new(),
            first_show: true,
        }
    }

    /// 用于修改已有颜色用途标签的对话框
    #[inline]
    pub fn modify(
        kind: ColorTableKind,
        index: usize,
        purpose_index: usize,
        purpose: String,
    ) -> Self {
        Self {
            id: 0,
            add_mode: false,
            kind,
            index,
            purpose_index,
            purpose,
            first_show: true,
        }
    }
}

impl super::Dialog for ColorPurposeEditDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut eframe::egui::Ui) -> super::DialogResponse {
        let mut response = super::DialogResponse::default();

        super::show_title(
            ui,
            if self.add_mode {
                "添加颜色用途标签"
            } else {
                "修改颜色用途标签"
            },
        );
        let text_edit =
            ui.add(TextEdit::singleline(&mut self.purpose).desired_width(super::MIN_WIDTH));
        if self.first_show {
            text_edit.request_focus();
            self.first_show = false;
        }

        if ui.input_mut(|i| i.consume_key_exact(Modifiers::NONE, Key::Enter)) {
            response = make_confirm_closure(
                self.add_mode,
                self.kind,
                self.index,
                self.purpose_index,
                self.purpose.clone(),
            )
            .into();
        }
        if let Some(confirmed) = super::show_confirm_cancel_buttons(ui) {
            let add_mode = self.add_mode;
            let kind = self.kind;
            let index = self.index;
            let purpose_index = self.purpose_index;
            let purpose = self.purpose.clone();
            if confirmed {
                response =
                    make_confirm_closure(add_mode, kind, index, purpose_index, purpose).into();
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
    purpose_index: usize,
    purpose: String,
) -> super::BoxedDialogClosure {
    Box::new(
        move |dialog: Box<dyn super::Dialog>,
              app: &mut App,
              ctx: &Context,
              _frame: &mut eframe::Frame| {
            let result = if add_mode {
                request::add_color_purpose(app, ctx, kind, index, purpose_index, purpose)
            } else {
                request::modify_color_purpose(app, ctx, kind, index, purpose_index, purpose)
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
