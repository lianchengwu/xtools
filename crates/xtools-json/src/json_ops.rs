use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JsonIssue {
    pub line: usize,
    pub column: usize,
    pub message: String,
}

impl JsonIssue {
    pub fn display(&self) -> String {
        format!("第 {} 行第 {} 列：{}", self.line, self.column, self.message)
    }
}

pub fn empty_input(text: &str) -> bool {
    text.trim().is_empty()
}

pub fn format_json(input: &str) -> Result<String, JsonIssue> {
    let value = parse(input)?;
    serde_json::to_string_pretty(&value).map_err(from_error)
}

pub fn minify_json(input: &str) -> Result<String, JsonIssue> {
    let value = parse(input)?;
    serde_json::to_string(&value).map_err(from_error)
}

pub fn validate_json(input: &str) -> Result<(), JsonIssue> {
    parse(input).map(|_| ())
}

pub fn parse(input: &str) -> Result<Value, JsonIssue> {
    serde_json::from_str(input).map_err(from_error)
}

/// Unescape stringified / escaped JSON text.
///
/// Handles:
/// 1. Double-quoted JSON strings: `"{\"a\": 1}"` -> `{"a": 1}`
/// 2. Escaped JSON without outer quotes: `{\"a\": 1}` -> `{"a": 1}`
/// 3. Backslash escapes: `\"`, `\\`, `\n`, `\t`, `\/`, `\r`, `\uXXXX`
///
/// If the unescaped result is valid JSON, formats it nicely; otherwise returns the unescaped text.
pub fn unescape_json(input: &str) -> Result<String, JsonIssue> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Ok(String::new());
    }

    // Attempt 1: If input is a JSON string literal (starts and ends with quotes)
    if trimmed.starts_with('"') && trimmed.ends_with('"') && trimmed.len() >= 2 {
        if let Ok(parsed_str) = serde_json::from_str::<String>(trimmed) {
            // Check if the inner string is itself JSON
            if let Ok(json_val) = serde_json::from_str::<Value>(&parsed_str) {
                return serde_json::to_string_pretty(&json_val).map_err(from_error);
            } else {
                return Ok(parsed_str);
            }
        }
    }

    // Attempt 2: Direct character-level escape decoding
    let unescaped = unescape_raw_string(trimmed);

    // If the unescaped string is valid JSON, format it prettily
    if let Ok(json_val) = serde_json::from_str::<Value>(&unescaped) {
        return serde_json::to_string_pretty(&json_val).map_err(from_error);
    }

    // Attempt 3: If wrapping in quotes parses as a JSON string containing JSON
    let wrapped = format!("\"{}\"", trimmed);
    if let Ok(parsed_str) = serde_json::from_str::<String>(&wrapped) {
        if let Ok(json_val) = serde_json::from_str::<Value>(&parsed_str) {
            return serde_json::to_string_pretty(&json_val).map_err(from_error);
        }
    }

    Ok(unescaped)
}

fn unescape_raw_string(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next() {
                Some('"') => out.push('"'),
                Some('\\') => out.push('\\'),
                Some('/') => out.push('/'),
                Some('b') => out.push('\x08'),
                Some('f') => out.push('\x0c'),
                Some('n') => out.push('\n'),
                Some('r') => out.push('\r'),
                Some('t') => out.push('\t'),
                Some('u') => {
                    let mut hex = String::with_capacity(4);
                    for _ in 0..4 {
                        if let Some(&hc) = chars.peek() {
                            if hc.is_ascii_hexdigit() {
                                hex.push(hc);
                                chars.next();
                            } else {
                                break;
                            }
                        }
                    }
                    if hex.len() == 4 {
                        if let Ok(code) = u32::from_str_radix(&hex, 16) {
                            if let Some(ch) = char::from_u32(code) {
                                out.push(ch);
                                continue;
                            }
                        }
                    }
                    out.push('\\');
                    out.push('u');
                    out.push_str(&hex);
                }
                Some(other) => {
                    out.push(other);
                }
                None => {
                    out.push('\\');
                }
            }
        } else {
            out.push(c);
        }
    }

    out
}

fn from_error(err: serde_json::Error) -> JsonIssue {
    let full = err.to_string();
    let message = full
        .split(" at line ")
        .next()
        .unwrap_or(&full)
        .trim()
        .to_string();
    JsonIssue {
        line: err.line(),
        column: err.column(),
        message,
    }
}

// -----------------------------------------------------------------------------
// JSON Tree Folding Model
// -----------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeType {
    Object,
    Array,
    String,
    Number,
    Boolean,
    Null,
    CloseBrace,
    CloseBracket,
}

impl NodeType {
    pub fn as_str(&self) -> &'static str {
        match self {
            NodeType::Object => "object",
            NodeType::Array => "array",
            NodeType::String => "string",
            NodeType::Number => "number",
            NodeType::Boolean => "boolean",
            NodeType::Null => "null",
            NodeType::CloseBrace | NodeType::CloseBracket => "close",
        }
    }
}

