use std::borrow::Cow;

use anyhow::Context as _;
use eframe::egui::{Align, Context, Frame, Label, Layout, ScrollArea, Ui};
use rfd::FileDialog;

use crate::app::App;

pub struct ExportDialog {
    id: usize,
    content: Cow<'static, str>,
}

impl ExportDialog {
    pub fn new<S>(content: S) -> Self
    where
        S: Into<Cow<'static, str>>,
    {
        Self {
            id: 0,
            content: content.into(),
        }
    }
}

impl super::Dialog for ExportDialog {
    super::define_id_methods!(id);

    fn show(&mut self, ui: &mut Ui) -> super::DialogResponse {
        const MAX_HEIGHT: f32 = 500.0;

        let mut response = super::DialogResponse::default();

        super::show_title(ui, "导出配色方案");

        Frame::NONE
            .inner_margin(8)
            .fill(ui.visuals().extreme_bg_color)
            .show(ui, |ui| {
                ui.set_max_height(MAX_HEIGHT);
                ScrollArea::both().show(ui, |ui| {
                    ui.set_min_width(ui.available_width());
                    ui.add(Label::new(self.content.as_ref()).selectable(true));
                });
            });

        ui.horizontal(|ui| {
            if ui.button("复制").clicked() {
                ui.copy_text(self.content.to_string());
            }
            if ui.button("导出..").clicked() {
                let content = self.content.to_string();
                response = (move |dialog: Box<dyn super::Dialog>,
                                  app: &mut App,
                                  _ctx: &Context,
                                  frame: &mut eframe::Frame| {
                    app.dialog.push_boxed(dialog);
                    if let Some(path) = FileDialog::new()
                        .set_title("选择导出路径")
                        .set_parent(frame)
                        .save_file()
                    {
                        if let Err(e) = std::fs::write(path, content).context("导出配色方案失败")
                        {
                            app.dialog.push(super::AlertDialog::new(format!("{e:#}")));
                        } else {
                            app.dialog
                                .push(super::AlertDialog::new("已成功导出配色方案"));
                        }
                    }
                })
                .into();
            }

            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui.button("关闭").clicked() {
                    response = super::DialogResponse::Close;
                }
            })
        });

        response
    }
}
