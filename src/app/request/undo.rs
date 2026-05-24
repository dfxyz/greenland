use anyhow::Context as _;
use eframe::egui::Context;

use crate::app::{App, history::*};

/// 处理撤销请求
pub fn undo(app: &mut App, ctx: &Context) {
    let Some(op) = app.history.pop_undo() else {
        return;
    };
    match op {
        Operation::AddColor(arg) => undo_add_color(app, ctx, arg),
        Operation::ModifyColor(arg) => undo_modify_color(app, ctx, arg),
        Operation::MoveColor(arg) => undo_move_color(app, ctx, arg),
        Operation::RemoveColor(arg) => undo_remove_color(app, ctx, arg),
        Operation::AddColorPurpose(arg) => undo_add_color_purpose(app, ctx, arg),
        Operation::ModifyColorPurpose(arg) => undo_modify_color_purpose(app, ctx, arg),
        Operation::MoveColorPurpose(arg) => undo_move_color_purpose(app, ctx, arg),
        Operation::RemoveColorPurpose(arg) => undo_remove_color_purpose(app, ctx, arg),
    };
}

fn undo_add_color(app: &mut App, ctx: &Context, arg: AddColorOpArg) {
    match super::remove_color_inner(app, ctx, arg.kind, arg.index)
        .with_context(|| format!("无法撤销「在{}中添加颜色配置项」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已撤销「在{}中添加颜色配置项」", arg.kind.desc()));
            app.history.push_redo(Operation::AddColor(arg));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn undo_modify_color(app: &mut App, ctx: &Context, arg: ModifyColorOpArg) {
    match super::modify_color_inner(app, ctx, arg.kind, arg.index, arg.before)
        .with_context(|| format!("无法撤销「在{}中修改颜色配置项的数值」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast.info_auto_dismiss(format!(
                "已撤销「在{}中修改颜色配置项的数值」",
                arg.kind.desc()
            ));
            app.history.push_redo(Operation::ModifyColor(arg));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn undo_move_color(app: &mut App, ctx: &Context, arg: MoveColorOpArg) {
    let undo_from = if arg.from < arg.to {
        arg.to - 1
    } else {
        arg.to
    };
    let undo_to = if undo_from < arg.from {
        arg.from + 1
    } else {
        arg.from
    };
    match super::move_color_inner(app, ctx, arg.kind, undo_from, undo_to)
        .with_context(|| format!("无法撤销「在{}中移动颜色配置项」", arg.kind.desc()))
    {
        Ok(true) => {
            app.toast
                .info_auto_dismiss(format!("已撤销「在{}中移动颜色配置项」", arg.kind.desc()));
            app.history.push_redo(Operation::MoveColor(arg));
        }
        Ok(false) => {
            app.toast.warn_auto_dismiss(format!(
                "无法撤销「在{}中移动颜色配置项」；未产生实质性的修改",
                arg.kind.desc()
            ));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn undo_remove_color(app: &mut App, ctx: &Context, arg: RemoveColorOpArg) {
    match super::add_color_inner(
        app,
        ctx,
        arg.kind,
        arg.index,
        arg.entry.color,
        arg.entry.purposes.clone(),
    )
    .with_context(|| format!("无法撤销「在{}中删除颜色配置项」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已撤销「在{}中删除颜色配置项」", arg.kind.desc()));
            app.history.push_redo(Operation::RemoveColor(arg));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn undo_add_color_purpose(app: &mut App, ctx: &Context, arg: AddColorPurposeOpArg) {
    match super::remove_color_purpose_inner(app, ctx, arg.kind, arg.index, arg.purpose_index)
        .with_context(|| format!("无法撤销「在{}中添加颜色用途」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已撤销「在{}中添加颜色用途」", arg.kind.desc()));
            app.history.push_redo(Operation::AddColorPurpose(arg));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn undo_modify_color_purpose(app: &mut App, ctx: &Context, arg: ModifyColorPurposeOpArg) {
    match super::modify_color_purpose_inner(
        app,
        ctx,
        arg.kind,
        arg.index,
        arg.purpose_index,
        arg.before.clone(),
    )
    .with_context(|| format!("无法撤销「在{}中修改颜色用途」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已撤销「在{}中修改颜色用途」", arg.kind.desc()));
            app.history.push_redo(Operation::ModifyColorPurpose(arg));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn undo_move_color_purpose(app: &mut App, ctx: &Context, arg: MoveColorPurposeOpArg) {
    let src_index = arg.after_index;
    let mut src_purpose_index = arg.after_purpose_index;
    let dst_index = arg.before_index;
    let mut dst_purpose_index = arg.before_purpose_index;
    if arg.before_index == arg.after_index {
        src_purpose_index = if arg.before_purpose_index < arg.after_purpose_index {
            arg.after_purpose_index - 1
        } else {
            arg.after_purpose_index
        };
        dst_purpose_index = if src_purpose_index < arg.before_purpose_index {
            arg.before_purpose_index + 1
        } else {
            arg.before_purpose_index
        };
    }
    match super::move_color_purpose_inner(
        app,
        ctx,
        arg.kind,
        src_index,
        src_purpose_index,
        dst_index,
        dst_purpose_index,
    )
    .with_context(|| format!("无法撤销「在{}中移动颜色用途」", arg.kind.desc()))
    {
        Ok(true) => {
            app.toast
                .info_auto_dismiss(format!("已撤销「在{}中移动颜色用途」", arg.kind.desc()));
            app.history.push_redo(Operation::MoveColorPurpose(arg));
        }
        Ok(false) => {
            app.toast.warn_auto_dismiss(format!(
                "无法撤销「在{}中移动颜色用途」；未产生实质性的修改",
                arg.kind.desc()
            ));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}

fn undo_remove_color_purpose(app: &mut App, ctx: &Context, arg: RemoveColorPurposeOpArg) {
    match super::add_color_purpose_inner(
        app,
        ctx,
        arg.kind,
        arg.index,
        arg.purpose_index,
        arg.purpose.clone(),
    )
    .with_context(|| format!("无法撤销「在{}中删除颜色用途」", arg.kind.desc()))
    {
        Ok(_) => {
            app.toast
                .info_auto_dismiss(format!("已撤销「在{}中删除颜色用途」", arg.kind.desc()));
            app.history.push_redo(Operation::RemoveColorPurpose(arg));
        }
        Err(e) => {
            app.toast.error_auto_dismiss(format!("{e:#}"));
        }
    }
}
