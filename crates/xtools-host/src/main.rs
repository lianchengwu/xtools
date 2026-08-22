mod anim;
mod input;
mod layout;
mod overlay;
mod paint;

use std::cell::RefCell;
use std::path::PathBuf;
use std::process::Command;
use std::rc::Rc;

use gtk4::gdk::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, DrawingArea, GestureDrag};

use xtools_ui::{
    claim_instance, func_radius, main_radius, raise_instance, ToolId, HOST_INSTANCE, SLOP,
    TIME_INSTANCE,
};

use crate::layout::{
    clamp_main, default_main_center, fan_seats, hit_disk, surface_rect, vis_scale, Rect,
};

#[derive(Clone, Copy, Debug)]
enum Menu {
    Collapsed,
    Expanding { start_us: i64 },
    Expanded,
    Collapsing { start_us: i64 },
}

impl Menu {
    fn amount(self, now_us: i64) -> f64 {
        match self {
            Menu::Collapsed => 0.0,
            Menu::Expanded => 1.0,
            Menu::Expanding { start_us } => anim::ease_out_cubic(anim::progress(now_us, start_us)),
            Menu::Collapsing { start_us } => {
                1.0 - anim::ease_out_cubic(anim::progress(now_us, start_us))
            }
        }
    }

    fn is_openish(self) -> bool {
        !matches!(self, Menu::Collapsed)
    }
}

struct Host {
    main: (f64, f64),
    origin_main: (f64, f64),
    monitor: Rect,
    menu: Menu,
    dragging: bool,
    last_pointer_event: Option<gtk4::gdk::Event>,
    ticking: bool,
    last_t: f64,
    seated: bool,
    _instance: std::os::unix::net::UnixListener,
}

impl Host {
    fn vis(&self) -> f64 {
        vis_scale(self.monitor)
    }

    fn main_r(&self) -> f64 {
        main_radius() * self.vis()
    }

    fn func_r(&self) -> f64 {
        func_radius() * self.vis()
    }

    fn slop(&self) -> f64 {
        SLOP * self.vis()
    }

    fn seats(&self) -> [(ToolId, f64, f64); 3] {
        fan_seats(self.main, self.monitor, self.vis())
    }

    fn func_at(&self, px: f64, py: f64) -> Option<ToolId> {
        if matches!(self.menu, Menu::Collapsed) {
            return None;
        }
        let fr = self.func_r();
        self.seats()
            .into_iter()
            .find(|(_, x, y)| hit_disk(px, py, *x, *y, fr))
            .map(|(id, _, _)| id)
    }
}

fn load_css() {
    let provider = CssProvider::new();
    provider.load_from_string("window { background: transparent; }");
    if let Some(display) = gtk4::gdk::Display::default() {
        gtk4::style_context_add_provider_for_display(
            &display,
            &provider,
            gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
        );
    }
}

fn primary_output_size() -> (i32, i32) {
    let Some(display) = gtk4::gdk::Display::default() else {
        return (1920, 1080);
    };
    let monitors = display.monitors();
    let mut best = (1920, 1080);
    let mut area = best.0 * best.1;
    for i in 0..monitors.n_items() {
        if let Some(obj) = monitors.item(i) {
            if let Ok(mon) = obj.downcast::<gtk4::gdk::Monitor>() {
                let g = mon.geometry();
                let a = g.width() * g.height();
                if a > area {
                    best = (g.width(), g.height());
                    area = a;
                }
            }
        }
    }
    eprintln!("xtools-host: monitor size {}x{}", best.0, best.1);
    best
}

fn seat_surface(area: &DrawingArea, host: &mut Host) {
    let w = f64::from(area.width());
    let h = f64::from(area.height());
    if w < 2.0 || h < 2.0 {
        return;
    }
    let rect = surface_rect(w, h);
    host.monitor = rect;
    if !host.seated {
        let r = host.main_r();
        host.main = (w / 2.0, h - r - 12.0);
        host.origin_main = host.main;
        host.seated = true;
        eprintln!(
            "xtools-host: surface {:.0}x{:.0} vis={} main=({:.0},{:.0})",
            w,
            h,
            host.vis(),
            host.main.0,
            host.main.1
        );
    } else {
        host.main = clamp_main(host.main.0, host.main.1, host.main_r(), rect);
    }
}

fn sync_region(area: &DrawingArea, host: &Host) {
    match host.menu {
        Menu::Collapsed => {
            input::apply_collapsed_from_widget(area, host.main.0, host.main.1, host.main_r())
        }
        _ => input::apply_expanded_from_widget(area),
    }
}

