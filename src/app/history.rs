use std::collections::VecDeque;

use crate::utils::{ColorEntry, ColorTableKind, OklchColor};

const CAPACITY: usize = 16;

/// 用户操作的历史记录，用于实现撤销和重做功能
pub struct OperationHistory {
    undo_stack: VecDeque<Operation>,
    redo_stack: VecDeque<Operation>,
}

impl Default for OperationHistory {
    fn default() -> Self {
        Self {
            undo_stack: VecDeque::with_capacity(CAPACITY),
            redo_stack: VecDeque::with_capacity(CAPACITY),
        }
    }
}

impl OperationHistory {
    /// 清空
    #[inline]
    pub fn clear(&mut self) {
        self.undo_stack.clear();
        self.redo_stack.clear();
    }

    /// 检查当前是否有可撤销的操作
    #[inline]
    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    /// 检查当前是否有可重做的操作
    #[inline]
    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }

    /// 向撤销栈中压入一个操作记录
    pub fn push_undo(&mut self, op: Operation, clear_redo_stack: bool) {
        if clear_redo_stack {
            self.redo_stack.clear();
        }
        if self.undo_stack.len() >= CAPACITY {
            self.undo_stack.pop_front();
        }
        self.undo_stack.push_back(op);
    }

    /// 向重做栈中压入一个操作记录
    #[inline]
    pub fn push_redo(&mut self, op: Operation) {
        // 不需要检查容量，因为重做栈中的操作记录总是来自撤销栈
        self.redo_stack.push_back(op);
    }

    /// 从撤销栈中弹出一个操作记录
    #[inline]
    pub fn pop_undo(&mut self) -> Option<Operation> {
        self.undo_stack.pop_back()
    }

    /// 从重做栈中弹出一个操作记录
    #[inline]
    pub fn pop_redo(&mut self) -> Option<Operation> {
        self.redo_stack.pop_back()
    }
}

pub enum Operation {
    AddColor(AddColorOpArg),
    ModifyColor(ModifyColorOpArg),
    MoveColor(MoveColorOpArg),
    RemoveColor(RemoveColorOpArg),
    AddColorPurpose(AddColorPurposeOpArg),
    ModifyColorPurpose(ModifyColorPurposeOpArg),
    MoveColorPurpose(MoveColorPurposeOpArg),
    RemoveColorPurpose(RemoveColorPurposeOpArg),
}
pub struct AddColorOpArg {
    pub kind: ColorTableKind,
    pub index: usize,
    pub color: OklchColor,
}
pub struct ModifyColorOpArg {
    pub kind: ColorTableKind,
    pub index: usize,
    pub before: OklchColor,
    pub after: OklchColor,
}
pub struct MoveColorOpArg {
    pub kind: ColorTableKind,
    pub from: usize,
    pub to: usize,
}
pub struct RemoveColorOpArg {
    pub kind: ColorTableKind,
    pub index: usize,
    pub entry: ColorEntry,
}
pub struct AddColorPurposeOpArg {
    pub kind: ColorTableKind,
    pub index: usize,
    pub purpose_index: usize,
    pub purpose: String,
}
pub struct ModifyColorPurposeOpArg {
    pub kind: ColorTableKind,
    pub index: usize,
    pub purpose_index: usize,
    pub before: String,
    pub after: String,
}
pub struct MoveColorPurposeOpArg {
    pub kind: ColorTableKind,
    pub before_index: usize,
    pub before_purpose_index: usize,
    pub after_index: usize,
    pub after_purpose_index: usize,
}
pub struct RemoveColorPurposeOpArg {
    pub kind: ColorTableKind,
    pub index: usize,
    pub purpose_index: usize,
    pub purpose: String,
}
