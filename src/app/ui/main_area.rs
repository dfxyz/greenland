use eframe::{
    egui::{
        Align, Button, CentralPanel, Color32, Context, Direction, DragAndDrop, Frame, Id, Key,
        LayerId, Layout, Modifiers, Order, Response, RichText, ScrollArea, Sense, Stroke,
        StrokeKind, Ui, UiBuilder, Vec2, pos2, vec2,
    },
    emath,
};

use crate::{
    app::{
        App, request,
        ui::{ColorEntryEditDialog, ColorPurposeEditDialog, ConfirmDialog},
    },
    utils::{ColorEntry, ColorTable, ColorTableKind, OklchColor},
};

const MARKER_PREVIEW_AREA_WIDTH: f32 = 16.0;
const MARKER_PREVIEW_AREA_START_SPACE: f32 = 16.0;
const GUTTER_SIZE: Vec2 = vec2(4.0, 16.0);
const STRIPE_SIZE: Vec2 = vec2(16.0, 4.0);
const DRAG_INDICATOR_STROKE_WIDTH: f32 = 2.0;
const DRAG_INDICATOR_COLOR: Color32 = Color32::GREEN;

struct ColorEntryDragPayload {
    kind: ColorTableKind,
    index: usize,
}
struct ColorPurposeDragPayload {
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
}

/// 绘制APP主区域
pub fn show(app: &mut App, ui: &mut Ui, _frame: &mut eframe::Frame) {
    CentralPanel::default().show_inside(ui, |ui| {
        ui.horizontal(|ui| {
            show_top_area(app, ui);
        });
        show_color_entries(app, ui);
    });
}

/// 绘制主页面顶部的颜色表切换按钮、添加颜色配置项按钮
fn show_top_area(app: &mut App, ui: &mut Ui) {
    ui.horizontal(|ui| {
        ui.label("颜色表：");

        macro_rules! toggle_button {
            ($text:literal, $shortcut:literal, $key:ident, $table_kind:path) => {
                if ui
                    .add(
                        Button::selectable(matches!(app.current_table_kind, $table_kind), $text)
                            .shortcut_text($shortcut),
                    )
                    .clicked()
                    || ui.input_mut(|i| i.consume_key_exact(Modifiers::ALT, Key::$key))
                {
                    app.current_table_kind = $table_kind;
                }
            };
        }
        toggle_button!("前景色", "ALT+1", Num1, ColorTableKind::Foreground);
        toggle_button!("背景色", "ALT+2", Num2, ColorTableKind::Background);
        toggle_button!("标记颜色", "ALT+3", Num3, ColorTableKind::Marker);
        toggle_button!("其他颜色", "ALT+4", Num4, ColorTableKind::Other);

        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            if ui
                .button("新增..")
                .on_hover_text("在当前颜色表的末尾新增颜色配置项")
                .clicked()
            {
                let kind = app.current_table_kind;
                app.dialog.push(ColorEntryEditDialog::add(
                    kind,
                    app.scheme.get_table(kind).len(),
                    app.get_last_edited_color(kind),
                    app.ref_color.fg(),
                    app.ref_color.bg(),
                ));
            }
        });
    });
}

/// 绘制颜色配置项列表
fn show_color_entries(app: &mut App, ui: &mut Ui) {
    let kind = app.current_table_kind;
    let table = app.scheme.get_table(kind);
    if table.is_empty() {
        ui.with_layout(
            Layout::centered_and_justified(Direction::LeftToRight),
            |ui| {
                ui.label("（暂无颜色配置项）");
            },
        );
        return;
    }
    match kind {
        ColorTableKind::Foreground => {
            let table = app.scheme.get_table_mut(kind);
            table.update_apca_contrast(app.ref_color.bg(), true, false);
            show_scroll_area_for_color_entries(app, ui);
        }
        ColorTableKind::Background => {
            let table = app.scheme.get_table_mut(kind);
            table.update_apca_contrast(app.ref_color.fg(), false, false);
            show_scroll_area_for_color_entries(app, ui);
        }
        ColorTableKind::Marker => show_color_entries_for_marker(app, ui),
        ColorTableKind::Other => show_scroll_area_for_color_entries(app, ui),
    }
}

