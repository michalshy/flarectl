pub fn empty_object() -> serde_json::Value {
    serde_json::Value::Object(Default::default()) // same as json!({})
}
