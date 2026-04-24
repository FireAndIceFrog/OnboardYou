use std::sync::{Arc, Mutex};

use async_trait::async_trait;

use crate::PipelineWarning;
/// Shared collector that Polars `.map()` closures can push warnings into.
///
/// Because LazyFrames defer execution, closures inside `.map()` run at
/// collection time — long after `execute()` returns. This `Arc<Mutex<_>>`
/// lets the closures write warnings that are later drained by the pipeline
/// runner.
pub type WarningCollector = Arc<Mutex<Vec<PipelineWarning>>>;

/// Repository trait used by the pipeline engine to fetch pipeline configs.
#[async_trait]
pub trait IPipelineLogger: Send + Sync {
    fn warn(&self, warning: PipelineWarning);
    fn drain_deferred_warnings(&self) -> Vec<PipelineWarning>;
}

pub struct Logging {
    /// Shared collector for warnings emitted inside Polars `.map()` closures.
    /// Cloned into closures; drained after `.collect()` by the pipeline runner.
    warning_collector: WarningCollector,
}
impl Default for Logging {
    fn default() -> Self {
        Self::new()
    }
}

impl Logging {
    pub fn new() -> Self {
        Self {
            warning_collector: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl IPipelineLogger for Logging {
    /// Record a non-fatal warning that will be surfaced in the run log.
    fn warn(&self, warning: PipelineWarning) {
        if let Ok(mut guard) = self.warning_collector.lock() {
            guard.push(warning);
        }
    }
    /// Drain deferred warnings from the shared collector.
    ///
    /// Call this after the LazyFrame has been collected (materialised) so that
    /// any warnings emitted inside Polars `.map()` closures are captured.
    fn drain_deferred_warnings(&self) -> Vec<PipelineWarning> {
        if let Ok(mut guard) = self.warning_collector.lock() {
            guard.drain(..).collect()
        } else {
            Vec::new()
        }
    }
}