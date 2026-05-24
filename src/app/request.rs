use anyhow::Context as _;
use eframe::egui::Context;
use rfd::FileDialog;

use crate::{
    app::{
        App,
        history::*,
        ui::{AlertDialog, ConfirmDialog, ExportDialog},
    },
    utils::{ColorEntry, ColorScheme, ColorTableKind, OklchColor, process_template},
};

mod redo;
mod undo;

pub use redo::*;
pub use undo::*;

/// 处理新建配色方案的请求
pub fn new_scheme(app: &mut App, ctx: &Context) {
    if app.scheme_dirty {
        app.dialog.push(ConfirmDialog::new(
            "配色方案的变更尚未保存，确定要新建方案吗？",
            |app, ctx, _frame| {
                app.new_scheme(ctx);
            },
        ));
        return;
    }
    app.new_scheme(ctx);
}

/// 处理打开配色方案的请求
pub fn open_scheme(app: &mut App, ctx: &Context, frame: &mut eframe::Frame) {
    fn do_open(app: &mut App, ctx: &Context, frame: &mut eframe::Frame) {
        if let Some(path) = FileDialog::new()
            .set_title("打开配色方案文件")
            .set_parent(frame)
            .add_filter("TOML文件", &["toml"])
            .pick_file()
        {
            match ColorScheme::load_from(&path).context("无法打开指定的配色方案文件") {
                Ok(s) => {
                    app.set_scheme(ctx, s, Some(path));
                }
                Err(e) => {
                    app.toast.error(format!("{e:#}"));
                }
            }
        }
    }

    if app.scheme_dirty {
        app.dialog.push(ConfirmDialog::new(
            "配色方案的变更尚未保存，确定要打开其他方案吗？",
            |app, ctx, frame| {
                do_open(app, ctx, frame);
            },
        ));
        return;
    }
    do_open(app, ctx, frame);
}

/// 处理保存配色方案的请求
pub fn save_scheme(app: &mut App, ctx: &Context, frame: &mut eframe::Frame) {
    match &app.scheme_file_path {
        Some(p) => match app.scheme.save_to(p).context("保存配色方案数据失败") {
            Ok(_) => {
                app.mark_scheme_clean(ctx);
                app.toast.info_auto_dismiss("配色方案数据已保存");
            }
            Err(e) => {
                app.toast.error(format!("{e:#}"));
            }
        },
        None => {
            save_scheme_to_other_path(app, ctx, frame);
        }
    }
}

/// 处理将配色方案保存到其他路径的请求
pub fn save_scheme_to_other_path(app: &mut App, ctx: &Context, frame: &mut eframe::Frame) {
    if let Some(path) = FileDialog::new()
        .set_title("保存配色方案文件")
        .set_parent(frame)
        .add_filter("TOML文件", &["toml"])
        .save_file()
    {
        match app.scheme.save_to(&path).context("保存配色方案数据失败") {
            Ok(_) => {
                app.scheme_file_path = Some(path);
                app.mark_scheme_clean(ctx);
                app.toast.info_auto_dismiss("配色方案数据已保存");
            }
            Err(e) => app.toast.error(format!("{e:#}")),
        }
    }
}

/// 处理导出配色方案的请求
pub fn export_scheme(app: &mut App, frame: &mut eframe::Frame) {
    if let Some(path) = FileDialog::new()
        .set_title("选择模板文件")
        .set_parent(frame)
        .pick_file()
    {
        match process_template(path, &app.scheme) {
            Ok(result) => app.dialog.push(ExportDialog::new(result)),
            Err(e) => app.dialog.push(AlertDialog::new(format!("{e:#}"))),
        }
    }
}

#[inline]
fn add_color_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    color: OklchColor,
    purposes: Vec<String>,
) -> anyhow::Result<()> {
    let table = app.scheme.get_table_mut(kind);
    table.add(index, color, purposes)?;
    app.mark_scheme_dirty(ctx);
    Ok(())
}

#[inline]
fn modify_color_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    color: OklchColor,
) -> anyhow::Result<Option<OklchColor>> {
    let table = app.scheme.get_table_mut(kind);
    let old_color = table.modify_color(index, color)?;
    if old_color.is_some() {
        app.mark_scheme_dirty(ctx);
    }
    Ok(old_color)
}

#[inline]
fn move_color_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    from: usize,
    to: usize,
) -> anyhow::Result<bool> {
    let table = app.scheme.get_table_mut(kind);
    let moved = table.move_color(from, to)?;
    if moved {
        app.mark_scheme_dirty(ctx);
    }
    Ok(moved)
}

#[inline]
fn remove_color_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
) -> anyhow::Result<ColorEntry> {
    let table = app.scheme.get_table_mut(kind);
    let entry = table.remove(index)?;
    app.mark_scheme_dirty(ctx);
    Ok(entry)
}

/// 处理添加颜色配置项的请求
pub fn add_color(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    color: OklchColor,
) -> anyhow::Result<()> {
    add_color_inner(app, ctx, kind, index, color, Default::default())?;
    app.history.push_undo(
        Operation::AddColor(AddColorOpArg { kind, index, color }),
        true,
    );
    app.set_last_edited_color(kind, color);
    Ok(())
}

/// 处理修改颜色配置项的请求
pub fn modify_color(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    color: OklchColor,
) -> anyhow::Result<()> {
    let Some(before) = modify_color_inner(app, ctx, kind, index, color)? else {
        return Ok(());
    };
    app.history.push_undo(
        Operation::ModifyColor(ModifyColorOpArg {
            kind,
            index,
            before,
            after: color,
        }),
        true,
    );
    app.set_last_edited_color(kind, color);
    Ok(())
}

