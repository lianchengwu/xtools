mod anim;
mod layout;

#[cfg(unix)]
mod input;
#[cfg(unix)]
mod overlay;
#[cfg(unix)]
mod paint;
#[cfg(unix)]
mod tray;
#[cfg(unix)]
mod unix;

#[cfg(any(windows, test))]
pub mod windows;

fn main() {
    #[cfg(unix)]
    unix::run();

    #[cfg(windows)]
    windows::run();
}
