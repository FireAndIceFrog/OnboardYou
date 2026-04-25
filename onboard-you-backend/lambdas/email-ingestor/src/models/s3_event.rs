use serde::Deserialize;

/// Minimal S3 event types — only fields this Lambda needs.
#[derive(Debug, Deserialize)]
pub struct S3Event {
    #[serde(rename = "Records")]
    pub records: Vec<S3EventRecord>,
}

#[derive(Debug, Deserialize)]
pub struct S3EventRecord {
    pub s3: S3Object,
}

#[derive(Debug, Deserialize)]
pub struct S3Object {
    pub bucket: S3Bucket,
    pub object: S3ObjectKey,
}

#[derive(Debug, Deserialize)]
pub struct S3Bucket {
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct S3ObjectKey {
    pub key: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserializes_s3_event() {
        let json = serde_json::json!({
            "Records": [{
                "s3": {
                    "bucket": { "name": "ses-inbox" },
                    "object": { "key": "email/abc123" }
                }
            }]
        });

        let event: S3Event = serde_json::from_value(json).unwrap();
        assert_eq!(event.records.len(), 1);
        assert_eq!(event.records[0].s3.bucket.name, "ses-inbox");
        assert_eq!(event.records[0].s3.object.key, "email/abc123");
    }
}