type Action = Box<dyn FnOnce(&mut App, &Context) + 'static>;

/// 绘制颜色配置项列表对应的滚动区域
fn show_scroll_area_for_color_entries(app: &mut App, ui: &mut Ui) {
    let kind = app.current_table_kind;
    let table = app.scheme.get_table(kind);
    let ref_fg = if matches!(kind, ColorTableKind::Foreground) {
        None
    } else {
        Some(app.ref_color.fg())
    };
    let ref_bg = if matches!(kind, ColorTableKind::Background) {
        None
    } else {
        Some(app.ref_color.bg())
    };
    let last_edited_color = app.get_last_edited_color(kind);
    let mut action: Option<Action> = None;
    ScrollArea::vertical().show(ui, |ui| {
        for (index, entry) in table.iter().enumerate() {
            if let Some(a) =
                show_color_entry(ui, kind, index, entry, ref_fg, ref_bg, last_edited_color)
            {
                action = Some(a);
            }
        }
    });
    if let Some(action) = action {
        action(app, ui);
    }
}

/// 绘制标记颜色表的配置项列表；与其他颜色表的不同之处在于主区域的两侧会额外绘制Gutter、Stripe的预览控件
fn show_color_entries_for_marker(app: &mut App, ui: &mut Ui) {
    ui.allocate_ui_with_layout(
        ui.available_size_before_wrap(),
        Layout::left_to_right(Align::TOP),
        |ui| {
            show_marker_preview_area(
                ui,
                app.scheme.get_table(ColorTableKind::Marker),
                app.ref_color.bg(),
                GUTTER_SIZE,
            );

            let mut rect = ui.available_rect_before_wrap();
            rect.min.x += ui.spacing().item_spacing.x;
            rect.max.x -= MARKER_PREVIEW_AREA_WIDTH + ui.spacing().item_spacing.x;
            ui.scope_builder(
                UiBuilder::new()
                    .max_rect(rect)
                    .layout(Layout::top_down(Align::LEFT)),
                |ui| {
                    show_scroll_area_for_color_entries(app, ui);
                },
            );

            show_marker_preview_area(
                ui,
                app.scheme.get_table(ColorTableKind::Marker),
                app.ref_color.bg(),
                STRIPE_SIZE,
            );
        },
    );
}

fn show_marker_preview_area(
    ui: &mut Ui,
    table: &ColorTable,
    ref_bg: OklchColor,
    widget_size: Vec2,
) {
    let (id, rect) = ui.allocate_space(vec2(MARKER_PREVIEW_AREA_WIDTH, ui.available_height()));
    ui.painter().rect_filled(rect, 0, ref_bg);

    ui.scope_builder(
        UiBuilder::new()
            .id_salt(id)
            .max_rect(rect)
            .layout(Layout::top_down(Align::LEFT)),
        |ui| {
            ScrollArea::vertical().show(ui, |ui| {
                ui.add_space(MARKER_PREVIEW_AREA_START_SPACE);
                for entry in table.iter() {
                    let (rect, response) = ui.allocate_exact_size(widget_size, Sense::empty());
                    ui.painter().rect_filled(rect, 0, entry.color);
                    let mut hover_text = entry.oklch.to_owned();
                    if !entry.purposes.is_empty() {
                        hover_text.push_str(": ");
                        hover_text.push_str(&entry.purposes.join(", "))
                    }
                    response.on_hover_text(hover_text);
                }
            });
        },
    );
}

