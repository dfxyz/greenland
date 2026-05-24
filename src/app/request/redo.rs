use anyhow::Context as _;
use eframe::egui::Context;

use crate::app::{App, history::*};

/// 处理重做请求
pub fn redo(app: &mut App, ctx: &Context) {
    let Some(op) = app.history.pop_redo() else {
        return;
    };
    match op {
        Operation::AddColor(arg) => redo_add_color(app, ctx, arg),
        Operation::ModifyColor(arg) => redo_modify_color(app, ctx, arg),
        Operation::MoveColor(arg) => redo_move_color(app, ctx, arg),
        Operation::RemoveColor(arg) => redo_remove_color(app, ctx, arg),
        Operation::AddColorPurpose(arg) => redo_add_color_purpose(app, ctx, arg),
        Operation::ModifyColorPurpose(arg) => redo_modify_color_purpose(app, ctx, arg),
        Operation::MoveColorPurpose(arg) => redo_move_color_purpose(app, ctx, arg),
        Operation::RemoveColorPurpose(arg) => redo_remove_color_purpose(app, ctx, arg),
    }
}

fn redo_add_color(app: &mut App, ctx: &Context, arg: AddColorOpArg) {
    match super::add_color_inner(app, ctx, arg.kind, arg.index, arg.color, Default::default())
        .with_context(|| format!("无法重做「在{}中添加颜色配置项」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已重做「在{}中添加颜色配置项」", arg.kind.desc()));
            app.history.push_undo(Operation::AddColor(arg), false);
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn redo_modify_color(app: &mut App, ctx: &Context, arg: ModifyColorOpArg) {
    match super::modify_color_inner(app, ctx, arg.kind, arg.index, arg.after)
        .with_context(|| format!("无法重做「在{}中修改颜色配置项的数值」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast.info_auto_dismiss(format!(
                "已重做「在{}中修改颜色配置项的数值」",
                arg.kind.desc()
            ));
            app.history.push_undo(Operation::ModifyColor(arg), false);
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn redo_move_color(app: &mut App, ctx: &Context, arg: MoveColorOpArg) {
    match super::move_color_inner(app, ctx, arg.kind, arg.from, arg.to)
        .with_context(|| format!("无法重做「在{}中移动颜色配置项」", arg.kind.desc()))
    {
        Ok(true) => {
            app.toast
                .info_auto_dismiss(format!("已重做「在{}中移动颜色配置项」", arg.kind.desc()));
            app.history.push_undo(Operation::MoveColor(arg), false);
        }
        Ok(false) => {
            app.toast.warn_auto_dismiss(format!(
                "无法重做「在{}中移动颜色配置项」；未产生实质性的修改",
                arg.kind.desc()
            ));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn redo_remove_color(app: &mut App, ctx: &Context, arg: RemoveColorOpArg) {
    match super::remove_color_inner(app, ctx, arg.kind, arg.index)
        .with_context(|| format!("无法重做「在{}中删除颜色配置项」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已重做「在{}中删除颜色配置项」", arg.kind.desc()));
            app.history.push_undo(Operation::RemoveColor(arg), false);
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn redo_add_color_purpose(app: &mut App, ctx: &Context, arg: AddColorPurposeOpArg) {
    match super::add_color_purpose_inner(
        app,
        ctx,
        arg.kind,
        arg.index,
        arg.purpose_index,
        arg.purpose.clone(),
    )
    .with_context(|| format!("无法重做「在{}中添加颜色用途」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已重做「在{}中添加颜色用途」", arg.kind.desc()));
            app.history
                .push_undo(Operation::AddColorPurpose(arg), false);
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn redo_modify_color_purpose(app: &mut App, ctx: &Context, arg: ModifyColorPurposeOpArg) {
    match super::modify_color_purpose_inner(
        app,
        ctx,
        arg.kind,
        arg.index,
        arg.purpose_index,
        arg.after.clone(),
    )
    .with_context(|| format!("无法重做「在{}中修改颜色用途」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已重做「在{}中修改颜色用途」", arg.kind.desc()));
            app.history
                .push_undo(Operation::ModifyColorPurpose(arg), false);
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn redo_move_color_purpose(app: &mut App, ctx: &Context, arg: MoveColorPurposeOpArg) {
    match super::move_color_purpose_inner(
        app,
        ctx,
        arg.kind,
        arg.before_index,
        arg.before_purpose_index,
        arg.after_index,
        arg.after_purpose_index,
    )
    .with_context(|| format!("无法重做「在{}中移动颜色用途」", arg.kind.desc()))
    {
        Ok(true) => {
            app.toast
                .info_auto_dismiss(format!("已重做「在{}中移动颜色用途」", arg.kind.desc()));
            app.history
                .push_undo(Operation::MoveColorPurpose(arg), false);
        }
        Ok(false) => {
            app.toast.warn_auto_dismiss(format!(
                "无法重做「在{}中移动颜色用途」；未产生实质性的修改",
                arg.kind.desc()
            ));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn redo_remove_color_purpose(app: &mut App, ctx: &Context, arg: RemoveColorPurposeOpArg) {
    match super::remove_color_purpose_inner(app, ctx, arg.kind, arg.index, arg.purpose_index)
        .with_context(|| format!("无法重做「在{}中删除颜色用途」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已重做「在{}中删除颜色用途」", arg.kind.desc()));
            app.history
                .push_undo(Operation::RemoveColorPurpose(arg), false);
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}
