use std::collections::VecDeque;

use crate::utils::OklchColor;

const RECENT_COLOR_CAPACITY: usize = 5;

/// 参考前景色/背景色相关的数据
pub struct RefColor {
    /// 参考前景色
    fg: OklchColor,
    /// 参考背景色
    bg: OklchColor,
    /// 最近用过的其他参考前景色
    recent_fg_colors: VecDeque<OklchColor>,
    /// 最近用过的其他参考背景色
    recent_bg_colors: VecDeque<OklchColor>,
}

impl Default for RefColor {
    fn default() -> Self {
        Self {
            fg: OklchColor::BLACK,
            bg: OklchColor::WHITE,
            recent_fg_colors: VecDeque::with_capacity(RECENT_COLOR_CAPACITY),
            recent_bg_colors: VecDeque::with_capacity(RECENT_COLOR_CAPACITY),
        }
    }
}

impl RefColor {
    /// 获取当前的参考前景色
    #[inline]
    pub fn fg(&self) -> OklchColor {
        self.fg
    }

    /// 获取当前的参考背景色
    #[inline]
    pub fn bg(&self) -> OklchColor {
        self.bg
    }

    /// 获取最近用过的参考前景色列表的迭代器
    #[inline]
    pub fn recent_fg_colors(&self) -> impl Iterator<Item = OklchColor> {
        self.recent_fg_colors.iter().cloned()
    }

    /// 获取最近用过的参考背景色列表的迭代器
    #[inline]
    pub fn recent_bg_colors(&self) -> impl Iterator<Item = OklchColor> {
        self.recent_bg_colors.iter().cloned()
    }

    /// 设置参考前景色
    #[inline]
    pub fn set_fg(&mut self, fg: OklchColor) -> bool {
        if self.fg == fg {
            return false;
        }
        self.fg = fg;
        Self::update_recent_colors(self.fg, &mut self.recent_fg_colors);
        true
    }

    /// 设置参考背景色
    #[inline]
    pub fn set_bg(&mut self, bg: OklchColor) -> bool {
        if self.bg == bg {
            return false;
        }
        self.bg = bg;
        Self::update_recent_colors(self.bg, &mut self.recent_bg_colors);
        true
    }

    fn update_recent_colors(color: OklchColor, recent_colors: &mut VecDeque<OklchColor>) {
        recent_colors.retain(|c| c != &color);
        recent_colors.push_front(color);
        recent_colors.truncate(RECENT_COLOR_CAPACITY);
    }
}
