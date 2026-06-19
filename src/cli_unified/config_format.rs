use anyhow::{Context, Result};
use serde_json::Value as JsonValue;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigValue {
    String(String),
    Bool(bool),
    StringList(Vec<String>),
}

pub fn parse_config_values(content: &str) -> Result<HashMap<String, ConfigValue>> {
    let trimmed = content.trim_start();
    if trimmed.starts_with('{') {
        return parse_json_values(trimmed);
    }

    Ok(parse_human_values(content))
}

fn parse_json_values(content: &str) -> Result<HashMap<String, ConfigValue>> {
    let json: JsonValue = serde_json::from_str(content).context("invalid JSON media config")?;
    let mut values = HashMap::new();
    collect_json_values("", &json, &mut values);
    Ok(values)
}

fn collect_json_values(prefix: &str, value: &JsonValue, values: &mut HashMap<String, ConfigValue>) {
    match value {
        JsonValue::Object(map) => {
            for (key, value) in map {
                let next = join_path(prefix, key);
                collect_json_values(&next, value, values);
            }
        }
        JsonValue::String(value) => {
            insert_value(prefix, ConfigValue::String(value.clone()), values)
        }
        JsonValue::Bool(value) => insert_value(prefix, ConfigValue::Bool(*value), values),
        JsonValue::Array(items) => {
            let strings = items
                .iter()
                .filter_map(|item| match item {
                    JsonValue::String(value) => Some(value.clone()),
                    _ => None,
                })
                .collect();
            insert_value(prefix, ConfigValue::StringList(strings), values);
        }
        _ => {}
    }
}

fn parse_human_values(content: &str) -> HashMap<String, ConfigValue> {
    let mut values = HashMap::new();
    let mut stack: Vec<(usize, String)> = Vec::new();

    for raw_line in content.lines() {
        let Some(line) = strip_comment(raw_line) else {
            continue;
        };
        if line.trim().is_empty() {
            continue;
        }

        let indent = line.chars().take_while(|c| c.is_whitespace()).count();
        let trimmed = line.trim();
        let Some(separator) = find_separator(trimmed) else {
            continue;
        };

        let key = trimmed[..separator].trim();
        let value = trimmed[separator + 1..].trim();
        if key.is_empty() {
            continue;
        }

        while stack.last().is_some_and(|(level, _)| *level >= indent) {
            stack.pop();
        }

        let parent = stack.last().map(|(_, path)| path.as_str()).unwrap_or("");
        let path = join_path(parent, key);
        if value.is_empty() {
            stack.push((indent, path));
            continue;
        }

        if let Some(value) = parse_scalar(value) {
            insert_value(&path, value, &mut values);
        }
    }

    values
}

fn insert_value(path: &str, value: ConfigValue, values: &mut HashMap<String, ConfigValue>) {
    if path.is_empty() {
        return;
    }

    values.insert(normalize_path(path), value);
}

fn normalize_path(path: &str) -> String {
    path.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .strip_prefix("context.")
        .unwrap_or(path.trim())
        .split('.')
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(".")
}

fn join_path(parent: &str, key: &str) -> String {
    let key = normalize_key(key);
    if parent.is_empty() {
        key
    } else if key.is_empty() {
        parent.to_string()
    } else {
        format!("{parent}.{key}")
    }
}

fn normalize_key(key: &str) -> String {
    key.trim()
        .trim_matches('"')
        .trim_matches('\'')
        .trim_end_matches(':')
        .trim()
        .to_string()
}

fn parse_scalar(value: &str) -> Option<ConfigValue> {
    let value = value.trim().trim_end_matches(',').trim();
    if value.eq_ignore_ascii_case("true") {
        return Some(ConfigValue::Bool(true));
    }
    if value.eq_ignore_ascii_case("false") {
        return Some(ConfigValue::Bool(false));
    }
    if value.starts_with('[') && value.ends_with(']') {
        return Some(ConfigValue::StringList(split_array_items(
            &value[1..value.len() - 1],
        )));
    }

    Some(ConfigValue::String(unquote(value)))
}

fn split_array_items(value: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;

    for ch in value.chars() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'' | '"') => quote = Some(ch),
            (None, ',') => {
                let item = unquote(current.trim());
                if !item.is_empty() {
                    items.push(item);
                }
                current.clear();
            }
            _ => current.push(ch),
        }
    }

    let item = unquote(current.trim());
    if !item.is_empty() {
        items.push(item);
    }

    items
}

fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string()
}

fn strip_comment(line: &str) -> Option<&str> {
    let trimmed = line.trim_start();
    if trimmed.starts_with('#') || trimmed.starts_with("//") {
        return None;
    }

    let mut quote: Option<char> = None;
    for (index, ch) in line.char_indices() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'' | '"') => quote = Some(ch),
            (None, '#') => return Some(&line[..index]),
            (None, '/') if line[index..].starts_with("//") => return Some(&line[..index]),
            _ => {}
        }
    }

    Some(line)
}

fn find_separator(line: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (index, ch) in line.char_indices() {
        match (quote, ch) {
            (Some(q), c) if c == q => quote = None,
            (None, '\'' | '"') => quote = Some(ch),
            (None, ':' | '=') => return Some(index),
            _ => {}
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dotted_dx_config_values() {
        let values = parse_config_values(
            r#"
            media.cli.base_dir: "./assets"
            media.cli.auto_create: false
            media.cli.fonts.formats: ["ttf", "woff2"]
            "#,
        )
        .unwrap();

        assert_eq!(
            values.get("media.cli.base_dir"),
            Some(&ConfigValue::String("./assets".to_string()))
        );
        assert_eq!(
            values.get("media.cli.auto_create"),
            Some(&ConfigValue::Bool(false))
        );
        assert_eq!(
            values.get("media.cli.fonts.formats"),
            Some(&ConfigValue::StringList(vec![
                "ttf".to_string(),
                "woff2".to_string()
            ]))
        );
    }

    #[test]
    fn parses_indented_config_values() {
        let values = parse_config_values(
            r#"
            media:
              cli:
                directories:
                  media: images
                providers:
                  default_media: openverse
            "#,
        )
        .unwrap();

        assert_eq!(
            values.get("media.cli.directories.media"),
            Some(&ConfigValue::String("images".to_string()))
        );
        assert_eq!(
            values.get("media.cli.providers.default_media"),
            Some(&ConfigValue::String("openverse".to_string()))
        );
    }

    #[test]
    fn parses_json_config_values() {
        let values = parse_config_values(
            r#"{
              "media": {
                "cli": {
                  "organize_by_type": true,
                  "fonts": { "subsets": ["latin", "latin-ext"] }
                }
              }
            }"#,
        )
        .unwrap();

        assert_eq!(
            values.get("media.cli.organize_by_type"),
            Some(&ConfigValue::Bool(true))
        );
        assert_eq!(
            values.get("media.cli.fonts.subsets"),
            Some(&ConfigValue::StringList(vec![
                "latin".to_string(),
                "latin-ext".to_string()
            ]))
        );
    }
}