/// 绘制单个颜色配置项
fn show_color_entry(
    ui: &mut Ui,
    kind: ColorTableKind,
    index: usize,
    entry: &ColorEntry,
    ref_fg: Option<OklchColor>,
    ref_bg: Option<OklchColor>,
    last_edited_color: OklchColor,
) -> Option<Action> {
    let id = Id::new("ColorEntry").with((kind, index));
    let ref_fg = ref_fg.unwrap_or(entry.color);
    let ref_bg = ref_bg.unwrap_or(entry.color);

    let is_being_dragged = ui.is_being_dragged(id) && ui.input(|i| i.pointer.primary_down());
    if is_being_dragged {
        DragAndDrop::set_payload(ui.ctx(), ColorEntryDragPayload { kind, index });
        let bg = Color32::from(ref_bg).to_srgba_unmultiplied();
        let bg = Color32::from_rgba_unmultiplied(
            bg[0],
            bg[1],
            bg[2],
            (bg[3] as f32 * 0.5).round() as u8,
        );
        let layer_id = LayerId::new(Order::Tooltip, id);
        let rect = ui
            .scope_builder(UiBuilder::new().layer_id(layer_id), |ui| {
                show_color_entry_frame(ui, kind, index, entry, ref_fg.into(), bg);
            })
            .response
            .rect;
        if let Some(pointer_pos) = ui.pointer_interact_pos() {
            let delta = vec2(0.0, pointer_pos.y - rect.center().y);
            ui.transform_layer_shapes(layer_id, emath::TSTransform::from_translation(delta));
        }
        return None;
    }

    let mut action = None;
    let response = ui
        .scope_builder(
            UiBuilder::new().id(id).sense(Sense::click_and_drag()),
            |ui| {
                if let Some(a) =
                    show_color_entry_frame(ui, kind, index, entry, ref_fg.into(), ref_bg.into())
                {
                    action = Some(a);
                }
            },
        )
        .response;
    if let Some(a) = handle_color_entry_response(
        ui,
        response,
        kind,
        index,
        entry,
        last_edited_color,
        ref_fg,
        ref_bg,
    ) {
        action = Some(a);
    }
    action
}

/// 绘制颜色配置项控件对应的Frame
fn show_color_entry_frame(
    ui: &mut Ui,
    kind: ColorTableKind,
    index: usize,
    entry: &ColorEntry,
    fg: Color32,
    bg: Color32,
) -> Option<Action> {
    const INNER_MARGIN: Vec2 = vec2(12.0, 8.0);

    let mut action = None;
    Frame::NONE
        .fill(bg)
        .inner_margin(INNER_MARGIN)
        .show(ui, |ui| {
            ui.set_min_width(ui.available_width());
            match kind {
                ColorTableKind::Foreground | ColorTableKind::Background => {
                    ui.colored_label(
                        fg,
                        format!(
                            "{} | {} | {} | APCA对比度：{:.2}",
                            entry.oklch,
                            entry.rgb,
                            entry.hex,
                            entry.apca_contrast.expect("未按预期计算APCA对比度")
                        ),
                    );
                }
                _ => {
                    ui.horizontal(|ui| {
                        ui.colored_label(entry.color, "󰝤");
                        ui.colored_label(
                            fg,
                            format!("{} | {} | {}", entry.oklch, entry.rgb, entry.hex),
                        );
                    });
                }
            }
            if let Some(a) = show_color_purpose_tags(ui, kind, index, entry) {
                action = Some(a);
            }
        });
    action
}

