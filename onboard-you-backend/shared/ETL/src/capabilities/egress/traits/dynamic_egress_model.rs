use std::collections::HashMap;

pub trait DynamicEgressModel {
    fn schema(&self) -> HashMap<String, String>;
}