/// 处理移动颜色配置项的请求
pub fn move_color(app: &mut App, ctx: &Context, kind: ColorTableKind, from: usize, to: usize) {
    match move_color_inner(app, ctx, kind, from, to).context("移动颜色配置项时发生错误")
    {
        Ok(true) => {
            app.history.push_undo(
                Operation::MoveColor(MoveColorOpArg { kind, from, to }),
                true,
            );
        }
        Ok(false) => {}
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

/// 处理删除颜色配置项的请求
pub fn remove_color(app: &mut App, ctx: &Context, kind: ColorTableKind, index: usize) {
    match remove_color_inner(app, ctx, kind, index).context("删除颜色配置项时发生错误")
    {
        Ok(entry) => {
            app.history.push_undo(
                Operation::RemoveColor(RemoveColorOpArg { kind, index, entry }),
                true,
            );
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

#[inline]
fn add_color_purpose_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: String,
) -> anyhow::Result<()> {
    let purpose = {
        let trimmed = purpose.trim();
        if trimmed.len() != purpose.len() {
            trimmed.to_string()
        } else {
            purpose
        }
    };
    if purpose.contains("|") {
        anyhow::bail!("颜色用途字符串不能包含字符'|'");
    }
    if purpose == "$" {
        anyhow::bail!("颜色用途字符串不能为'$'");
    }
    let table = app.scheme.get_table_mut(kind);
    table.add_purpose(index, purpose_index, purpose)?;
    app.mark_scheme_dirty(ctx);
    Ok(())
}

#[inline]
fn modify_color_purpose_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: String,
) -> anyhow::Result<Option<String>> {
    let table = app.scheme.get_table_mut(kind);
    let old_purpose = table.modify_purpose(index, purpose_index, purpose)?;
    if old_purpose.is_some() {
        app.mark_scheme_dirty(ctx);
    }
    Ok(old_purpose)
}

#[inline]
fn move_color_purpose_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    src_index: usize,
    src_purpose_index: usize,
    dst_index: usize,
    dst_purpose_index: usize,
) -> anyhow::Result<bool> {
    let table = app.scheme.get_table_mut(kind);

    let moved = if src_index == dst_index {
        table.move_purpose(src_index, src_purpose_index, dst_purpose_index)?
    } else {
        let purpose = table.remove_purpose(src_index, src_purpose_index)?;
        table.add_purpose(dst_index, dst_purpose_index, purpose)?;
        true
    };

    if moved {
        app.mark_scheme_dirty(ctx);
    }
    Ok(moved)
}

#[inline]
fn remove_color_purpose_inner(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
) -> anyhow::Result<String> {
    let table = app.scheme.get_table_mut(kind);
    let purpose = table.remove_purpose(index, purpose_index)?;
    app.mark_scheme_dirty(ctx);
    Ok(purpose)
}

/// 处理添加颜色用途的请求
pub fn add_color_purpose(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: String,
) -> anyhow::Result<()> {
    add_color_purpose_inner(app, ctx, kind, index, purpose_index, purpose.clone())
        .context("添加颜色用途时发生错误")?;
    app.history.push_undo(
        Operation::AddColorPurpose(AddColorPurposeOpArg {
            kind,
            index,
            purpose_index,
            purpose,
        }),
        true,
    );
    Ok(())
}

/// 处理修改颜色用途的请求
pub fn modify_color_purpose(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
    purpose: String,
) -> anyhow::Result<()> {
    let Some(before) =
        modify_color_purpose_inner(app, ctx, kind, index, purpose_index, purpose.clone())
            .context("修改颜色用途时发生错误")?
    else {
        return Ok(());
    };
    app.history.push_undo(
        Operation::ModifyColorPurpose(ModifyColorPurposeOpArg {
            kind,
            index,
            purpose_index,
            before,
            after: purpose,
        }),
        true,
    );
    Ok(())
}

/// 处理移动颜色用途标签的请求
pub fn move_color_purpose(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    src_index: usize,
    src_purpose_index: usize,
    dst_index: usize,
    dst_purpose_index: usize,
) {
    match move_color_purpose_inner(
        app,
        ctx,
        kind,
        src_index,
        src_purpose_index,
        dst_index,
        dst_purpose_index,
    )
    .context("移动颜色用途时发生错误")
    {
        Ok(true) => {
            app.history.push_undo(
                Operation::MoveColorPurpose(MoveColorPurposeOpArg {
                    kind,
                    before_index: src_index,
                    before_purpose_index: src_purpose_index,
                    after_index: dst_index,
                    after_purpose_index: dst_purpose_index,
                }),
                true,
            );
        }
        Ok(false) => {}
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

/// 处理删除颜色用途标签的请求
pub fn remove_color_purpose(
    app: &mut App,
    ctx: &Context,
    kind: ColorTableKind,
    index: usize,
    purpose_index: usize,
) {
    match remove_color_purpose_inner(app, ctx, kind, index, purpose_index)
        .context("删除颜色用途时发生错误")
    {
        Ok(purpose) => {
            app.history.push_undo(
                Operation::RemoveColorPurpose(RemoveColorPurposeOpArg {
                    kind,
                    index,
                    purpose_index,
                    purpose,
                }),
                true,
            );
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}
