use mailparse::MailHeaderMap;

/// Parsed representation of an inbound email's key fields.
pub struct ParsedEmail {
    /// Bare sender address extracted from the `From` header.
    pub sender: String,
    /// Value of the `Subject` header (empty string if absent).
    pub subject: String,
    /// The first non-inline attachment found: `(filename, raw_bytes)`.
    pub attachment: Option<(String, Vec<u8>)>,
}

/// Parse raw RFC 2822 email bytes into a [`ParsedEmail`].
pub fn parse(raw: &[u8]) -> Result<ParsedEmail, String> {
    let parsed = mailparse::parse_mail(raw)
        .map_err(|e| format!("MIME parse failed: {e}"))?;

    let from_raw = parsed.headers.get_first_value("From").unwrap_or_default();
    let sender = extract_address(&from_raw);
    let subject = parsed.headers.get_first_value("Subject").unwrap_or_default();
    let attachment = find_best_attachment(&parsed);

    Ok(ParsedEmail { sender, subject, attachment })
}

/// Strip RFC 2822 display-name decoration and return the bare address.
///
/// `"Alice Smith <alice@acme.com>"` → `"alice@acme.com"`
fn extract_address(from: &str) -> String {
    if let Some(start) = from.find('<') {
        if let Some(end) = from.find('>') {
            return from[start + 1..end].trim().to_string();
        }
    }
    from.trim().to_string()
}

/// Score an attachment filename so we prefer tabular files over images.
///
/// Higher is better. Scores:
/// - CSV                           → 100
/// - Spreadsheet (xlsx/xls/ods)    →  90
/// - Document with tables (pdf/docx/doc) → 70
/// - Unknown extension             →  50
/// - Image (png/jpg/jpeg/gif/webp/bmp/tiff/svg) →  10
fn attachment_score(filename: &str) -> u8 {
    match filename.rsplit('.').next().map(|e| e.to_lowercase()).as_deref() {
        Some("csv")                          => 100,
        Some("xlsx") | Some("xls") | Some("ods") =>  90,
        Some("pdf") | Some("docx") | Some("doc") =>  70,
        Some("png") | Some("jpg") | Some("jpeg")
        | Some("gif") | Some("webp") | Some("bmp")
        | Some("tiff") | Some("tif") | Some("svg") =>  10,
        _ =>  50,
    }
}

/// Recursively collect every `Content-Disposition: attachment` part in the
/// MIME tree, then return the one with the highest [`attachment_score`].
///
/// Images score lowest so a CSV or spreadsheet always wins, but if the only
/// attachment is an image it is still returned (for Textract conversion).
fn find_best_attachment(msg: &mailparse::ParsedMail) -> Option<(String, Vec<u8>)> {
    let mut candidates: Vec<(String, Vec<u8>)> = Vec::new();
    collect_attachments(msg, &mut candidates);

    candidates
        .into_iter()
        .max_by_key(|(name, _)| attachment_score(name))
}

