//! Shared tokens, instance lock, and optional egui chrome.

pub mod ids;
pub mod instance;
pub mod theme;

#[cfg(feature = "egui-chrome")]
pub mod chrome;

pub use ids::{HOST_INSTANCE, TIME_INSTANCE, ToolId};
pub use instance::{accept_raise, claim_instance, raise_instance};
pub use theme::{
    Color, FUNC_D, GAP, MAIN_D, MARK_PX, ORB_FILL, ORB_MARK, POP_MS, SLOP, func_radius,
    main_radius, orbit_radius,
};