#[derive(Debug, Clone)]
pub struct RawTreeNode {
    pub id: usize,
    #[allow(dead_code)]
    pub parent: Option<usize>,
    pub depth: usize,
    pub key_text: String,
    pub node_type: NodeType,
    pub value_text: String,
    pub summary_text: String,
    pub is_expandable: bool,
    pub is_expanded: bool,
    pub has_comma: bool,
    pub children: Vec<usize>,
    pub close_node_id: Option<usize>,
}

#[derive(Debug, Clone)]
pub struct JsonTree {
    pub nodes: Vec<RawTreeNode>,
}

impl JsonTree {
    pub fn from_value(value: &Value) -> Self {
        let mut nodes = Vec::new();
        let _ = build_tree_recursive(value, None, 0, String::new(), false, &mut nodes);
        Self { nodes }
    }

    pub fn toggle(&mut self, node_id: usize) {
        if let Some(node) = self.nodes.get_mut(node_id) {
            if node.is_expandable {
                node.is_expanded = !node.is_expanded;
            }
        }
    }

    pub fn expand_all(&mut self) {
        for node in &mut self.nodes {
            if node.is_expandable {
                node.is_expanded = true;
            }
        }
    }

    pub fn collapse_all(&mut self) {
        for node in &mut self.nodes {
            if node.is_expandable {
                node.is_expanded = false;
            }
        }
    }

    pub fn fold_level(&mut self, max_depth: usize) {
        for node in &mut self.nodes {
            if node.is_expandable {
                node.is_expanded = node.depth < max_depth;
            }
        }
    }

    /// Compute visible items for Slint model based on expanded state
    pub fn visible_nodes(&self) -> Vec<RawTreeNode> {
        let mut visible = Vec::new();
        if self.nodes.is_empty() {
            return visible;
        }

        // We collect nodes starting from root (id 0)
        self.collect_visible(0, &mut visible);
        visible
    }

    fn collect_visible(&self, node_id: usize, visible: &mut Vec<RawTreeNode>) {
        if node_id >= self.nodes.len() {
            return;
        }
        let node = &self.nodes[node_id];
        visible.push(node.clone());

        if node.is_expandable && node.is_expanded {
            for &child_id in &node.children {
                self.collect_visible(child_id, visible);
            }
            if let Some(close_id) = node.close_node_id {
                if close_id < self.nodes.len() {
                    visible.push(self.nodes[close_id].clone());
                }
            }
        }
    }
}

