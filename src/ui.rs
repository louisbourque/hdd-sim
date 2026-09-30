use std::borrow::Cow;
use std::rc::Rc;
use std::sync::Arc;

use gpui_kit::component::button::{Button, ButtonVariants};
use gpui_kit::component::slider::{Slider, SliderEvent, SliderState};
use gpui_kit::component::switch::Switch;
use gpui_kit::component::{Disableable, Theme, ThemeMode, h_flex, v_flex};
use gpui_kit::prelude::FluentBuilder as _;
use gpui_kit::*;

use crate::config::{Config, DriveConfig, load_config, save_config};
use crate::monitor::Monitor;
use crate::{Drive, scan_hard_drives};

// RusticOtter brand tokens, from rusticotter.ca/src/styles/global.css
const OTTER: u32 = 0x544839;
const INK: u32 = 0x473c2d;
const MUTED: u32 = 0x584c3d;
const LAKE: u32 = 0x374756;
const MOSS: u32 = 0x424e3f;
const PARCHMENT: u32 = 0xf6f1e9;
const LINEN: u32 = 0xebe3d6;
const SANS: &str = "Inter";
const DISPLAY: &str = "Zilla Slab";

fn c(hex: u32) -> Hsla {
    rgb(hex).into()
}

/// Registers the brand fonts and points the component theme at the brand colours.
pub fn init(cx: &mut App) {
    cx.text_system()
        .add_fonts(vec![
            Cow::Borrowed(include_bytes!("../assets/fonts/Inter-Regular.ttf").as_slice()),
            Cow::Borrowed(include_bytes!("../assets/fonts/Inter-Medium.ttf").as_slice()),
            Cow::Borrowed(include_bytes!("../assets/fonts/Inter-SemiBold.ttf").as_slice()),
            Cow::Borrowed(include_bytes!("../assets/fonts/ZillaSlab-SemiBold.ttf").as_slice()),
            Cow::Borrowed(include_bytes!("../assets/fonts/ZillaSlab-Bold.ttf").as_slice()),
        ])
        .expect("bundled fonts load");

    Theme::change(ThemeMode::Light, None, cx);
    Theme::update(cx, |t| {
        t.font_family = SANS.into();
        t.foreground = c(INK);
        t.muted_foreground = c(MUTED);
        t.background = white();
        t.border = c(LINEN);
        t.ring = c(LAKE);
        t.primary = c(OTTER);
        t.primary_hover = c(OTTER).opacity(0.9);
        t.primary_active = c(INK);
        t.primary_foreground = white();
        t.switch = c(MUTED).opacity(0.3);
        t.slider_bar = c(OTTER);
        t.slider_thumb = c(OTTER);
    });
}

pub struct HddApp {
    config: Config,
    drives: Vec<Drive>,
    selected: Option<usize>,
    monitor: Rc<Monitor>,
    volume: Entity<SliderState>,
    interval: Entity<SliderState>,
    otter: Arc<Image>,
    _subscriptions: Vec<Subscription>,
}

impl HddApp {
    pub fn new(monitor: Rc<Monitor>, _window: &mut Window, cx: &mut Context<Self>) -> Self {
        let config = load_config();
        let volume = cx.new(|_| SliderState::new().min(0.).max(100.).step(1.));
        let interval = cx.new(|_| {
            SliderState::new()
                .min(50.)
                .max(1000.)
                .step(10.)
                .default_value(config.poll_interval_ms as f32)
        });
        let _subscriptions = vec![
            cx.subscribe(&volume, |this, _, ev: &SliderEvent, cx| {
                if let SliderEvent::Change(v) = ev {
                    this.update_drive(cx, |d| d.volume = v.end() as u8);
                }
            }),
            cx.subscribe(&interval, |this, _, ev: &SliderEvent, cx| {
                if let SliderEvent::Change(v) = ev {
                    this.config.poll_interval_ms = (v.end() as u64).clamp(50, 1000);
                    this.persist(cx);
                }
            }),
        ];
        Self {
            config,
            drives: scan_hard_drives(),
            selected: None,
            monitor,
            volume,
            interval,
            otter: Arc::new(Image::from_bytes(
                ImageFormat::Png,
                include_bytes!("../assets/otter_small.png").to_vec(),
            )),
            _subscriptions,
        }
    }

    fn drive_config(&self, ix: usize) -> DriveConfig {
        self.config
            .drives
            .get(&self.drives[ix].name)
            .cloned()
            .unwrap_or_default()
    }