/// 处理颜色配置项控件的交互响应
#[expect(clippy::too_many_arguments)]
fn handle_color_entry_response(
    ui: &mut Ui,
    response: Response,
    kind: ColorTableKind,
    index: usize,
    entry: &ColorEntry,
    last_edited_color: OklchColor,
    ref_fg: OklchColor,
    ref_bg: OklchColor,
) -> Option<Action> {
    let mut action: Option<Action> = None;

    if let Some(payload) = response.dnd_hover_payload::<ColorEntryDragPayload>()
        && payload.kind == kind
        && payload.index != index
        && let Some(pos) = ui.pointer_interact_pos()
    {
        // 拖拽其他配置项到当前配置项上方时，绘制移动位置提示线
        let above = pos.y < response.rect.center().y;
        let line_y = if above {
            response.rect.min.y
        } else {
            response.rect.max.y
        };
        ui.painter().line_segment(
            [
                pos2(response.rect.min.x, line_y),
                pos2(response.rect.max.x, line_y),
            ],
            Stroke::new(DRAG_INDICATOR_STROKE_WIDTH, DRAG_INDICATOR_COLOR),
        );

        // 释放拖拽时，请求移动颜色配置项
        if response
            .dnd_release_payload::<ColorEntryDragPayload>()
            .is_some()
        {
            let src_index = payload.index;
            let dst_index = if above { index } else { index + 1 };
            action = Some(Box::new(move |app, ctx| {
                request::move_color(app, ctx, kind, src_index, dst_index);
            }));
        }
    } else if let Some(payload) = response.dnd_hover_payload::<ColorPurposeDragPayload>()
        && payload.kind == kind
        && !(payload.index == index && payload.purpose_index == entry.purposes.len() - 1)
    {
        // 拖拽颜色用途标签到当前配置项上方时，绘制移动位置提示边框
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            Stroke::new(DRAG_INDICATOR_STROKE_WIDTH, DRAG_INDICATOR_COLOR),
            StrokeKind::Inside,
        );

        // 释放拖拽时，请求移动颜色用途标签
        if response
            .dnd_release_payload::<ColorPurposeDragPayload>()
            .is_some()
        {
            let src_index = payload.index;
            let src_purpose_index = payload.purpose_index;
            let dst_index = index;
            let dst_purpose_index = entry.purposes.len();
            action = Some(Box::new(move |app, ctx| {
                request::move_color_purpose(
                    app,
                    ctx,
                    kind,
                    src_index,
                    src_purpose_index,
                    dst_index,
                    dst_purpose_index,
                );
            }));
        }
    } else if response.hovered() {
        // 没有拖拽且指针悬停在控件上方时，绘制可交互提示边框
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            ui.style().visuals.widgets.hovered.bg_stroke,
            StrokeKind::Inside,
        );
    }
    response.context_menu(|ui| {
        if let Some(a) =
            show_color_entry_context_menu(ui, kind, index, entry, ref_fg, ref_bg, last_edited_color)
        {
            action = Some(a);
        }
    });

    action
}

