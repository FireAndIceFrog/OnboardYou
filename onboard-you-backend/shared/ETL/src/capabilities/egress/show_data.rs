//! ShowData: writes the current pipeline data as CSV to S3 for UI visualisation.
//!
//! The output key is `{org_id}/{company_id}/outputs/{action_id}.csv`, resolved
//! before pipeline execution so multiple `ShowData` steps in one pipeline each
//! write to their own distinct file.

use onboard_you_models::{ColumnCalculator, Error, OnboardingAction, Result, RosterContext, ShowDataConfig};
use polars::prelude::*;
use tracing::info;

/// Pipeline action that persists the current data snapshot to S3 as CSV.
///
/// After execution the data continues to flow unchanged — `ShowData` is a
/// non-destructive tap, identical to a passthrough from the next action's
/// perspective.
pub struct ShowData {
    s3_key: String,
}

impl ShowData {
    /// Build from a resolved `ShowDataConfig`.
    pub fn from_action_config(config: &ShowDataConfig) -> Result<Self> {
        let s3_key = config.resolved_key()?.to_string();
        Ok(Self { s3_key })
    }
}

impl ColumnCalculator for ShowData {
    /// ShowData does not alter the schema — pass through unchanged.
    fn calculate_columns(&self, context: RosterContext) -> Result<RosterContext> {
        Ok(context)
    }
}

impl OnboardingAction for ShowData {
    fn id(&self) -> &str {
        "show_data"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        let bucket = std::env::var("CSV_UPLOAD_BUCKET").map_err(|_| {
            Error::ConfigurationError("CSV_UPLOAD_BUCKET environment variable is not set".into())
        })?;

        // Collect the lazy plan so we can serialise it.
        let df = context
            .get_data()
            .clone()
            .collect()
            .map_err(|e| Error::EgressError(format!("ShowData: failed to collect data: {e}")))?;

        // Store back as lazy so the pipeline row-count collect doesn't re-fire closures.
        context.set_data(df.clone().lazy());

        // Serialise to CSV bytes.
        let csv_bytes = dataframe_to_csv_bytes(&df)?;

        info!(
            key = %self.s3_key,
            rows = df.height(),
            cols = df.width(),
            "ShowData: uploading output CSV to S3"
        );

        // Upload — bridge async SDK into the sync action interface.
        let key = self.s3_key.clone();
        let handle = tokio::runtime::Handle::current();
        tokio::task::block_in_place(|| {
            handle.block_on(async {
                let aws_cfg = aws_config::load_defaults(aws_config::BehaviorVersion::latest()).await;
                let client = aws_sdk_s3::Client::new(&aws_cfg);
                client
                    .put_object()
                    .bucket(&bucket)
                    .key(&key)
                    .content_type("text/csv")
                    .body(aws_sdk_s3::primitives::ByteStream::from(csv_bytes))
                    .send()
                    .await
                    .map_err(|e| Error::EgressError(format!("ShowData: S3 upload failed: {e}")))
            })
        })?;

        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Serialise a Polars `DataFrame` to CSV bytes.
fn dataframe_to_csv_bytes(df: &DataFrame) -> Result<Vec<u8>> {
    let mut buf: Vec<u8> = Vec::new();
    CsvWriter::new(&mut buf)
        .finish(&mut df.clone())
        .map_err(|e| Error::EgressError(format!("ShowData: CSV serialisation failed: {e}")))?;
    Ok(buf)
}