    fn select(&mut self, ix: Option<usize>, window: &mut Window, cx: &mut Context<Self>) {
        self.selected = ix;
        if let Some(ix) = ix {
            let volume = self.drive_config(ix).volume as f32;
            self.volume
                .update(cx, |s, cx| s.set_value(volume, window, cx));
        }
        cx.notify();
    }

    fn update_drive(&mut self, cx: &mut Context<Self>, f: impl FnOnce(&mut DriveConfig)) {
        let Some(ix) = self.selected else { return };
        f(self
            .config
            .drives
            .entry(self.drives[ix].name.clone())
            .or_default());
        self.persist(cx);
    }

    fn persist(&mut self, cx: &mut Context<Self>) {
        save_config(&self.config);
        self.monitor.update_config(&self.config);
        cx.notify();
    }

    fn render_drive_list(&self, cx: &mut Context<Self>) -> impl IntoElement {
        card()
            .id("drive-list")
            .role(Role::ListBox)
            .aria_label("Hard drives")
            .w(px(280.))
            .flex_none()
            .p_2()
            .overflow_y_scroll()
            .on_click(cx.listener(|this, _, window, cx| this.select(None, window, cx)))
            .children(self.drives.iter().enumerate().map(|(ix, drive)| {
                let selected = self.selected == Some(ix);
                div()
                    .id(("drive", ix))
                    .role(Role::ListBoxOption)
                    .aria_selected(selected)
                    .aria_label(format!("{} disk, {}", drive.size_display, drive.model))
                    .tab_index(0)
                    .p_4()
                    .mb_2()
                    .rounded(px(12.))
                    .cursor_pointer()
                    .when(selected, |s| {
                        s.bg(c(LAKE).opacity(0.1))
                            .border_l_4()
                            .border_color(c(LAKE))
                    })
                    .when(!selected, |s| s.hover(|s| s.bg(c(LINEN))))
                    .focus_visible(|s| s.border_2().border_color(c(LAKE)))
                    .on_click(cx.listener(move |this, _, window, cx| {
                        cx.stop_propagation();
                        this.select(Some(ix), window, cx);
                    }))
                    .on_key_down(cx.listener(move |this, ev: &KeyDownEvent, window, cx| {
                        match ev.keystroke.key.as_str() {
                            "enter" | "space" => this.select(Some(ix), window, cx),
                            "down" => window.focus_next(cx),
                            "up" => window.focus_prev(cx),
                            _ => return,
                        }
                        cx.stop_propagation();
                    }))
                    .child(
                        div()
                            .font_family(DISPLAY)
                            .font_weight(FontWeight::SEMIBOLD)
                            .mb_1()
                            .child(format!("{} disk", drive.size_display)),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(c(MUTED))
                            .child(drive.model.clone()),
                    )
            }))
            .when(self.drives.is_empty(), |s| {
                s.child(
                    div()
                        .p_4()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("No hard drives found"),
                )
            })
    }

    fn render_drive_settings(&self, ix: usize, cx: &mut Context<Self>) -> AnyElement {
        let model = self.drives[ix].model.clone();
        let dc = self.drive_config(ix);
        let on = dc.enabled;

        v_flex()
            .gap_6()
            .child(heading("drive-heading", 2, format!("{model} settings")))
            .child(
                v_flex()
                    .id("drive-settings")
                    .role(Role::Group)
                    .aria_label(format!("{model} settings"))
                    .max_w(px(600.))
                    .p_2()
                    .rounded(px(16.))
                    .bg(c(PARCHMENT))
                    .border_1()
                    .border_color(c(LINEN))
                    .child(setting_row(
                        "Enabled",
                        switch("enabled")
                            .checked(on)
                            .accessibility_label(format!("Enable {model}"))
                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                this.update_drive(cx, |d| d.enabled = *v)
                            })),
                    ))
                    .child(setting_row(
                        "Read",
                        switch("read")
                            .checked(dc.read)
                            .disabled(!on)
                            .accessibility_label(format!("Enable read operations for {model}"))
                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                this.update_drive(cx, |d| d.read = *v)
                            })),
                    ))
                    .child(setting_row(
                        "Write",
                        switch("write")
                            .checked(dc.write)
                            .disabled(!on)
                            .accessibility_label(format!("Enable write operations for {model}"))
                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                this.update_drive(cx, |d| d.write = *v)
                            })),
                    ))
                    .child(setting_row(
                        "Volume",
                        slider_with_value(
                            "volume",
                            format!("Volume level for {model}"),
                            Slider::new(&self.volume).disabled(!on),
                            dc.volume.to_string(),
                        ),
                    )),
            )
            .into_any_element()
    }

    fn render_global(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let active = self.config.active;
        card()
            .id("global-settings")
            .role(Role::Group)
            .aria_label("Global settings")
            .flex()
            .items_center()
            .gap_6()
            .py_5()
            .px_8()
            .child(
                Button::new("active")
                    .primary()
                    .rounded(px(24.))
                    .px_6()
                    .label(if active { "Pause monitoring" } else { "Start monitoring" })
                    .on_click(cx.listener(|this, _, _, cx| {
                        this.config.active = !this.config.active;
                        this.persist(cx);
                    })),
            )
            .child(
                v_flex()
                    .flex_1()
                    .max_w(px(512.))
                    .child(setting_row(
                        "Interval",
                        slider_with_value(
                            "interval",
                            "Poll interval in milliseconds",
                            Slider::new(&self.interval),
                            format!("{} ms", self.config.poll_interval_ms),
                        ),
                    ))
                    .child(setting_row(
                        "Keep running in tray when closed",
                        switch("close-to-tray")
                            .checked(self.config.close_to_tray)
                            .accessibility_label("Keep running in tray when the window is closed")
                            .on_click(cx.listener(|this, v: &bool, _, cx| {
                                this.config.close_to_tray = *v;
                                this.persist(cx);
                            })),
                    )),
            )
    }
}

