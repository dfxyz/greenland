use eframe::egui::{Align, Context, Id, Key, Layout, Modal, Modifiers, RichText, Ui};

use crate::app::App;

mod alert;
mod color_entry_edit;
mod color_purpose_edit;
mod confirm;
mod export;
mod ref_color;

pub use alert::*;
pub use color_entry_edit::*;
pub use color_purpose_edit::*;
pub use confirm::*;
pub use export::*;
pub use ref_color::*;

#[cfg(debug_assertions)]
mod test;
#[cfg(debug_assertions)]
pub use test::*;

const MIN_WIDTH: f32 = 300.0;
const MAX_WIDTH: f32 = 600.0;
const SPACE_BEFORE_BOTTOM_BUTTONS: f32 = 12.0;

type BoxedDialogClosure = Box<dyn FnOnce(Box<dyn Dialog>, &mut App, &Context, &mut eframe::Frame)>;

#[derive(Default)]
pub enum DialogResponse {
    /// 对话框保持打开状态
    #[default]
    KeepOpen,
    /// 关闭对话框
    Close,
    /// 关闭对话框，并执行指定的闭包；闭包入参中带有对话框的Box指针，可以在闭包执行时通过压栈操作“取消”关闭对话框的操作
    CloseWithClosure(BoxedDialogClosure),
}

impl<D> From<D> for DialogResponse
where
    D: FnOnce(Box<dyn Dialog>, &mut App, &Context, &mut eframe::Frame) + 'static,
{
    #[inline]
    fn from(value: D) -> Self {
        Self::CloseWithClosure(Box::new(value))
    }
}

pub trait Dialog {
    fn id(&self) -> usize;

    fn set_id(&mut self, id: usize);

    /// 绘制对话框，并根据用户的操作返回[`DialogResponse``]
    fn show(&mut self, ui: &mut Ui) -> DialogResponse;
}

macro_rules! define_id_methods {
    ($name:ident) => {
        #[inline]
        fn id(&self) -> usize {
            self.$name
        }

        #[inline]
        fn set_id(&mut self, id: usize) {
            self.$name = id;
        }
    };
}
use define_id_methods;

/// 对话框管理器
#[derive(Default)]
pub struct DialogManager {
    next_id: usize,
    stack: Vec<Box<dyn Dialog>>,
}

impl DialogManager {
    fn ensure_id(&mut self, dialog: &mut dyn Dialog) {
        if dialog.id() == 0 {
            if self.next_id == 0 {
                self.next_id = 1;
            }
            dialog.set_id(self.next_id);
            self.next_id = self.next_id.wrapping_add(1);
        }
    }

    /// 判断对话框栈是否为空
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    /// 向对话框栈中压入一个对话框实现
    #[inline]
    pub fn push<D: Dialog + 'static>(&mut self, mut dialog: D) {
        self.ensure_id(&mut dialog);
        self.stack.push(Box::new(dialog));
    }

    /// 向对话框栈中压入一个已在堆上分配的对话框实现
    #[inline]
    pub fn push_boxed(&mut self, mut dialog: Box<dyn Dialog>) {
        self.ensure_id(dialog.as_mut());
        self.stack.push(dialog);
    }

    #[inline]
    fn take_all(&mut self) -> Vec<Box<dyn Dialog>> {
        std::mem::take(&mut self.stack)
    }
}

/// 绘制对话框
pub fn show(app: &mut App, ui: &mut Ui, frame: &mut eframe::Frame) {
    let len = app.dialog.stack.len();
    if len == 0 {
        return;
    }
    let dialogs = app.dialog.take_all();
    for (index, mut dialog) in dialogs.into_iter().enumerate() {
        let mut response = DialogResponse::default();
        Modal::new(Id::new(("Dialog", dialog.id()))).show(ui, |ui| {
            response = dialog.show(ui);
        });
        if index == len - 1 && ui.input_mut(|i| i.consume_key_exact(Modifiers::NONE, Key::Escape)) {
            response = DialogResponse::Close;
        }
        match response {
            DialogResponse::KeepOpen => {
                app.dialog.push_boxed(dialog);
            }
            DialogResponse::Close => {}
            DialogResponse::CloseWithClosure(closure) => {
                closure(dialog, app, ui.ctx(), frame);
            }
        }
    }
}

/// 在对话框顶部绘制标题，顺便设置对话框的宽度范围
fn show_title<S: Into<RichText>>(ui: &mut Ui, title: S) {
    ui.heading(title);
    ui.separator();
    ui.set_min_width(MIN_WIDTH);
    ui.set_max_width(MAX_WIDTH);
}

/// 在对话框底部绘制关闭按钮；返回关闭按钮是否被按下
fn show_close_button(ui: &mut Ui) -> bool {
    ui.shrink_width_to_current();
    ui.add_space(SPACE_BEFORE_BOTTOM_BUTTONS);

    let mut clicked = false;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if ui.button("关闭").clicked() {
            clicked = true;
        }
    });
    clicked
}

/// 在对话框底部绘制确认、取消按钮
/// - 如果确认按钮被按下，返回`Some(true)`
/// - 如果取消按钮被按下，返回`Some(false)`
/// - 其他情况返回`None`
fn show_confirm_cancel_buttons(ui: &mut Ui) -> Option<bool> {
    ui.shrink_width_to_current();
    ui.add_space(SPACE_BEFORE_BOTTOM_BUTTONS);

    let mut result = None;
    ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
        if ui.button("确认").clicked() {
            result = Some(true);
        }
        if ui.button("取消").clicked() {
            result = Some(false);
        }
    });
    result
}
