use base64::Engine;
use serde_json::Value;
use std::collections::HashMap;
use urlencoding::encode;

pub fn url_encode(value: &str) -> String {
    encode(value).into_owned()
}

pub fn encode_path_segment(segment: &str) -> String {
    url_encode(segment)
}

pub fn build_query(params: &HashMap<String, Vec<String>>) -> String {
    let mut parts = Vec::new();
    for (key, values) in params {
        for val in values {
            parts.push(format!("{}={}", url_encode(key), url_encode(val)));
        }
    }
    parts.join("&")
}

pub fn normalize_query(params: &HashMap<String, Value>) -> HashMap<String, Vec<String>> {
    let mut normalized = HashMap::new();
    for (key, value) in params {
        if value.is_null() {
            continue;
        }
        if let Some(arr) = value.as_array() {
            let mut list = Vec::new();
            for item in arr {
                if item.is_null() {
                    continue;
                }
                if let Some(s) = item.as_str() {
                    list.push(s.to_string());
                } else {
                    list.push(item.to_string());
                }
            }
            if !list.is_empty() {
                normalized.insert(key.clone(), list);
            }
        } else if let Some(s) = value.as_str() {
            normalized.insert(key.clone(), vec![s.to_string()]);
        } else {
            normalized.insert(key.clone(), vec![value.to_string()]);
        }
    }
    normalized
}

pub fn build_relative_url(path: &str, query: &HashMap<String, Value>) -> String {
    let mut rel = if path.starts_with('/') {
        path.to_string()
    } else {
        format!("/{}", path)
    };
    if !query.is_empty() {
        let normalized = normalize_query(query);
        let qs = build_query(&normalized);
        if !qs.is_empty() {
            rel.push('?');
            rel.push_str(&qs);
        }
    }
    rel
}

pub fn base64_url_decode(input: &str) -> Option<Vec<u8>> {
    let mut data = input.replace('-', "+").replace('_', "/");
    while data.len() % 4 != 0 {
        data.push('=');
    }
    base64::engine::general_purpose::STANDARD.decode(data).ok()
}

pub fn to_serializable(value: &Value) -> Value {
    value.clone()
}
