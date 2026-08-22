//! Shared tokens, instance lock, and optional egui chrome.

pub mod ids;
pub mod instance;
pub mod theme;

#[cfg(feature = "egui-chrome")]
pub mod chrome;

pub use ids::{ToolId, HOST_INSTANCE, TIME_INSTANCE};
pub use instance::{accept_raise, claim_instance, raise_instance};
pub use theme::{
    func_radius, main_radius, orbit_radius, Color, FUNC_D, GAP, MAIN_D, MARK_PX, ORB_FILL, ORB_MARK,
    POP_MS, SLOP,
};