/// 绘制颜色配置项的上下文菜单
fn show_color_entry_context_menu(
    ui: &mut Ui,
    kind: ColorTableKind,
    index: usize,
    entry: &ColorEntry,
    ref_fg: OklchColor,
    ref_bg: OklchColor,
    last_edited_color: OklchColor,
) -> Option<Action> {
    let mut action: Option<Action> = None;
    if matches!(kind, ColorTableKind::Foreground) {
        if ui.button("设置为参考前景色").clicked() {
            let color = entry.color;
            action = Some(Box::new(move |app: &mut App, _ctx| {
                app.set_ref_fg_color(color);
            }));
        }
        ui.separator();
    }
    if matches!(kind, ColorTableKind::Background) {
        if ui.button("设置为参考背景色").clicked() {
            let color = entry.color;
            action = Some(Box::new(move |app: &mut App, _ctx| {
                app.set_ref_bg_color(color);
            }));
        }
        ui.separator();
    }
    if ui.button("修改..").clicked() {
        let color = entry.color;
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog.push(ColorEntryEditDialog::modify(
                kind, index, color, ref_fg, ref_bg,
            ));
        }));
    }
    ui.separator();
    if ui
        .button(("复制：", RichText::new(&entry.oklch).code()))
        .clicked()
    {
        ui.copy_text(entry.oklch.to_owned());
    }
    if ui
        .button(("复制：", RichText::new(&entry.rgb).code()))
        .clicked()
    {
        ui.copy_text(entry.rgb.to_owned());
    }
    if ui
        .button(("复制：", RichText::new(&entry.hex).code()))
        .clicked()
    {
        ui.copy_text(entry.hex.to_owned());
    }
    ui.separator();
    if ui.button("添加颜色用途..").clicked() {
        let purpose_index = entry.purposes.len();
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog
                .push(ColorPurposeEditDialog::add(kind, index, purpose_index));
        }));
    }
    ui.separator();
    if ui.button("在此前添加颜色配置项..").clicked() {
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog.push(ColorEntryEditDialog::add(
                kind,
                index,
                last_edited_color,
                ref_fg,
                ref_bg,
            ));
        }));
    }
    if ui.button("在此后添加颜色配置项..").clicked() {
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog.push(ColorEntryEditDialog::add(
                kind,
                index + 1,
                last_edited_color,
                ref_fg,
                ref_bg,
            ));
        }));
    }
    ui.separator();
    if ui.button("删除..").clicked() {
        let oklch = entry.oklch.to_owned();
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog.push(ConfirmDialog::new(
                format!("确定要删除「{oklch}」吗？"),
                move |app: &mut App, ctx, _frame| {
                    request::remove_color(app, ctx, kind, index);
                },
            ));
        }));
    }
    action
}

/// 绘制颜色配置项的所有颜色用途标签
fn show_color_purpose_tags(
    ui: &mut Ui,
    kind: ColorTableKind,
    index: usize,
    entry: &ColorEntry,
) -> Option<Action> {
    if entry.purposes.is_empty() {
        return None;
    }
    let mut action = None;
    ui.horizontal_wrapped(|ui| {
        for (purpose_index, purpose) in entry.purposes.iter().enumerate() {
            if let Some(a) = show_color_purpose_tag(ui, kind, index, purpose_index, purpose) {
                action = Some(a);
            }
        }
    });
    action
}

/// 绘制颜色用途标签控件
fn show_color_purpose_tag(
    ui: &mut Ui,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: &str,
) -> Option<Action> {
    let id = Id::new("ColorPurposeTag").with((kind, index, purpose_index));
    let bg = ui.visuals().widgets.inactive.bg_fill;
    let is_being_dragged = ui.is_being_dragged(id) && ui.input(|i| i.pointer.primary_down());
    if is_being_dragged {
        DragAndDrop::set_payload(
            ui.ctx(),
            ColorPurposeDragPayload {
                kind,
                index,
                purpose_index,
            },
        );
        let bg = bg.to_srgba_unmultiplied();
        let bg = Color32::from_rgba_unmultiplied(
            bg[0],
            bg[1],
            bg[2],
            (bg[3] as f32 * 0.5).round() as u8,
        );
        let layer_id = LayerId::new(Order::Tooltip, id);
        let rect = ui
            .scope_builder(UiBuilder::new().layer_id(layer_id), |ui| {
                show_color_purpose_tag_frame(ui, purpose, bg);
            })
            .response
            .rect;
        if let Some(pointer_pos) = ui.pointer_interact_pos() {
            let delta = pointer_pos - rect.center();
            ui.transform_layer_shapes(layer_id, emath::TSTransform::from_translation(delta));
        }
        return None;
    }
    let response = ui
        .scope(|ui| {
            show_color_purpose_tag_frame(ui, purpose, bg);
        })
        .response;
    let response = ui.interact(response.rect, id, Sense::click_and_drag());
    handle_color_purpose_tag_response(ui, response, kind, index, purpose_index, purpose)
}

/// 绘制颜色用途标签控件对应的Frame
#[inline]
fn show_color_purpose_tag_frame(ui: &mut Ui, purpose: &str, bg: Color32) {
    ui.add(
        Button::new(purpose)
            .sense(Sense::empty())
            .corner_radius(0)
            .stroke(Stroke::NONE)
            .fill(bg),
    );
}