fn ensure_tick(area: &DrawingArea, state: &Rc<RefCell<Host>>) {
    if state.borrow().ticking {
        return;
    }
    state.borrow_mut().ticking = true;
    let state = Rc::clone(state);
    area.add_tick_callback(move |widget, clock| {
        let now = clock.frame_time();
        let mut host = state.borrow_mut();
        host.last_t = host.menu.amount(now);
        let finished = match host.menu {
            Menu::Expanding { start_us } if anim::progress(now, start_us) >= 1.0 => {
                host.menu = Menu::Expanded;
                host.last_t = 1.0;
                true
            }
            Menu::Collapsing { start_us } if anim::progress(now, start_us) >= 1.0 => {
                host.menu = Menu::Collapsed;
                host.last_t = 0.0;
                true
            }
            Menu::Expanding { .. } | Menu::Collapsing { .. } => false,
            _ => true,
        };
        widget.queue_draw();
        if finished {
            host.ticking = false;
            if let Some(area) = widget.downcast_ref::<DrawingArea>() {
                sync_region(area, &host);
            }
            glib::ControlFlow::Break
        } else {
            glib::ControlFlow::Continue
        }
    });
}

fn begin_expand(area: &DrawingArea, state: &Rc<RefCell<Host>>) {
    let now = area.frame_clock().map(|c| c.frame_time()).unwrap_or(0);
    {
        let mut host = state.borrow_mut();
        host.menu = Menu::Expanding { start_us: now };
        host.last_t = 0.0;
    }
    input::apply_expanded_from_widget(area);
    ensure_tick(area, state);
    area.queue_draw();
}

fn begin_collapse(area: &DrawingArea, state: &Rc<RefCell<Host>>) {
    let now = area.frame_clock().map(|c| c.frame_time()).unwrap_or(0);
    {
        let mut host = state.borrow_mut();
        host.menu = Menu::Collapsing { start_us: now };
    }
    ensure_tick(area, state);
    area.queue_draw();
}

fn snap_collapse(area: &DrawingArea, state: &Rc<RefCell<Host>>) {
    let mut host = state.borrow_mut();
    host.menu = Menu::Collapsed;
    host.last_t = 0.0;
    host.ticking = false;
    sync_region(area, &host);
    area.queue_draw();
}

fn handle_click(
    area: &DrawingArea,
    state: &Rc<RefCell<Host>>,
    x: f64,
    y: f64,
    event: Option<gtk4::gdk::Event>,
) {
    let (on_main, on_func, openish) = {
        let mut host = state.borrow_mut();
        if let Some(ev) = event {
            host.last_pointer_event = Some(ev);
        }
        let on_main = hit_disk(x, y, host.main.0, host.main.1, host.main_r());
        let on_func = host.func_at(x, y);
        (on_main, on_func, host.menu.is_openish())
    };

    if let Some(id) = on_func {
        if id == ToolId::Time {
            let ev = state.borrow().last_pointer_event.clone();
            launch_time(ev.as_ref());
        }
        begin_collapse(area, state);
        return;
    }
    if on_main {
        if openish {
            begin_collapse(area, state);
        } else {
            begin_expand(area, state);
        }
        return;
    }
    if openish {
        begin_collapse(area, state);
    }
}

