/// Compile-time menu entries. No plugin directory.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum ToolId {
    Time,
    Json,
    Trans,
}

impl ToolId {
    /// Left-to-right fan order: Time, Json, Trans.
    pub const ALL: [ToolId; 3] = [ToolId::Time, ToolId::Json, ToolId::Trans];

    /// On-disk mark. Clock is drawn as a glyph in the host; others are pango text.
    pub fn mark(self) -> &'static str {
        match self {
            ToolId::Time => "clock",
            ToolId::Json => "{}",
            ToolId::Trans => "文",
        }
    }
}

/// Abstract single-instance name for the host process (`\0` + this).
pub const HOST_INSTANCE: &str = "xtools-host";