/// 处理颜色用途标签控件的交互响应
fn handle_color_purpose_tag_response(
    ui: &mut Ui,
    response: Response,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: &str,
) -> Option<Action> {
    let mut action: Option<Action> = None;

    if let Some(payload) = response.dnd_hover_payload::<ColorPurposeDragPayload>()
        && payload.kind == kind
        && !(payload.index == index && payload.purpose_index == purpose_index)
        && let Some(pos) = ui.pointer_interact_pos()
    {
        // 拖拽其他标签到当前标签上方时，绘制移动位置提示线
        let left = pos.x < response.rect.center().x;
        let line_x = if left {
            response.rect.min.x
        } else {
            response.rect.max.x
        };
        ui.painter().line_segment(
            [
                pos2(line_x, response.rect.min.y),
                pos2(line_x, response.rect.max.y),
            ],
            Stroke::new(DRAG_INDICATOR_STROKE_WIDTH, DRAG_INDICATOR_COLOR),
        );

        // 释放拖拽时，请求移动颜色用途标签
        if response
            .dnd_release_payload::<ColorPurposeDragPayload>()
            .is_some()
        {
            let src_index = payload.index;
            let src_purpose_index = payload.purpose_index;
            let dst_index = index;
            let dst_purpose_index = if left {
                purpose_index
            } else {
                purpose_index + 1
            };
            action = Some(Box::new(move |app, ctx| {
                request::move_color_purpose(
                    app,
                    ctx,
                    kind,
                    src_index,
                    src_purpose_index,
                    dst_index,
                    dst_purpose_index,
                );
            }));
        }
    } else if response.hovered() {
        // 没有拖拽且指针悬停在控件上方时，绘制可交互提示边框
        ui.painter().rect_stroke(
            response.rect,
            0.0,
            ui.style().visuals.widgets.hovered.bg_stroke,
            StrokeKind::Inside,
        );
    }
    response.context_menu(|ui| {
        if let Some(a) =
            show_color_purpose_tag_context_menu(ui, kind, index, purpose_index, purpose)
        {
            action = Some(a);
        }
    });

    action
}

/// 绘制颜色用途标签控件的上下文菜单
fn show_color_purpose_tag_context_menu(
    ui: &mut Ui,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: &str,
) -> Option<Action> {
    let mut action: Option<Action> = None;
    if ui.button("修改..").clicked() {
        let purpose = purpose.to_owned();
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog.push(ColorPurposeEditDialog::modify(
                kind,
                index,
                purpose_index,
                purpose,
            ));
        }));
    }
    ui.separator();
    if ui
        .button(("复制：", RichText::new(purpose).code()))
        .clicked()
    {
        ui.copy_text(purpose.to_owned());
    }
    let full_text = format!("{}{purpose}", kind.prefix());
    if ui
        .button(("复制：", RichText::new(&full_text).code()))
        .clicked()
    {
        ui.copy_text(full_text);
    }
    ui.separator();
    if ui.button("在此前添加颜色用途..").clicked() {
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog
                .push(ColorPurposeEditDialog::add(kind, index, purpose_index));
        }));
    }
    if ui.button("在此后添加颜色用途..").clicked() {
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog
                .push(ColorPurposeEditDialog::add(kind, index, purpose_index + 1));
        }));
    }
    ui.separator();
    if ui.button("删除..").clicked() {
        let purpose = purpose.to_owned();
        action = Some(Box::new(move |app: &mut App, _ctx| {
            app.dialog.push(ConfirmDialog::new(
                format!("确定要删除「{purpose}」吗？"),
                move |app: &mut App, ctx, _frame| {
                    request::remove_color_purpose(app, ctx, kind, index, purpose_index);
                },
            ));
        }));
    }
    action
}
