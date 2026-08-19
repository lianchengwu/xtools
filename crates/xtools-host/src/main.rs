mod anim;
mod input;
mod layout;
mod overlay;
mod paint;

use std::cell::RefCell;
use std::rc::Rc;

use gtk4::gdk::prelude::*;
use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Application, ApplicationWindow, CssProvider, DrawingArea, GestureDrag};

use xtools_ui::{claim_instance, func_radius, main_radius, ToolId, HOST_INSTANCE, SLOP};

use crate::layout::{clamp_main, default_main_center, fan_seats, hit_disk, Rect};

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
    _instance: std::os::unix::net::UnixListener,
}

impl Host {
    fn seats(&self) -> [(ToolId, f64, f64); 3] {
        fan_seats(self.main, self.monitor)
    }

    fn func_at(&self, px: f64, py: f64) -> Option<ToolId> {
        if matches!(self.menu, Menu::Collapsed) {
            return None;
        }
        self.seats()
            .into_iter()
            .find(|(_, x, y)| hit_disk(px, py, *x, *y, func_radius()))
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

fn monitor_rect(widget: &impl IsA<gtk4::Widget>) -> Rect {
    if let Some(native) = widget.native() {
        if let Some(surface) = native.surface() {
            if let Some(monitor) = widget.display().monitor_at_surface(&surface) {
                return Rect::from_monitor(&monitor.geometry());
            }
            if let Some(obj) = widget.display().monitors().item(0) {
                if let Ok(monitor) = obj.downcast::<gtk4::gdk::Monitor>() {
                    return Rect::from_monitor(&monitor.geometry());
                }
            }
        }
    }
    Rect {
        x: 0.0,
        y: 0.0,
        w: 1920.0,
        h: 1080.0,
    }
}

fn sync_region(area: &DrawingArea, host: &Host) {
    match host.menu {
        Menu::Collapsed => input::apply_collapsed_from_widget(area, host.main.0, host.main.1),
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

fn handle_click(area: &DrawingArea, state: &Rc<RefCell<Host>>, x: f64, y: f64, event: Option<gtk4::gdk::Event>) {
    let (on_main, on_func, openish) = {
        let mut host = state.borrow_mut();
        if let Some(ev) = event {
            host.last_pointer_event = Some(ev);
        }
        let on_main = hit_disk(x, y, host.main.0, host.main.1, main_radius());
        let on_func = host.func_at(x, y);
        (on_main, on_func, host.menu.is_openish())
    };

    if on_func.is_some() {
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
        .build();

    overlay::attach_overlay(&window);

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
            if t > 0.0 {
                for (id, x, y) in host.seats() {
                    let px = anim::lerp(host.main.0, x, t);
                    let py = anim::lerp(host.main.1, y, t);
                    paint::draw_func(cr, id, px, py, t);
                }
            }
            paint::draw_main(cr, host.main.0, host.main.1);
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
            let mon = monitor_rect(area);
            {
                let mut host = state.borrow_mut();
                host.monitor = mon;
                host.main = default_main_center(mon, main_radius());
                host.origin_main = host.main;
            }
            let host = state.borrow();
            input::apply_collapsed_from_widget(area, host.main.0, host.main.1);
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
            let dist = (dx * dx + dy * dy).sqrt();
            let should_snap = {
                let host = state.borrow();
                !host.dragging && dist > SLOP && host.menu.is_openish()
            };
            if !state.borrow().dragging && dist > SLOP {
                if should_snap {
                    snap_collapse(&area, &state);
                }
                state.borrow_mut().dragging = true;
            }
            if state.borrow().dragging {
                let mut host = state.borrow_mut();
                let (cx, cy) = (host.origin_main.0 + dx, host.origin_main.1 + dy);
                host.main = clamp_main(cx, cy, main_radius(), host.monitor);
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
                input::apply_collapsed_from_widget(&area, host.main.0, host.main.1);
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
