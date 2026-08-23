use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::Sender;

use gtk4::gdk_pixbuf::Pixbuf;
use ksni::blocking::TrayMethods;
use ksni::menu::{MenuItem, StandardItem};
use ksni::{Icon, ToolTip, Tray};

const XTOOLS_SVG: &[u8] = include_bytes!("../../../xtools.svg");

#[derive(Clone, Copy, Debug)]
pub enum TrayAction {
    Show,
    Hide,
    Toggle,
    Quit,
}

pub struct XToolsTray {
    sender: Sender<TrayAction>,
    visible: Arc<AtomicBool>,
    icons: Vec<Icon>,
    icon_theme_path: String,
}

impl Tray for XToolsTray {
    fn id(&self) -> String {
        "dev.xtools.host".into()
    }

    fn title(&self) -> String {
        "xtools".into()
    }

    fn tool_tip(&self) -> ToolTip {
        ToolTip {
            title: "xtools".into(),
            description: "Linux 桌面悬浮工具箱".into(),
            icon_name: "xtools".into(),
            icon_pixmap: self.icons.clone(),
        }
    }

    fn icon_theme_path(&self) -> String {
        self.icon_theme_path.clone()
    }

    fn icon_name(&self) -> String {
        if self.icon_theme_path.is_empty() {
            "applications-utilities".into()
        } else {
            "xtools".into()
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        self.icons.clone()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        let _ = self.sender.send(TrayAction::Toggle);
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let _visible = self.visible.load(Ordering::Relaxed);
        vec![
            StandardItem {
                label: "显示".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.sender.send(TrayAction::Show);
                }),
                ..Default::default()
            }
            .into(),
            StandardItem {
                label: "隐藏".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.sender.send(TrayAction::Hide);
                }),
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "退出".into(),
                activate: Box::new(|this: &mut Self| {
                    let _ = this.sender.send(TrayAction::Quit);
                }),
                ..Default::default()
            }
            .into(),
        ]
    }
}

pub fn render_icons() -> Vec<Icon> {
    let sizes = [22, 24, 32, 48, 64];
    let mut icons = Vec::with_capacity(sizes.len());
    for size in sizes {
        if let Some(icon) = render_icon(size) {
            icons.push(icon);
        }
    }
    icons
}

fn load_base_pixbuf() -> Option<Pixbuf> {
    Pixbuf::from_read(std::io::Cursor::new(XTOOLS_SVG)).ok()
}

fn render_icon(size: i32) -> Option<Icon> {
    let base = load_base_pixbuf()?;
    let pb = base.scale_simple(size, size, gtk4::gdk_pixbuf::InterpType::Bilinear)?;
    pixbuf_to_sni_icon(&pb)
}

fn pixbuf_to_sni_icon(pb: &Pixbuf) -> Option<Icon> {
    let width = pb.width();
    let height = pb.height();
    let n_channels = pb.n_channels();
    if width <= 0 || height <= 0 || n_channels < 3 {
        return None;
    }
    let rowstride = pb.rowstride() as usize;
    let bytes = pb.read_pixel_bytes();
    let slice = bytes.as_ref();

    let mut data = Vec::with_capacity((width * height * 4) as usize);
    for y in 0..height as usize {
        let row_start = y * rowstride;
        for x in 0..width as usize {
            let px = row_start + x * n_channels as usize;
            if px + n_channels as usize > slice.len() {
                return None;
            }
            let r = slice[px];
            let g = slice[px + 1];
            let b = slice[px + 2];
            let a = if n_channels >= 4 { slice[px + 3] } else { 255 };
            let (r, g, b) = if r < 50 && g < 50 && b < 50 {
                (240, 240, 245)
            } else {
                (r, g, b)
            };
            data.extend_from_slice(&[a, r, g, b]);
        }
    }

    Some(Icon {
        width,
        height,
        data,
    })
}

fn write_theme_icon() -> Option<String> {
    let base = load_base_pixbuf()?;
    let pb = base.scale_simple(32, 32, gtk4::gdk_pixbuf::InterpType::Bilinear)?;
    let dir = runtime_icon_dir()?;
    let apps = dir.join("hicolor").join("32x32").join("apps");
    std::fs::create_dir_all(&apps).ok()?;
    let path = apps.join("xtools.png");
    pb.savev(&path, "png", &[]).ok()?;
    Some(dir.to_string_lossy().into_owned())
}

fn runtime_icon_dir() -> Option<std::path::PathBuf> {
    let root = std::env::var_os("XDG_RUNTIME_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    Some(root.join("xtools-icons"))
}
pub fn spawn_tray(sender: Sender<TrayAction>, visible: Arc<AtomicBool>) {
    let icons = render_icons();
    if icons.is_empty() {
        eprintln!("xtools-host: tray icon pixmap is empty, falling back to theme icon");
    }
    let icon_theme_path = write_theme_icon().unwrap_or_default();

    std::thread::Builder::new()
        .name("xtools-tray".into())
        .spawn(move || {
            let tray = XToolsTray {
                sender,
                visible,
                icons,
                icon_theme_path,
            };
            match tray.assume_sni_available(true).spawn() {
                Ok(handle) => {
                    eprintln!("xtools-host: status notifier registered");
                    // Keep the SNI connection alive for the process lifetime.
                    std::mem::forget(handle);
                }
                Err(err) => eprintln!("xtools-host: tray spawn failed: {err}"),
            }
        })
        .expect("xtools-host: failed to start tray thread");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_icons() {
        gtk4::init().ok();
        let icons = render_icons();
        assert!(!icons.is_empty(), "Should render at least one icon");
        for icon in &icons {
            assert!(icon.width > 0);
            assert!(icon.height > 0);
            assert_eq!(
                icon.data.len(),
                (icon.width * icon.height * 4) as usize,
                "Icon data length should match ARGB32 size"
            );
        }
    }

    #[test]
    fn test_tray_menu_and_actions() {
        let (tx, rx) = std::sync::mpsc::channel();
        let visible = Arc::new(AtomicBool::new(true));
        let mut tray = XToolsTray {
            sender: tx,
            visible,
            icons: Vec::new(),
            icon_theme_path: String::new(),
        };

        assert_eq!(tray.id(), "dev.xtools.host");
        assert_eq!(tray.title(), "xtools");

        tray.activate(0, 0);
        assert!(matches!(rx.try_recv(), Ok(TrayAction::Toggle)));

        let menu = tray.menu();
        assert_eq!(menu.len(), 4);
    }
}