fn build_tree_recursive(
    value: &Value,
    parent: Option<usize>,
    depth: usize,
    key_text: String,
    has_comma: bool,
    nodes: &mut Vec<RawTreeNode>,
) -> usize {
    let id = nodes.len();
    match value {
        Value::Object(map) => {
            let is_empty = map.is_empty();
            let summary_text = format!("{{ {} 项 }}", map.len());
            let node = RawTreeNode {
                id,
                parent,
                depth,
                key_text,
                node_type: NodeType::Object,
                value_text: if is_empty { "{}".into() } else { "{".into() },
                summary_text,
                is_expandable: !is_empty,
                is_expanded: true,
                has_comma,
                children: Vec::new(),
                close_node_id: None,
            };
            nodes.push(node);

            if !is_empty {
                let mut child_ids = Vec::with_capacity(map.len());
                let total = map.len();
                for (idx, (k, v)) in map.iter().enumerate() {
                    let child_comma = idx + 1 < total;
                    let k_display = format!("\"{}\": ", k);
                    let child_id = build_tree_recursive(
                        v,
                        Some(id),
                        depth + 1,
                        k_display,
                        child_comma,
                        nodes,
                    );
                    child_ids.push(child_id);
                }

                // Add closing node
                let close_id = nodes.len();
                let close_node = RawTreeNode {
                    id: close_id,
                    parent: Some(id),
                    depth,
                    key_text: String::new(),
                    node_type: NodeType::CloseBrace,
                    value_text: "}".into(),
                    summary_text: String::new(),
                    is_expandable: false,
                    is_expanded: false,
                    has_comma,
                    children: Vec::new(),
                    close_node_id: None,
                };
                nodes.push(close_node);

                nodes[id].children = child_ids;
                nodes[id].close_node_id = Some(close_id);
            }

            id
        }
        Value::Array(arr) => {
            let is_empty = arr.is_empty();
            let summary_text = format!("[ {} 项 ]", arr.len());
            let node = RawTreeNode {
                id,
                parent,
                depth,
                key_text,
                node_type: NodeType::Array,
                value_text: if is_empty { "[]".into() } else { "[".into() },
                summary_text,
                is_expandable: !is_empty,
                is_expanded: true,
                has_comma,
                children: Vec::new(),
                close_node_id: None,
            };
            nodes.push(node);

            if !is_empty {
                let mut child_ids = Vec::with_capacity(arr.len());
                let total = arr.len();
                for (idx, v) in arr.iter().enumerate() {
                    let child_comma = idx + 1 < total;
                    let k_display = format!("[{}]: ", idx);
                    let child_id = build_tree_recursive(
                        v,
                        Some(id),
                        depth + 1,
                        k_display,
                        child_comma,
                        nodes,
                    );
                    child_ids.push(child_id);
                }

                let close_id = nodes.len();
                let close_node = RawTreeNode {
                    id: close_id,
                    parent: Some(id),
                    depth,
                    key_text: String::new(),
                    node_type: NodeType::CloseBracket,
                    value_text: "]".into(),
                    summary_text: String::new(),
                    is_expandable: false,
                    is_expanded: false,
                    has_comma,
                    children: Vec::new(),
                    close_node_id: None,
                };
                nodes.push(close_node);

                nodes[id].children = child_ids;
                nodes[id].close_node_id = Some(close_id);
            }

            id
        }
        Value::String(s) => {
            let node = RawTreeNode {
                id,
                parent,
                depth,
                key_text,
                node_type: NodeType::String,
                value_text: format!("\"{}\"", s),
                summary_text: String::new(),
                is_expandable: false,
                is_expanded: false,
                has_comma,
                children: Vec::new(),
                close_node_id: None,
            };
            nodes.push(node);
            id
        }
        Value::Number(n) => {
            let node = RawTreeNode {
                id,
                parent,
                depth,
                key_text,
                node_type: NodeType::Number,
                value_text: n.to_string(),
                summary_text: String::new(),
                is_expandable: false,
                is_expanded: false,
                has_comma,
                children: Vec::new(),
                close_node_id: None,
            };
            nodes.push(node);
            id
        }
        Value::Bool(b) => {
            let node = RawTreeNode {
                id,
                parent,
                depth,
                key_text,
                node_type: NodeType::Boolean,
                value_text: b.to_string(),
                summary_text: String::new(),
                is_expandable: false,
                is_expanded: false,
                has_comma,
                children: Vec::new(),
                close_node_id: None,
            };
            nodes.push(node);
            id
        }
        Value::Null => {
            let node = RawTreeNode {
                id,
                parent,
                depth,
                key_text,
                node_type: NodeType::Null,
                value_text: "null".into(),
                summary_text: String::new(),
                is_expandable: false,
                is_expanded: false,
                has_comma,
                children: Vec::new(),
                close_node_id: None,
            };
            nodes.push(node);
            id
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_object() {
        let out = format_json("{\"a\":1}").unwrap();
        assert_eq!(out, "{\n  \"a\": 1\n}");
    }

    #[test]
    fn minifies_object() {
        let out = minify_json("{\n  \"a\": 1\n}").unwrap();
        assert_eq!(out, "{\"a\":1}");
    }

    #[test]
    fn validate_ok() {
        assert!(validate_json("[1, true, null]").is_ok());
    }

    #[test]
    fn reports_line_and_column() {
        let err = validate_json("{\n  \"a\": 1,\n}").unwrap_err();
        assert_eq!(err.line, 3);
        assert_eq!(err.column, 1);
        assert!(!err.display().contains(" at line "));
        assert!(err.display().starts_with("第 3 行第 1 列："));
    }

    #[test]
    fn empty_is_detected() {
        assert!(empty_input("  \n\t"));
        assert!(!empty_input("{}"));
    }

    #[test]
    fn unescapes_quoted_json_string() {
        let input = r#""{\"name\":\"Alice\",\"age\":30}""#;
        let out = unescape_json(input).unwrap();
        assert_eq!(out, "{\n  \"name\": \"Alice\",\n  \"age\": 30\n}");
    }

    #[test]
    fn unescapes_unquoted_escaped_json() {
        let input = r#"{\"name\":\"Bob\",\"arr\":[1,2]}"#;
        let out = unescape_json(input).unwrap();
        assert_eq!(out, "{\n  \"name\": \"Bob\",\n  \"arr\": [\n    1,\n    2\n  ]\n}");
    }

    #[test]
    fn unescapes_standard_escapes() {
        let input = r#"hello\nworld\t\"quoted\""#;
        let out = unescape_json(input).unwrap();
        assert_eq!(out, "hello\nworld\t\"quoted\"");
    }

    #[test]
    fn tree_model_folding() {
        let val: Value = serde_json::from_str(r#"{"a": [1, 2], "b": {"c": 3}}"#).unwrap();
        let mut tree = JsonTree::from_value(&val);

        // Initially expanded
        let vis1 = tree.visible_nodes();
        assert!(vis1.len() > 5);

        // Collapse all
        tree.collapse_all();
        let vis2 = tree.visible_nodes();
        assert_eq!(vis2.len(), 1); // Only root object

        // Expand root
        tree.toggle(0);
        let vis3 = tree.visible_nodes();
        // Root + children (collapsed "a" and "b") + close brace
        assert!(vis3.len() > 1);

        // Expand all again
        tree.expand_all();
        let vis4 = tree.visible_nodes();
        assert_eq!(vis4.len(), vis1.len());
    }
}
