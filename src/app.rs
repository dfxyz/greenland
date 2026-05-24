use std::{fs::File, io::Read, path::PathBuf};

use anyhow::Context as _;
use eframe::egui::{Context, ViewportBuilder, ViewportCommand};

use crate::utils::{ColorScheme, ColorTableKind, OklchColor};

mod history;
mod ref_color;
mod request;
mod ui;

pub fn run() -> anyhow::Result<()> {
    eframe::run_native(
        "dfxyz.GreenlandSchemeEditor",
        eframe::NativeOptions {
            viewport: ViewportBuilder::default()
                .with_inner_size([1000.0, 800.0])
                .with_maximize_button(false)
                .with_resizable(false),
            ..Default::default()
        },
        Box::new(|cc| {
            let ctx = &cc.egui_ctx;
            ctx.all_styles_mut(|style| {
                style.interaction.selectable_labels = false;
                #[cfg(debug_assertions)]
                {
                    style.debug.debug_on_hover_with_all_modifiers = true;
                    style.debug.hover_shows_next = true;
                }
            });

            let mut app = App::default();
            app.open_default_file(ctx);
            Ok(Box::new(app))
        }),
    )
    .context("无法启动GUI")
}

#[derive(Default)]
pub struct App {
    quit_confirmed: bool,
    scheme: ColorScheme,
    scheme_file_path: Option<PathBuf>,
    scheme_dirty: bool,
    ref_color: ref_color::RefColor,
    current_table_kind: ColorTableKind,
    last_edited_fg_color: Option<OklchColor>,
    last_edited_bg_color: Option<OklchColor>,
    last_edited_marker_color: Option<OklchColor>,
    last_edited_other_color: Option<OklchColor>,
    history: history::OperationHistory,
    dialog: ui::DialogManager,
    toast: ui::ToastManager,
}

// 配色方案数据相关方法
impl App {
    /// 打开默认配色方案文件
    pub fn open_default_file(&mut self, ctx: &Context) {
        fn open() -> anyhow::Result<(PathBuf, ColorScheme)> {
            let path = std::env::current_dir()
                .context("无法获取当前工作目录")?
                .join(super::DEFAULT_SCHEME_FILENAME);
            #[allow(clippy::suspicious_open_options)] // 打开已有文件，或创建空文件
            let mut file = File::options()
                .read(true)
                .write(true)
                .create(true)
                .open(&path)
                .context("无法打开默认配色方案文件")?;
            let mut content = String::new();
            file.read_to_string(&mut content)
                .context("无法读取默认配色方案文件的内容")?;
            let scheme =
                ColorScheme::from_toml(&content).context("无法从默认配色方案文件中加载数据")?;
            Ok((path, scheme))
        }
        match open() {
            Ok((p, s)) => {
                self.set_scheme(ctx, s, Some(p));
            }
            Err(e) => {
                self.toast.error(format!("{e:#}"));
            }
        }
    }

    /// 新建配色方案
    #[inline]
    fn new_scheme(&mut self, ctx: &Context) {
        self.set_scheme(ctx, ColorScheme::default(), None);
    }

    /// 更新配色方案数据与关联文件的路径，顺便：
    /// - 清空撤销、重做操作记录
    /// - 更新参考前景色/背景色
    /// - 更新窗口标题
    fn set_scheme(&mut self, ctx: &Context, scheme: ColorScheme, path: Option<PathBuf>) {
        self.scheme = scheme;
        self.scheme_file_path = path;
        self.scheme_dirty = false;
        self.history.clear();
        self.set_ref_fg_color(self.get_default_fg());
        self.set_ref_bg_color(self.get_default_bg());
        self.update_window_title(ctx);
    }

    /// 标记配色方案为干净状态
    #[inline]
    fn mark_scheme_clean(&mut self, ctx: &Context) {
        self.scheme_dirty = false;
        self.update_window_title(ctx);
    }

    /// 标记配色方案为脏状态
    #[inline]
    fn mark_scheme_dirty(&mut self, ctx: &Context) {
        self.scheme_dirty = true;
        self.update_window_title(ctx);
    }

    /// 获取指定颜色表中最近使用过的颜色
    #[inline]
    fn get_last_edited_color(&self, kind: ColorTableKind) -> OklchColor {
        match kind {
            ColorTableKind::Foreground => self.last_edited_fg_color.unwrap_or(OklchColor::BLACK),
            ColorTableKind::Background => self.last_edited_bg_color.unwrap_or(OklchColor::WHITE),
            ColorTableKind::Marker => self.last_edited_marker_color.unwrap_or(OklchColor::BLACK),
            ColorTableKind::Other => self.last_edited_other_color.unwrap_or(OklchColor::BLACK),
        }
    }

    /// 记录指定颜色表中最近使用过的颜色
    #[inline]
    fn set_last_edited_color(&mut self, kind: ColorTableKind, color: OklchColor) {
        match kind {
            ColorTableKind::Foreground => self.last_edited_fg_color = Some(color),
            ColorTableKind::Background => self.last_edited_bg_color = Some(color),
            ColorTableKind::Marker => self.last_edited_marker_color = Some(color),
            ColorTableKind::Other => self.last_edited_other_color = Some(color),
        }
    }
}

// 参考前景色/背景色相关方法
impl App {
    /// 从配色方案中获取默认的前景色
    #[inline]
    fn get_default_fg(&self) -> OklchColor {
        self.scheme
            .get_table(ColorTableKind::Foreground)
            .get_by_purpose("默认")
            .map(|entry| entry.color)
            .unwrap_or(OklchColor::BLACK)
    }

    /// 从配色方案中获取默认的背景色
    #[inline]
    fn get_default_bg(&self) -> OklchColor {
        self.scheme
            .get_table(ColorTableKind::Background)
            .get_by_purpose("默认")
            .map(|entry| entry.color)
            .unwrap_or(OklchColor::WHITE)
    }

    /// 修改参考前景色
    #[inline]
    fn set_ref_fg_color(&mut self, color: OklchColor) {
        if self.ref_color.set_fg(color) {
            self.scheme
                .get_table_mut(ColorTableKind::Background)
                .update_apca_contrast(color, false, true);
        }
    }

    /// 修改参考背景色
    #[inline]
    fn set_ref_bg_color(&mut self, color: OklchColor) {
        if self.ref_color.set_bg(color) {
            self.scheme
                .get_table_mut(ColorTableKind::Foreground)
                .update_apca_contrast(color, true, true);
        }
    }
}

// 其他方法
impl App {
    /// 更新窗口标题；在配色方案文件关联文件路径变动或配色方案的脏状态变更时使用
    fn update_window_title(&mut self, ctx: &Context) {
        let modified_mark = if self.scheme_dirty { "*" } else { "" };
        let title = if let Some(p) = &self.scheme_file_path {
            format!(
                "Greenland 配色方案编辑器 [{}{}]",
                modified_mark,
                p.display()
            )
        } else {
            format!("Greenland 配色方案编辑器 [{}]", modified_mark)
        };
        ctx.send_viewport_cmd(ViewportCommand::Title(title));
    }
}
