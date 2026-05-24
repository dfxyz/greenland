use std::{
    borrow::Cow,
    collections::VecDeque,
    time::{Duration, Instant},
};

use eframe::egui::{
    Align, Align2, Area, Color32, Context, Frame, Id, Layout, Order, Rect, Sense, Stroke, Ui, vec2,
    widget_style::WidgetState,
};

const INFO_COLOR: Color32 = Color32::from_rgb(64, 173, 96); // oklch(0.667 0.150 150)
const WARN_COLOR: Color32 = Color32::from_rgb(213, 122, 23); // oklch(0.667 0.150 60)
const ERROR_COLOR: Color32 = Color32::from_rgb(246, 82, 95); // oklch(0.667 0.200 20)
const QUEUE_CAPACITY: usize = 3;
const AUTO_DISMISS_DURATION: Duration = Duration::from_secs(3);
const AREA_MARGIN: f32 = 12.0;
const REDRAW_DELAY: Duration = Duration::from_micros(16_667); // 60Hz
const FRAME_STROKE_WIDTH: f32 = 2.0;
const FRAME_INNER_MARGIN: f32 = 12.0;
const FRAME_CORNER_RADIUS: u8 = 4;
const FRAME_WIDTH: f32 = 300.0;
const SPACE_BETWEEN_PAYLOADS: f32 = 12.0;

/// Toast提示消息管理器
#[derive(Default)]
pub struct ToastManager {
    next_id: usize,
    queue: VecDeque<Toast>,
}

enum Severity {
    Info,
    Warn,
    Error,
}

struct Toast {
    id: usize,
    severity: Severity,
    message: Cow<'static, str>,
    created_at: Option<Instant>,
}

macro_rules! define_methods {
    (
        $severity:expr,
        $doc:literal,
        $name:ident,
        $doc_auto_dismiss:literal,
        $name_auto_dismiss:ident,
    ) => {
        #[allow(unused)]
        #[doc = $doc]
        pub fn $name<S>(&mut self, message: S)
        where
            S: Into<Cow<'static, str>>,
        {
            self.push($severity, message, false);
        }
        #[allow(unused)]
        #[doc = $doc_auto_dismiss]
        pub fn $name_auto_dismiss<S>(&mut self, message: S)
        where
            S: Into<Cow<'static, str>>,
        {
            self.push($severity, message, true);
        }
    };
}

impl ToastManager {
    fn push<S>(&mut self, severity: Severity, message: S, auto_dismiss: bool)
    where
        S: Into<Cow<'static, str>>,
    {
        let toast = Toast {
            id: self.next_id,
            severity,
            message: message.into(),
            created_at: if auto_dismiss {
                Some(Instant::now())
            } else {
                None
            },
        };
        self.next_id = self.next_id.wrapping_add(1);
        self.queue.push_back(toast);
        if self.queue.len() > QUEUE_CAPACITY {
            self.queue.pop_front();
        }
    }

    define_methods!(
        Severity::Info,
        "弹出INFO级别的Toast消息",
        info,
        "弹出INFO级别的Toast消息，并自动消失",
        info_auto_dismiss,
    );
    define_methods!(
        Severity::Warn,
        "弹出WARN级别的Toast消息",
        warn,
        "弹出WARN级别的Toast消息，并自动消失",
        warn_auto_dismiss,
    );
    define_methods!(
        Severity::Error,
        "弹出ERROR级别的Toast消息",
        error,
        "弹出ERROR级别的Toast消息，并自动消失",
        error_auto_dismiss,
    );
}

impl ToastManager {
    /// 绘制Toast消息
    pub fn show(&mut self, ctx: &Context) {
        let now = Instant::now();
        self.queue.retain(|msg| {
            msg.created_at.is_none()
                || now.saturating_duration_since(msg.created_at.unwrap()) < AUTO_DISMISS_DURATION
        });
        if self.queue.is_empty() {
            return;
        };

        let mut auto_dismiss_count = 0; // 若存在自动消失的Toast消息，则需要安排重绘
        let mut to_close: Option<usize> = None; // 本帧内用户主动关闭的消息ID
        Area::new(Id::new("Toast"))
            .anchor(Align2::RIGHT_BOTTOM, vec2(-AREA_MARGIN, -AREA_MARGIN))
            .order(Order::Foreground)
            .show(ctx, |ui| {
                ui.with_layout(Layout::bottom_up(Align::LEFT), |ui| {
                    for toast in self.queue.iter() {
                        show_toast(ui, now, toast, &mut auto_dismiss_count, &mut to_close);
                        ui.add_space(SPACE_BETWEEN_PAYLOADS);
                    }
                });
            });
        if let Some(id) = to_close {
            self.queue.retain(|message| message.id != id);
        }
        if auto_dismiss_count > 0 {
            ctx.request_repaint_after(REDRAW_DELAY);
        }
    }
}

fn show_toast(
    ui: &mut Ui,
    now: Instant,
    toast: &Toast,
    auto_dismiss_count: &mut usize,
    to_close: &mut Option<usize>,
) {
    if toast.created_at.is_some() {
        *auto_dismiss_count += 1;
    }
    let color = match toast.severity {
        Severity::Info => INFO_COLOR,
        Severity::Warn => WARN_COLOR,
        Severity::Error => ERROR_COLOR,
    };
    Frame::NONE
        .stroke(Stroke::new(FRAME_STROKE_WIDTH, color))
        .fill(ui.style().visuals.panel_fill)
        .inner_margin(FRAME_INNER_MARGIN)
        .corner_radius(FRAME_CORNER_RADIUS)
        .shadow(ui.visuals().window_shadow)
        .show(ui, |ui| {
            ui.set_min_width(FRAME_WIDTH);
            ui.set_max_width(FRAME_WIDTH);
            ui.with_layout(Layout::right_to_left(Align::BOTTOM), |ui| {
                if ui.link("关闭").clicked() {
                    if toast.created_at.is_some() {
                        *auto_dismiss_count -= 1;
                    }
                    *to_close = Some(toast.id);
                }
                if let Some(created_at) = toast.created_at {
                    show_auto_dismiss_indicator(ui, now, color, created_at);
                }
            });
            ui.label(toast.message.as_ref());
        });
}

fn show_auto_dismiss_indicator(ui: &mut Ui, now: Instant, color: Color32, created_at: Instant) {
    ui.shrink_height_to_current();
    let fraction =
        1.0 - now.duration_since(created_at).as_secs_f32() / AUTO_DISMISS_DURATION.as_secs_f32();
    let full_rect = ui
        .allocate_exact_size(
            vec2(ui.available_width(), ui.available_height()),
            Sense::empty(),
        )
        .0;
    let rect = full_rect.shrink2(vec2(0.0, 4.0));
    ui.painter().rect_filled(
        rect,
        0,
        ui.style()
            .widget_style(&Default::default(), WidgetState::Inactive)
            .frame
            .fill,
    );
    ui.painter().rect_filled(
        Rect::from_min_size(rect.min, vec2(rect.width() * fraction, rect.height())),
        0,
        color,
    );
}
