use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};

/// Attach as a compositor Overlay. Must run before `present()`.
pub fn attach_overlay(window: &gtk4::ApplicationWindow) {
    window.init_layer_shell();
    window.set_layer(Layer::Overlay);
    window.set_namespace(Some("xtools"));
    window.set_keyboard_mode(KeyboardMode::None);
    for edge in [Edge::Top, Edge::Bottom, Edge::Left, Edge::Right] {
        window.set_anchor(edge, true);
    }
    window.set_exclusive_zone(-1);
}
