//! Shared tokens and compile-time tool ids. No GUI toolkit.

pub mod ids;
pub mod instance;
pub mod theme;

pub use ids::{ToolId, HOST_INSTANCE};
pub use instance::claim_instance;
pub use theme::{
    func_radius, main_radius, orbit_radius, Color, FUNC_D, GAP, MAIN_D, MARK_PX, ORB_FILL, ORB_MARK,
    POP_MS, SLOP,
};