fn build_ui(app: &Application, instance: std::os::unix::net::UnixListener) {
    if !gtk4_layer_shell::is_supported() {
        eprintln!("xtools-host: layer-shell is not supported on this compositor");
        app.quit();
        return;
    }

    load_css();

    let window = ApplicationWindow::builder()
        .application(app)
        .decorated(false)
        .resizable(false)
        .default_width(256)
        .default_height(256)
        .build();

    overlay::attach_overlay(&window);
    window.set_default_size(256, 256);

    let area = DrawingArea::new();
    area.set_hexpand(true);
    area.set_vexpand(true);
    window.set_child(Some(&area));

    let state = Rc::new(RefCell::new(Host {
        main: (0.0, 0.0),
        origin_main: (0.0, 0.0),
        monitor: Rect {
            x: 0.0,
            y: 0.0,
            w: 1920.0,
            h: 1080.0,
        },
        menu: Menu::Collapsed,
        dragging: false,
        last_pointer_event: None,
        ticking: false,
        last_t: 0.0,
        seated: false,
        _instance: instance,
    }));

    {
        let state = Rc::clone(&state);
        area.set_draw_func(move |_, cr, _w, _h| {
            let host = state.borrow();
            cr.set_operator(gtk4::cairo::Operator::Source);
            cr.set_source_rgba(0.0, 0.0, 0.0, 0.0);
            cr.paint().ok();
            cr.set_operator(gtk4::cairo::Operator::Over);
            let t = host.last_t;
            let vis = host.vis();
            if t > 0.0 {
                for (id, x, y) in host.seats() {
                    let px = anim::lerp(host.main.0, x, t);
                    let py = anim::lerp(host.main.1, y, t);
                    paint::draw_func(cr, id, px, py, t, vis);
                }
            }
            paint::draw_main(cr, host.main.0, host.main.1, vis);
        });
    }

    {
        let state = Rc::clone(&state);
        area.connect_realize(move |area| {
            if !input::refuse_if_no_input_shapes(&area.display()) {
                if let Some(win) = area.root().and_downcast::<ApplicationWindow>() {
                    if let Some(app) = win.application() {
                        app.quit();
                    }
                }
                return;
            }
            {
                let mut host = state.borrow_mut();
                seat_surface(area, &mut host);
            }
            let host = state.borrow();
            if host.seated {
                sync_region(area, &host);
                if let Some(win) = area.root().and_downcast::<ApplicationWindow>() {
                    let (_sw, sh) = primary_output_size();
                    overlay::place_mid_right(&win, sh, host.main.1);
                }
            }
            area.queue_draw();
        });
    }

    {
        let state = Rc::clone(&state);
        area.connect_resize(move |area, _w, _h| {
            {
                let mut host = state.borrow_mut();
                seat_surface(area, &mut host);
            }
            let host = state.borrow();
            if host.seated {
                sync_region(area, &host);
                if let Some(win) = area.root().and_downcast::<ApplicationWindow>() {
                    let (_sw, sh) = primary_output_size();
                    overlay::place_mid_right(&win, sh, host.main.1);
                }
            }
            area.queue_draw();
        });
    }

    let drag = GestureDrag::new();
    {
        let state = Rc::clone(&state);
        drag.connect_drag_begin(move |g, _x, _y| {
            let mut host = state.borrow_mut();
            host.last_pointer_event = g.last_event(None);
            host.origin_main = host.main;
            host.dragging = false;
        });
    }
    {
        let state = Rc::clone(&state);
        let area = area.clone();
        drag.connect_drag_update(move |_, dx, dy| {
            let slop = state.borrow().slop();
            let dist = (dx * dx + dy * dy).sqrt();
            let should_snap = {
                let host = state.borrow();
                !host.dragging && dist > slop && host.menu.is_openish()
            };
            if !state.borrow().dragging && dist > slop {
                if should_snap {
                    snap_collapse(&area, &state);
                }
                state.borrow_mut().dragging = true;
            }
            if state.borrow().dragging {
                let mut host = state.borrow_mut();
                let (cx, cy) = (host.origin_main.0 + dx, host.origin_main.1 + dy);
                let r = host.main_r();
                let mon = host.monitor;
                host.main = clamp_main(cx, cy, r, mon);
            }
            area.queue_draw();
        });
    }
    {
        let state = Rc::clone(&state);
        let area = area.clone();
        drag.connect_drag_end(move |g, dx, dy| {
            let start = g.start_point();
            let dragged = state.borrow().dragging;
            if dragged {
                let host = state.borrow();
                sync_region(&area, &host);
                area.queue_draw();
                return;
            }
            let Some((sx, sy)) = start else {
                return;
            };
            handle_click(&area, &state, sx + dx, sy + dy, g.last_event(None));
        });
    }
    area.add_controller(drag);

    window.present();
}
fn mint_token(event: &gtk4::gdk::Event) -> Option<String> {
    let display = event.display()?;
    let ctx = display.app_launch_context();
    ctx.set_timestamp(event.time());
    let id = gtk4::gio::prelude::AppLaunchContextExt::startup_notify_id(
        &ctx,
        None::<&gtk4::gio::AppInfo>,
        &[],
    )?;
    let s = id.to_string();
    if s.is_empty() {
        None
    } else {
        Some(s)
    }
}

fn sibling_bin(name: &str) -> Option<PathBuf> {
    let exe = std::env::current_exe().ok()?;
    let path = exe.parent()?.join(name);
    path.is_file().then_some(path)
}

fn launch_time(event: Option<&gtk4::gdk::Event>) {
    let token = event.and_then(mint_token);
    match raise_instance(TIME_INSTANCE, token.as_deref()) {
        Ok(true) => return,
        Ok(false) => {}
        Err(_) => {}
    }
    let Some(bin) = sibling_bin(ToolId::Time.binary_name()) else {
        eprintln!("xtools-host: missing sibling {}", ToolId::Time.binary_name());
        return;
    };
    let mut cmd = Command::new(bin);
    if let Some(token) = token {
        cmd.env("XDG_ACTIVATION_TOKEN", token);
    }
    if let Err(err) = cmd.spawn() {
        eprintln!("xtools-host: spawn xtools-time: {err}");
    }
}

fn main() {
    let instance = match claim_instance(HOST_INSTANCE) {
        Ok(Some(listener)) => listener,
        Ok(None) => return,
        Err(err) => {
            eprintln!("xtools-host: instance lock failed: {err}");
            std::process::exit(1);
        }
    };

    let app = Application::builder()
        .application_id("dev.xtools.host")
        .build();

    let held = Rc::new(RefCell::new(Some(instance)));
    app.connect_activate(move |app| {
        let Some(listener) = held.borrow_mut().take() else {
            return;
        };
        build_ui(app, listener);
    });

    app.run();
}