fn collect_attachments(msg: &mailparse::ParsedMail, out: &mut Vec<(String, Vec<u8>)>) {
    if msg.get_content_disposition().disposition == mailparse::DispositionType::Attachment {
        if let Some(filename) = msg
            .get_content_disposition()
            .params
            .get("filename")
            .map(|s| s.to_string())
        {
            if let Ok(body) = msg.get_body_raw() {
                out.push((filename, body));
            }
        }
    }

    for sub in &msg.subparts {
        collect_attachments(sub, out);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ── extract_address ─────────────────────────────────────────────────────

    #[test]
    fn extract_address_plain_email() {
        assert_eq!(extract_address("hr@acme.com"), "hr@acme.com");
    }

    #[test]
    fn extract_address_display_name() {
        assert_eq!(
            extract_address("Alice Smith <alice@acme.com>"),
            "alice@acme.com"
        );
    }

    #[test]
    fn extract_address_trims_whitespace() {
        assert_eq!(extract_address("  hr@acme.com  "), "hr@acme.com");
    }

    #[test]
    fn extract_address_unclosed_angle_bracket_falls_back_to_trim() {
        assert_eq!(extract_address("Alice <no-close"), "Alice <no-close");
    }

    // ── parse ────────────────────────────────────────────────────────────────

    #[test]
    fn parse_simple_text_email() {
        let raw = b"From: hr@acme.com\r\nSubject: Monthly Roster\r\n\r\nHello";
        let email = parse(raw).unwrap();
        assert_eq!(email.sender, "hr@acme.com");
        assert_eq!(email.subject, "Monthly Roster");
        assert!(email.attachment.is_none());
    }

    #[test]
    fn parse_extracts_display_name_sender() {
        let raw = b"From: Alice Smith <alice@acme.com>\r\nSubject: Test\r\n\r\nBody";
        let email = parse(raw).unwrap();
        assert_eq!(email.sender, "alice@acme.com");
    }

    #[test]
    fn parse_missing_subject_defaults_to_empty() {
        let raw = b"From: hr@acme.com\r\n\r\nBody";
        let email = parse(raw).unwrap();
        assert_eq!(email.subject, "");
    }

    // ── attachment_score ─────────────────────────────────────────────────────

    #[test]
    fn score_csv_is_highest() {
        assert_eq!(attachment_score("roster.csv"), 100);
    }

    #[test]
    fn score_spreadsheets_are_high() {
        assert_eq!(attachment_score("roster.xlsx"), 90);
        assert_eq!(attachment_score("data.xls"), 90);
        assert_eq!(attachment_score("data.ods"), 90);
    }

    #[test]
    fn score_documents_are_medium() {
        assert_eq!(attachment_score("report.pdf"), 70);
        assert_eq!(attachment_score("report.docx"), 70);
    }

    #[test]
    fn score_images_are_lowest() {
        assert_eq!(attachment_score("logo.png"), 10);
        assert_eq!(attachment_score("banner.jpg"), 10);
        assert_eq!(attachment_score("photo.jpeg"), 10);
        assert_eq!(attachment_score("anim.gif"), 10);
    }

    #[test]
    fn score_unknown_extension_is_middle() {
        assert_eq!(attachment_score("data.bin"), 50);
    }

    #[test]
    fn score_is_case_insensitive() {
        assert_eq!(attachment_score("ROSTER.CSV"), 100);
        assert_eq!(attachment_score("Logo.PNG"), 10);
    }

    // ── find_best_attachment ─────────────────────────────────────────────────

    /// Build a minimal multipart/mixed MIME message with named attachments.
    fn make_multipart(attachments: &[(&str, &str, &[u8])]) -> Vec<u8> {
        let boundary = "TEST_BOUNDARY";
        let mut msg = format!(
            "From: hr@acme.com\r\nSubject: Test\r\nContent-Type: multipart/mixed; boundary=\"{boundary}\"\r\n\r\n"
        );
        for (filename, mime_type, body) in attachments {
            msg.push_str(&format!(
                "--{boundary}\r\nContent-Type: {mime_type}\r\nContent-Disposition: attachment; filename=\"{filename}\"\r\n\r\n"
            ));
            // Push body as ASCII (fine for test data)
            msg.push_str(&String::from_utf8_lossy(body));
            msg.push_str("\r\n");
        }
        msg.push_str(&format!("--{boundary}--\r\n"));
        msg.into_bytes()
    }

    #[test]
    fn csv_beats_image_when_both_attached() {
        let raw = make_multipart(&[
            ("logo.png", "image/png", b"\x89PNG"),
            ("roster.csv", "text/csv", b"name,age\nAlice,30"),
        ]);
        let email = parse(&raw).unwrap();
        let (name, _) = email.attachment.unwrap();
        assert_eq!(name, "roster.csv");
    }

    #[test]
    fn image_is_returned_when_only_attachment() {
        let raw = make_multipart(&[("logo.png", "image/png", b"\x89PNG")]);
        let email = parse(&raw).unwrap();
        let (name, _) = email.attachment.unwrap();
        assert_eq!(name, "logo.png");
    }

    #[test]
    fn xlsx_beats_pdf_and_image() {
        let raw = make_multipart(&[
            ("banner.jpg", "image/jpeg", b"\xFF\xD8"),
            ("report.pdf", "application/pdf", b"%PDF"),
            ("data.xlsx", "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet", b"PK"),
        ]);
        let email = parse(&raw).unwrap();
        let (name, _) = email.attachment.unwrap();
        assert_eq!(name, "data.xlsx");
    }

    #[test]
    fn no_attachments_returns_none() {
        let raw = b"From: hr@acme.com\r\nSubject: No attach\r\n\r\nJust text";
        let email = parse(raw).unwrap();
        assert!(email.attachment.is_none());
    }
}

