use crate::client::BosBaseInner;
use crate::errors::ClientResponseError;
use crate::request::SendOptions;
use crate::services::BaseCrudService;
use crate::utils::{encode_path_segment};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Clone)]
pub struct CollectionService {
    base: BaseCrudService,
}

impl CollectionService {
    pub(crate) fn new(client: Arc<BosBaseInner>) -> Self {
        Self {
            base: BaseCrudService::new(client, "/api/collections"),
        }
    }

    pub fn base_crud_path(&self) -> &str {
        self.base.base_crud_path()
    }

    pub fn get_list(
        &self,
        page: i32,
        per_page: i32,
        skip_total: bool,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        filter: Option<String>,
        sort: Option<String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base
            .get_list(page, per_page, skip_total, query, headers, filter, sort, expand, fields)
    }

    pub fn get_one(
        &self,
        id: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base.get_one(id, query, headers, expand, fields)
    }

    pub fn get_first_list_item(
        &self,
        filter: String,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base
            .get_first_list_item(filter, query, headers, expand, fields)
    }

    pub fn get_full_list(
        &self,
        batch: i32,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        filter: Option<String>,
        sort: Option<String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base
            .get_full_list(batch, query, headers, filter, sort, expand, fields)
    }

    pub fn create(
        &self,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base
            .create(body, query, Vec::new(), headers, expand, fields)
    }

    pub fn update(
        &self,
        id: &str,
        body: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
        expand: Option<String>,
        fields: Option<String>,
    ) -> Result<Value, ClientResponseError> {
        self.base
            .update(id, body, query, Vec::new(), headers, expand, fields)
    }

    pub fn delete_collection(
        &self,
        id_or_name: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        self.base.remove(id_or_name, Value::Null, query, headers)
    }

    pub fn truncate(
        &self,
        id_or_name: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "DELETE".into();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!(
                    "{}/{}{}",
                    self.base_crud_path(),
                    encode_path_segment(id_or_name),
                    "/truncate"
                ),
                opts,
            )
            .map(|_| ())
    }

    pub fn import_collections(
        &self,
        collections: Value,
        delete_missing: bool,
        mut query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        query.insert("deleteMissing".into(), json!(delete_missing));
        let mut opts = SendOptions::default();
        opts.method = "PUT".into();
        opts.body = collections;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(&format!("{}/import", self.base_crud_path()), opts)
    }

    pub fn get_scaffolds(
        &self,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(&format!("{}/scaffolds", self.base_crud_path()), opts)
    }

    pub fn create_from_scaffold(
        &self,
        scaffold_type: &str,
        name: &str,
        overrides: Option<Value>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = json!({ "name": name });
        if let Some(overrides) = overrides {
            payload["overrides"] = overrides;
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/scaffolds/{}",
                self.base_crud_path(),
                encode_path_segment(scaffold_type)
            ),
            opts,
        )
    }

    pub fn create_base(
        &self,
        name: &str,
        overrides: Option<Value>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        self.create_from_scaffold("base", name, overrides, query, headers)
    }

    pub fn create_auth(
        &self,
        name: &str,
        overrides: Option<Value>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        self.create_from_scaffold("auth", name, overrides, query, headers)
    }

    pub fn create_view(
        &self,
        name: &str,
        view_query: Option<String>,
        overrides: Option<Value>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = json!({ "name": name });
        if let Some(vq) = view_query {
            payload["viewQuery"] = json!(vq);
        }
        if let Some(overrides) = overrides {
            payload["overrides"] = overrides;
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(&format!("{}/views", self.base_crud_path()), opts)
    }

    pub fn add_field(
        &self,
        collection: &str,
        field: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = field;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/fields",
                self.base_crud_path(),
                encode_path_segment(collection)
            ),
            opts,
        )
    }

    pub fn update_field(
        &self,
        collection: &str,
        field_name: &str,
        updates: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = updates;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/fields/{}",
                self.base_crud_path(),
                encode_path_segment(collection),
                encode_path_segment(field_name)
            ),
            opts,
        )
    }

    pub fn remove_field(
        &self,
        collection: &str,
        field_name: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "DELETE".into();
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!(
                    "{}/{}/fields/{}",
                    self.base_crud_path(),
                    encode_path_segment(collection),
                    encode_path_segment(field_name)
                ),
                opts,
            )
            .map(|_| ())
    }

    pub fn get_field(
        &self,
        collection: &str,
        field_name: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/fields/{}",
                self.base_crud_path(),
                encode_path_segment(collection),
                encode_path_segment(field_name)
            ),
            opts,
        )
    }

    pub fn add_index(
        &self,
        collection: &str,
        columns: Vec<String>,
        unique: Option<bool>,
        index_name: Option<String>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = json!({ "columns": columns });
        if let Some(unique) = unique {
            payload["unique"] = json!(unique);
        }
        if let Some(index_name) = index_name {
            payload["indexName"] = json!(index_name);
        }
        let mut opts = SendOptions::default();
        opts.method = "POST".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/indexes",
                self.base_crud_path(),
                encode_path_segment(collection)
            ),
            opts,
        )
    }

    pub fn remove_index(
        &self,
        collection: &str,
        columns: Vec<String>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<(), ClientResponseError> {
        let payload = json!({ "columns": columns });
        let mut opts = SendOptions::default();
        opts.method = "DELETE".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base
            .client
            .send(
                &format!(
                    "{}/{}/indexes",
                    self.base_crud_path(),
                    encode_path_segment(collection)
                ),
                opts,
            )
            .map(|_| ())
    }

    pub fn get_indexes(
        &self,
        collection: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/indexes",
                self.base_crud_path(),
                encode_path_segment(collection)
            ),
            opts,
        )
    }

    pub fn set_rules(
        &self,
        collection: &str,
        rules: Value,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = rules;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/rules",
                self.base_crud_path(),
                encode_path_segment(collection)
            ),
            opts,
        )
    }

    pub fn set_rule(
        &self,
        collection: &str,
        rule_type: &str,
        rule: Option<String>,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut payload = json!({});
        payload["rule"] = rule.map(Value::String).unwrap_or(Value::Null);
        let mut opts = SendOptions::default();
        opts.method = "PATCH".into();
        opts.body = payload;
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/rules/{}",
                self.base_crud_path(),
                encode_path_segment(collection),
                encode_path_segment(rule_type)
            ),
            opts,
        )
    }

    pub fn get_rules(
        &self,
        collection: &str,
        query: HashMap<String, Value>,
        headers: HashMap<String, String>,
    ) -> Result<Value, ClientResponseError> {
        let mut opts = SendOptions::default();
        opts.query = query;
        opts.headers = headers;
        self.base.client.send(
            &format!(
                "{}/{}/rules",
                self.base_crud_path(),
                encode_path_segment(collection)
            ),
            opts,
        )
    }
}
