use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorDocument {
    pub vector: Vec<f32>,
    pub id: Option<String>,
    pub metadata: Option<Value>,
    pub content: Option<String>,
}

impl VectorDocument {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({ "vector": self.vector });
        if let Some(id) = &self.id {
            payload["id"] = json!(id);
        }
        if let Some(meta) = &self.metadata {
            payload["metadata"] = meta.clone();
        }
        if let Some(content) = &self.content {
            payload["content"] = json!(content);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorSearchOptions {
    pub query_vector: Vec<f32>,
    pub limit: Option<i32>,
    pub filter: Option<Value>,
    pub min_score: Option<f32>,
    pub max_distance: Option<f32>,
    pub include_distance: Option<bool>,
    pub include_content: Option<bool>,
}

impl VectorSearchOptions {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({ "queryVector": self.query_vector });
        if let Some(limit) = self.limit {
            payload["limit"] = json!(limit);
        }
        if let Some(filter) = &self.filter {
            payload["filter"] = filter.clone();
        }
        if let Some(min_score) = self.min_score {
            payload["minScore"] = json!(min_score);
        }
        if let Some(max_distance) = self.max_distance {
            payload["maxDistance"] = json!(max_distance);
        }
        if let Some(include_distance) = self.include_distance {
            payload["includeDistance"] = json!(include_distance);
        }
        if let Some(include_content) = self.include_content {
            payload["includeContent"] = json!(include_content);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorBatchInsertOptions {
    pub documents: Vec<VectorDocument>,
    pub skip_duplicates: Option<bool>,
}

impl VectorBatchInsertOptions {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({
            "documents": self.documents.iter().map(|d| d.to_json()).collect::<Vec<_>>()
        });
        if let Some(skip) = self.skip_duplicates {
            payload["skipDuplicates"] = json!(skip);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct VectorCollectionConfig {
    pub dimension: Option<i32>,
    pub distance: Option<String>,
    pub options: Option<Value>,
}

impl VectorCollectionConfig {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({});
        if let Some(dimension) = self.dimension {
            payload["dimension"] = json!(dimension);
        }
        if let Some(distance) = &self.distance {
            payload["distance"] = json!(distance);
        }
        if let Some(options) = &self.options {
            payload["options"] = options.clone();
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LangChaingoModelConfig {
    pub provider: Option<String>,
    pub model: Option<String>,
    pub api_key: Option<String>,
    pub base_url: Option<String>,
}

impl LangChaingoModelConfig {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({});
        if let Some(provider) = &self.provider {
            payload["provider"] = json!(provider);
        }
        if let Some(model) = &self.model {
            payload["model"] = json!(model);
        }
        if let Some(api_key) = &self.api_key {
            payload["apiKey"] = json!(api_key);
        }
        if let Some(base_url) = &self.base_url {
            payload["baseUrl"] = json!(base_url);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LangChaingoCompletionMessage {
    pub content: String,
    pub role: Option<String>,
}

impl LangChaingoCompletionMessage {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({ "content": self.content });
        if let Some(role) = &self.role {
            payload["role"] = json!(role);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LangChaingoCompletionRequest {
    pub model: Option<LangChaingoModelConfig>,
    pub prompt: Option<String>,
    pub messages: Vec<LangChaingoCompletionMessage>,
    pub temperature: Option<f64>,
    pub max_tokens: Option<i32>,
    pub top_p: Option<f64>,
    pub candidate_count: Option<i32>,
    pub stop: Option<Vec<String>>,
    pub json_response: Option<bool>,
}

impl LangChaingoCompletionRequest {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({});
        if let Some(model) = &self.model {
            payload["model"] = model.to_json();
        }
        if let Some(prompt) = &self.prompt {
            payload["prompt"] = json!(prompt);
        }
        if !self.messages.is_empty() {
            payload["messages"] = Value::Array(self.messages.iter().map(|m| m.to_json()).collect());
        }
        if let Some(temp) = self.temperature {
            payload["temperature"] = json!(temp);
        }
        if let Some(max_tokens) = self.max_tokens {
            payload["maxTokens"] = json!(max_tokens);
        }
        if let Some(top_p) = self.top_p {
            payload["topP"] = json!(top_p);
        }
        if let Some(candidate_count) = self.candidate_count {
            payload["candidateCount"] = json!(candidate_count);
        }
        if let Some(stop) = &self.stop {
            payload["stop"] = json!(stop);
        }
        if let Some(json_response) = self.json_response {
            payload["json"] = json!(json_response);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LangChaingoRAGFilters {
    pub where_: Option<HashMap<String, String>>,
    pub where_document: Option<HashMap<String, String>>,
}

impl LangChaingoRAGFilters {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({});
        if let Some(where_) = &self.where_ {
            payload["where"] = json!(where_);
        }
        if let Some(where_doc) = &self.where_document {
            payload["whereDocument"] = json!(where_doc);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LangChaingoRAGRequest {
    pub collection: String,
    pub question: String,
    pub model: Option<LangChaingoModelConfig>,
    pub top_k: Option<i32>,
    pub score_threshold: Option<f64>,
    pub filters: Option<LangChaingoRAGFilters>,
    pub prompt_template: Option<String>,
    pub return_sources: Option<bool>,
}

impl LangChaingoRAGRequest {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({
            "collection": self.collection,
            "question": self.question
        });
        if let Some(model) = &self.model {
            payload["model"] = model.to_json();
        }
        if let Some(top_k) = self.top_k {
            payload["topK"] = json!(top_k);
        }
        if let Some(score_threshold) = self.score_threshold {
            payload["scoreThreshold"] = json!(score_threshold);
        }
        if let Some(filters) = &self.filters {
            payload["filters"] = filters.to_json();
        }
        if let Some(prompt_template) = &self.prompt_template {
            payload["promptTemplate"] = json!(prompt_template);
        }
        if let Some(return_sources) = self.return_sources {
            payload["returnSources"] = json!(return_sources);
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LLMDocument {
    pub id: String,
    pub content: String,
    pub metadata: Option<Value>,
}

impl LLMDocument {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({
            "id": self.id,
            "content": self.content
        });
        if let Some(meta) = &self.metadata {
            payload["metadata"] = meta.clone();
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LLMDocumentUpdate {
    pub content: Option<String>,
    pub metadata: Option<Value>,
}

impl LLMDocumentUpdate {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({});
        if let Some(content) = &self.content {
            payload["content"] = json!(content);
        }
        if let Some(metadata) = &self.metadata {
            payload["metadata"] = metadata.clone();
        }
        payload
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LLMQueryOptions {
    pub query: String,
    pub top_k: Option<i32>,
    pub filter: Option<Value>,
    pub include_document: Option<bool>,
}

impl LLMQueryOptions {
    pub fn to_json(&self) -> Value {
        let mut payload = json!({ "query": self.query });
        if let Some(top_k) = self.top_k {
            payload["topK"] = json!(top_k);
        }
        if let Some(filter) = &self.filter {
            payload["filter"] = filter.clone();
        }
        if let Some(include) = self.include_document {
            payload["includeDocument"] = json!(include);
        }
        payload
    }
}
