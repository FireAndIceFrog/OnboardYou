use std::collections::HashMap;
pub trait DynamicEgressModel {
    fn get_schema(&self) -> HashMap<String, String>;
    fn set_schema(&mut self, schema: HashMap<String, String>);
}

#[macro_export]
macro_rules! DynamicEgressModel {
    (
        // Match the attributes, visibility, name, generics, and fields of ONE struct
        $(#[$attr:meta])*
        $vis:vis struct $name:ident $(<$($gen:tt)*>)? $(where $($where:tt)*)? {
            $($fields:tt)*
        }
    ) => {
        // 1. Emit the struct with the injected field
        $(#[$attr])*
        $vis struct $name $(<$($gen)*>)? $(where $($where)*)? {
            #[serde(default)]
            pub schema: ::std::collections::HashMap<String, String>,
            pub body_path: Option<String>,
            $($fields)*
        }

        // 2. Emit the trait implementation
        impl $(<$($gen)*>)? $crate::DynamicEgressModel for $name $(<$($gen)*>)? $(where $($where)*)? {
            fn get_schema(&self) -> ::std::collections::HashMap<String, String> {
                self.schema.clone()
            }
            fn set_schema(&mut self, schema: ::std::collections::HashMap<String, String>) {
                self.schema = schema;
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use utoipa::ToSchema;

    use super::*;

    #[macro_rules_attribute::apply(DynamicEgressModel!)]
    #[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
    pub struct TestStruct {
        #[serde(default)]
        pub placement: String,
    }


    #[test]
    fn test_derive_works() {
        let mut instance = TestStruct { body_path: None, schema: HashMap::new(), placement: "test".to_string() };

        instance.set_schema(HashMap::new());
        instance.get_schema();
    }
}