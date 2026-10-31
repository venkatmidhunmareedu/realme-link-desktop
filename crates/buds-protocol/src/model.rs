//! Per-model config, decoded from the app's `headsetRusConfig` assets (crates/buds-protocol/models.json).
//! The UI is driven by `view_items`; no model is special-cased.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Deserialize)]
pub struct ModelConfig {
    #[serde(rename = "androidConfig")]
    pub android: AndroidConfig,
    #[serde(rename = "productAttr", default)]
    pub product: serde_json::Value,
    #[serde(rename = "viewItems", default)]
    pub view_items: Vec<ViewItem>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AndroidConfig {
    #[serde(rename = "sdkVendor", default)]
    pub vendor: String,
    #[serde(rename = "modelId", default)]
    pub model_id: String,
    #[serde(rename = "sppUUIDType", default)]
    pub spp_uuid_type: String,
    #[serde(rename = "sppCmdInterval", default)]
    pub cmd_interval_ms: i64,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ViewItem {
    #[serde(rename = "itemType")]
    pub item_type: String,
    #[serde(rename = "subItems", default)]
    pub sub_items: Vec<ViewItem>,
    #[serde(rename = "minSupportVersion", default)]
    pub min_version: Option<String>,
}

pub struct Catalog(pub BTreeMap<String, ModelConfig>);

impl Catalog {
    pub fn bundled() -> Self {
        Self(serde_json::from_str(include_str!("../models.json")).expect("models.json"))
    }

    /// Match a Bluetooth name like "realme Buds Air 5 Pro": exact, then longest prefix, case-insensitive.
    pub fn find(&self, bt_name: &str) -> Option<(&str, &ModelConfig)> {
        let n = bt_name.trim().to_lowercase();
        if let Some((k, v)) = self.0.iter().find(|(k, _)| k.to_lowercase() == n) {
            return Some((k, v));
        }
        self.0
            .iter()
            .filter(|(k, _)| n.starts_with(&k.to_lowercase()))
            .max_by_key(|(k, _)| k.len())
            .map(|(k, v)| (k.as_str(), v))
    }
}

impl ModelConfig {
    pub fn spp_uuid(&self) -> &'static str {
        if self.android.spp_uuid_type == "TYPE_SELF_PROTOCOL" {
            crate::UUID_SELF_PROTOCOL
        } else {
            crate::UUID_DEFAULT
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bundled_loads_and_matches() {
        let c = Catalog::bundled();
        assert_eq!(c.0.len(), 34);
        assert_eq!(c.find("realme Buds Air 5 Pro").unwrap().0, "realme Buds Air 5 Pro");
        assert_eq!(c.find("realme Buds T110").unwrap().1.spp_uuid(), crate::UUID_SELF_PROTOCOL);
        assert_eq!(c.find("realme Buds T310").unwrap().1.spp_uuid(), crate::UUID_DEFAULT);
        assert!(c.find("Some Other Buds").is_none());
    }
}