impl Render for HddApp {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let main = match self.selected {
            Some(ix) => self.render_drive_settings(ix, cx),
            None => empty_state().into_any_element(),
        };

        v_flex()
            .size_full()
            .p_4()
            .gap_4()
            .bg(linear_gradient(
                135.,
                linear_color_stop(c(PARCHMENT), 0.),
                linear_color_stop(c(LINEN), 1.),
            ))
            .font_family(SANS)
            .text_color(c(INK))
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    // Decorative mascot next to the name
                    .child(img(self.otter.clone()).h(px(40.)))
                    .child(
                        div()
                            .id("app-title")
                            .role(Role::Heading)
                            .aria_level(1)
                            .font_family(DISPLAY)
                            .font_weight(FontWeight::BOLD)
                            .text_2xl()
                            .text_color(c(OTTER))
                            .child("HDD Simulator"),
                    ),
            )
            .child(
                h_flex()
                    .flex_1()
                    .min_h_0()
                    .items_stretch()
                    .gap_4()
                    .child(self.render_drive_list(cx))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_4()
                            .child(
                                card()
                                    .id("main-content")
                                    .flex_1()
                                    .overflow_y_scroll()
                                    .p_8()
                                    .child(main),
                            )
                            .child(self.render_global(cx)),
                    ),
            )
    }
}

/// White card with linen border, per the brand `card` utility.
fn card() -> Div {
    div()
        .bg(white())
        .border_1()
        .border_color(c(LINEN))
        .rounded(px(16.))
        .shadow_lg()
}

fn heading(id: &'static str, level: usize, text: impl Into<SharedString>) -> impl IntoElement {
    div()
        .id(id)
        .role(Role::Heading)
        .aria_level(level)
        .font_family(DISPLAY)
        .font_weight(FontWeight::SEMIBOLD)
        .text_2xl()
        .child(text.into())
}

fn empty_state() -> impl IntoElement {
    v_flex()
        .size_full()
        .items_center()
        .justify_center()
        .gap_2()
        .child(heading("empty-heading", 2, "No device selected"))
        .child(
            div()
                .text_sm()
                .text_color(c(MUTED))
                .child("Select a drive to manage."),
        )
}

fn setting_row(label: &'static str, control: impl IntoElement) -> impl IntoElement {
    h_flex()
        .flex_1()
        .justify_between()
        .items_center()
        .gap_4()
        .p_4()
        .child(
            div()
                .min_w(px(80.))
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .child(label),
        )
        .child(control)
}

fn slider_with_value(
    id: &'static str,
    aria_label: impl Into<SharedString>,
    slider: Slider,
    value: String,
) -> impl IntoElement {
    h_flex()
        .id(id)
        .role(Role::Group)
        .aria_label(aria_label)
        .flex_1()
        .max_w(px(300.))
        .gap_3()
        .child(slider.flex_1())
        .child(
            div()
                .min_w(px(50.))
                .text_right()
                .text_sm()
                .font_weight(FontWeight::MEDIUM)
                .text_color(c(MUTED))
                .child(value),
        )
}

/// Switches read "on" in moss, the brand's success colour.
fn switch(id: &'static str) -> Switch {
    Switch::new(id).color(c(MOSS))
}
