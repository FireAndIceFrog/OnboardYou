use std::sync::Arc;

use crate::{IPipelineLogger, Logging};
#[derive(Clone)]
pub struct ETLDependancies {
    pub logger: Arc<dyn IPipelineLogger>,
}

impl ETLDependancies {
    pub fn new(logger: Arc<dyn IPipelineLogger>) -> Self {
        Self { logger }
    }
}

impl Default for ETLDependancies {
    fn default() -> Self {
        Self {
            logger: Arc::new(Logging::new()),
        }
    }
}