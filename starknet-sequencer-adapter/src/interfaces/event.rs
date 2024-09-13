use serde_json::Value;

#[derive(serde::Serialize)]
pub struct EventMap<'a> {
    pub transaction_index: Option<&'a Value>,
    pub execution_status: Option<&'a Value>,
    pub events: &'a [Value], // Utilisation de slice, toujours présent même s'il est vide
}