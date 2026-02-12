//! Cellphone Sanitizer: Normalises local phone numbers to international format
//!
//! Uses one or more country columns (in priority order) to resolve the
//! international dialling code, then prefixes the number with `+<code>`.
//!
//! Numbers that already carry an international prefix (`+…`) are left
//! untouched — the sanitizer never overwrites an existing code.
//!
//! # Manifest JSON
//!
//! ```json
//! {
//!   "phone_column": "mobile_phone",
//!   "country_columns": ["work_country", "home_country"],
//!   "output_column": "mobile_phone_intl"
//! }
//! ```
//!
//! | Field              | Type       | Description                                             |
//! |--------------------|------------|---------------------------------------------------------|
//! | `phone_column`     | string     | Column containing the raw phone number                  |
//! | `country_columns`  | `[string]` | Priority-ordered list of columns holding ISO 2/3 codes  |
//! | `output_column`    | string     | Column to write the internationalised number into       |

use crate::capabilities::logic::models::CellphoneSanitizerConfig;
use crate::capabilities::logic::traits::ColumnCalculator;
use crate::domain::{OnboardingAction, Result, RosterContext};
use polars::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// ISO alpha-2 → international dialling code
// ---------------------------------------------------------------------------

/// Build a map from **lowercase** ISO alpha-2 *and* alpha-3 codes to the
/// E.164 country calling code (without the leading `+`).
fn build_dial_code_map() -> HashMap<&'static str, &'static str> {
    // Entries: (alpha2, alpha3, dial_code)
    let entries: &[(&str, &str, &str)] = &[
        ("af", "afg", "93"),
        ("al", "alb", "355"),
        ("dz", "dza", "213"),
        ("as", "asm", "1684"),
        ("ad", "and", "376"),
        ("ao", "ago", "244"),
        ("ag", "atg", "1268"),
        ("ar", "arg", "54"),
        ("am", "arm", "374"),
        ("au", "aus", "61"),
        ("at", "aut", "43"),
        ("az", "aze", "994"),
        ("bs", "bhs", "1242"),
        ("bh", "bhr", "973"),
        ("bd", "bgd", "880"),
        ("bb", "brb", "1246"),
        ("by", "blr", "375"),
        ("be", "bel", "32"),
        ("bz", "blz", "501"),
        ("bj", "ben", "229"),
        ("bt", "btn", "975"),
        ("bo", "bol", "591"),
        ("ba", "bih", "387"),
        ("bw", "bwa", "267"),
        ("br", "bra", "55"),
        ("bn", "brn", "673"),
        ("bg", "bgr", "359"),
        ("bf", "bfa", "226"),
        ("bi", "bdi", "257"),
        ("cv", "cpv", "238"),
        ("kh", "khm", "855"),
        ("cm", "cmr", "237"),
        ("ca", "can", "1"),
        ("cf", "caf", "236"),
        ("td", "tcd", "235"),
        ("cl", "chl", "56"),
        ("cn", "chn", "86"),
        ("co", "col", "57"),
        ("km", "com", "269"),
        ("cg", "cog", "242"),
        ("cd", "cod", "243"),
        ("cr", "cri", "506"),
        ("ci", "civ", "225"),
        ("hr", "hrv", "385"),
        ("cu", "cub", "53"),
        ("cy", "cyp", "357"),
        ("cz", "cze", "420"),
        ("dk", "dnk", "45"),
        ("dj", "dji", "253"),
        ("dm", "dma", "1767"),
        ("do", "dom", "1809"),
        ("ec", "ecu", "593"),
        ("eg", "egy", "20"),
        ("sv", "slv", "503"),
        ("gq", "gnq", "240"),
        ("er", "eri", "291"),
        ("ee", "est", "372"),
        ("sz", "swz", "268"),
        ("et", "eth", "251"),
        ("fj", "fji", "679"),
        ("fi", "fin", "358"),
        ("fr", "fra", "33"),
        ("ga", "gab", "241"),
        ("gm", "gmb", "220"),
        ("ge", "geo", "995"),
        ("de", "deu", "49"),
        ("gh", "gha", "233"),
        ("gr", "grc", "30"),
        ("gd", "grd", "1473"),
        ("gt", "gtm", "502"),
        ("gn", "gin", "224"),
        ("gw", "gnb", "245"),
        ("gy", "guy", "592"),
        ("ht", "hti", "509"),
        ("hn", "hnd", "504"),
        ("hk", "hkg", "852"),
        ("hu", "hun", "36"),
        ("is", "isl", "354"),
        ("in", "ind", "91"),
        ("id", "idn", "62"),
        ("ir", "irn", "98"),
        ("iq", "irq", "964"),
        ("ie", "irl", "353"),
        ("il", "isr", "972"),
        ("it", "ita", "39"),
        ("jm", "jam", "1876"),
        ("jp", "jpn", "81"),
        ("jo", "jor", "962"),
        ("kz", "kaz", "7"),
        ("ke", "ken", "254"),
        ("ki", "kir", "686"),
        ("kp", "prk", "850"),
        ("kr", "kor", "82"),
        ("kw", "kwt", "965"),
        ("kg", "kgz", "996"),
        ("la", "lao", "856"),
        ("lv", "lva", "371"),
        ("lb", "lbn", "961"),
        ("ls", "lso", "266"),
        ("lr", "lbr", "231"),
        ("ly", "lby", "218"),
        ("li", "lie", "423"),
        ("lt", "ltu", "370"),
        ("lu", "lux", "352"),
        ("mo", "mac", "853"),
        ("mg", "mdg", "261"),
        ("mw", "mwi", "265"),
        ("my", "mys", "60"),
        ("mv", "mdv", "960"),
        ("ml", "mli", "223"),
        ("mt", "mlt", "356"),
        ("mh", "mhl", "692"),
        ("mr", "mrt", "222"),
        ("mu", "mus", "230"),
        ("mx", "mex", "52"),
        ("fm", "fsm", "691"),
        ("md", "mda", "373"),
        ("mc", "mco", "377"),
        ("mn", "mng", "976"),
        ("me", "mne", "382"),
        ("ma", "mar", "212"),
        ("mz", "moz", "258"),
        ("mm", "mmr", "95"),
        ("na", "nam", "264"),
        ("nr", "nru", "674"),
        ("np", "npl", "977"),
        ("nl", "nld", "31"),
        ("nz", "nzl", "64"),
        ("ni", "nic", "505"),
        ("ne", "ner", "227"),
        ("ng", "nga", "234"),
        ("mk", "mkd", "389"),
        ("no", "nor", "47"),
        ("om", "omn", "968"),
        ("pk", "pak", "92"),
        ("pw", "plw", "680"),
        ("ps", "pse", "970"),
        ("pa", "pan", "507"),
        ("pg", "png", "675"),
        ("py", "pry", "595"),
        ("pe", "per", "51"),
        ("ph", "phl", "63"),
        ("pl", "pol", "48"),
        ("pt", "prt", "351"),
        ("pr", "pri", "1787"),
        ("qa", "qat", "974"),
        ("ro", "rou", "40"),
        ("ru", "rus", "7"),
        ("rw", "rwa", "250"),
        ("kn", "kna", "1869"),
        ("lc", "lca", "1758"),
        ("vc", "vct", "1784"),
        ("ws", "wsm", "685"),
        ("sm", "smr", "378"),
        ("st", "stp", "239"),
        ("sa", "sau", "966"),
        ("sn", "sen", "221"),
        ("rs", "srb", "381"),
        ("sc", "syc", "248"),
        ("sl", "sle", "232"),
        ("sg", "sgp", "65"),
        ("sk", "svk", "421"),
        ("si", "svn", "386"),
        ("sb", "slb", "677"),
        ("so", "som", "252"),
        ("za", "zaf", "27"),
        ("ss", "ssd", "211"),
        ("es", "esp", "34"),
        ("lk", "lka", "94"),
        ("sd", "sdn", "249"),
        ("sr", "sur", "597"),
        ("se", "swe", "46"),
        ("ch", "che", "41"),
        ("sy", "syr", "963"),
        ("tw", "twn", "886"),
        ("tj", "tjk", "992"),
        ("tz", "tza", "255"),
        ("th", "tha", "66"),
        ("tl", "tls", "670"),
        ("tg", "tgo", "228"),
        ("to", "ton", "676"),
        ("tt", "tto", "1868"),
        ("tn", "tun", "216"),
        ("tr", "tur", "90"),
        ("tm", "tkm", "993"),
        ("tv", "tuv", "688"),
        ("ug", "uga", "256"),
        ("ua", "ukr", "380"),
        ("ae", "are", "971"),
        ("gb", "gbr", "44"),
        ("us", "usa", "1"),
        ("uy", "ury", "598"),
        ("uz", "uzb", "998"),
        ("vu", "vut", "678"),
        ("va", "vat", "379"),
        ("ve", "ven", "58"),
        ("vn", "vnm", "84"),
        ("ye", "yem", "967"),
        ("zm", "zmb", "260"),
        ("zw", "zwe", "263"),
        // Territories & dependencies commonly seen in HR data
        ("aw", "abw", "297"),
        ("bm", "bmu", "1441"),
        ("ky", "cym", "1345"),
        ("cw", "cuw", "599"),
        ("gi", "gib", "350"),
        ("gg", "ggy", "44"),
        ("im", "imn", "44"),
        ("je", "jey", "44"),
        ("xk", "xkx", "383"),
        ("gu", "gum", "1671"),
        ("vi", "vir", "1340"),
    ];

    let mut map = HashMap::with_capacity(entries.len() * 2);
    for &(a2, a3, dial) in entries {
        map.insert(a2, dial);
        map.insert(a3, dial);
    }
    map
}

/// Global singleton — initialised on first access.
fn dial_code_map() -> &'static HashMap<&'static str, &'static str> {
    static INSTANCE: OnceLock<HashMap<&'static str, &'static str>> = OnceLock::new();
    INSTANCE.get_or_init(build_dial_code_map)
}

/// Resolve an ISO alpha-2 or alpha-3 code to its E.164 calling code.
///
/// Returns `None` for unrecognised codes.
fn resolve_dial_code(iso_code: &str) -> Option<&'static str> {
    let key = iso_code.trim().to_ascii_lowercase();
    dial_code_map().get(key.as_str()).copied()
}

// ---------------------------------------------------------------------------
// Core sanitisation logic (pure function — easily testable)
// ---------------------------------------------------------------------------

/// Strip all non-digit, non-`+` characters from a phone number.
fn strip_noise(raw: &str) -> String {
    raw.chars().filter(|c| c.is_ascii_digit() || *c == '+').collect()
}

/// Sanitise a single phone number given an already-resolved calling code.
///
/// Returns `None` if the number is empty after stripping noise.
///
/// Rules:
/// 1. If the stripped number starts with `+` → already international → return as-is.
/// 2. Otherwise strip the leading `0` (local trunk prefix) if present,
///    then prepend `+<calling_code> `.
fn sanitise_number(raw_phone: &str, calling_code: Option<&str>) -> Option<String> {
    let cleaned = strip_noise(raw_phone);
    if cleaned.is_empty() {
        return None;
    }

    // Already international — keep it.
    if cleaned.starts_with('+') {
        return Some(cleaned);
    }

    match calling_code {
        Some(cc) => {
            // Strip leading local trunk prefix '0'
            let local = cleaned.strip_prefix('0').unwrap_or(&cleaned);
            Some(format!("+{cc} {local}"))
        }
        // No country resolved — return the cleaned digits unchanged.
        None => Some(cleaned),
    }
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Normalises local phone numbers to international E.164-ish format using
/// country information from the same row.
#[derive(Debug, Clone)]
pub struct CellphoneSanitizer {
    config: CellphoneSanitizerConfig,
}

impl CellphoneSanitizer {
    pub fn new(config: CellphoneSanitizerConfig) -> Self {
        Self { config }
    }

    /// Deserialise and construct from manifest JSON.
    pub fn from_action_config(value: &serde_json::Value) -> Result<Self> {
        let config: CellphoneSanitizerConfig = serde_json::from_value(value.clone())?;
        config.validate()?;
        Ok(Self::new(config))
    }
}

impl ColumnCalculator for CellphoneSanitizer {
    fn calculate_columns(&self, mut context: RosterContext) -> Result<RosterContext> {
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        context.data = lf.with_column(
            col(&self.config.phone_column).alias(&self.config.output_column),
        );
        context.set_field_source(
            self.config.output_column.clone(),
            "cellphone_sanitizer".into(),
        );
        Ok(context)
    }
}

impl OnboardingAction for CellphoneSanitizer {
    fn id(&self) -> &str {
        "cellphone_sanitizer"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            phone = %self.config.phone_column,
            countries = ?self.config.country_columns,
            output = %self.config.output_column,
            "CellphoneSanitizer: internationalising phone numbers"
        );

        let lf = std::mem::replace(&mut context.data, LazyFrame::default());

        // Bundle phone + country columns into a struct so we can access
        // multiple columns inside a single `.map()` closure while staying lazy.
        let all_cols: Vec<Expr> = std::iter::once(col(&self.config.phone_column))
            .chain(self.config.country_columns.iter().map(|c| col(c)))
            .collect();

        let phone_col_name = self.config.phone_column.clone();
        let country_col_names = self.config.country_columns.clone();

        context.data = lf.with_column(
            as_struct(all_cols)
                .map(
                    move |s| {
                        let ca = s.struct_().map_err(|e| {
                            polars::error::PolarsError::ComputeError(
                                format!("cellphone_sanitizer: expected struct column: {e}")
                                    .into(),
                            )
                        })?;

                        let phone_field = ca.field_by_name(&phone_col_name).map_err(|e| {
                            polars::error::PolarsError::ComputeError(
                                format!("cellphone_sanitizer: phone field not found: {e}")
                                    .into(),
                            )
                        })?;
                        let phone_ca = phone_field.str().map_err(|e| {
                            polars::error::PolarsError::ComputeError(
                                format!("cellphone_sanitizer: phone field is not string: {e}")
                                    .into(),
                            )
                        })?;

                        // Extract country string columns, preserving priority order.
                        let country_fields: Vec<Series> = country_col_names
                            .iter()
                            .filter_map(|name| ca.field_by_name(name).ok())
                            .collect();
                        let country_cas: Vec<&StringChunked> = country_fields
                            .iter()
                            .filter_map(|f| f.str().ok())
                            .collect();

                        let result: StringChunked = phone_ca
                            .into_iter()
                            .enumerate()
                            .map(|(idx, opt_phone)| {
                                opt_phone.and_then(|phone| {
                                    let calling_code = country_cas.iter().find_map(|cc| {
                                        cc.get(idx).and_then(resolve_dial_code)
                                    });
                                    sanitise_number(phone, calling_code)
                                })
                            })
                            .collect();

                        Ok(result.into_column())
                    },
                    |_: &Schema, _: &Field| Ok(Field::new("".into(), DataType::String)),
                )
                .alias(&self.config.output_column),
        );

        context.set_field_source(
            self.config.output_column.clone(),
            "cellphone_sanitizer".into(),
        );
        context.mark_field_modified(
            self.config.output_column.clone(),
            "cellphone_sanitizer".into(),
        );

        Ok(context)
    }
}

// ---------------------------------------------------------------------------
// Unit tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // ---- helper -----------------------------------------------------------

    fn sample_df() -> DataFrame {
        df! {
            "employee_id"  => &["E001", "E002", "E003", "E004", "E005", "E006"],
            "mobile_phone" => &[
                "02102201202",      // NZ local
                "+44 7911 123456",  // already international (UK)
                "0412345678",       // AU local
                "2025551234",       // US (no leading 0)
                "(021) 555-1234",   // NZ with formatting noise
                "+61 412 000 111",  // already international (AU)
            ],
            "work_country"  => &["NZL", "GBR", "AU", "US", "NZ", "AUS"],
            "home_country"  => &["NZ",  "GB",  "AU", "US", "NZ", "AU"],
        }
        .unwrap()
    }

    // ---- pure function tests ----------------------------------------------

    #[test]
    fn test_strip_noise() {
        assert_eq!(strip_noise("(021) 555-1234"), "0215551234");
        assert_eq!(strip_noise("+64 21 555 1234"), "+64215551234");
        assert_eq!(strip_noise("  0412 345 678  "), "0412345678");
    }

    #[test]
    fn test_sanitise_already_international() {
        let result = sanitise_number("+44 7911 123456", Some("64"));
        assert_eq!(result, Some("+447911123456".into()));
    }

    #[test]
    fn test_sanitise_local_nz() {
        let result = sanitise_number("02102201202", Some("64"));
        assert_eq!(result, Some("+64 2102201202".into()));
    }

    #[test]
    fn test_sanitise_local_au() {
        let result = sanitise_number("0412345678", Some("61"));
        assert_eq!(result, Some("+61 412345678".into()));
    }

    #[test]
    fn test_sanitise_no_leading_zero() {
        let result = sanitise_number("2025551234", Some("1"));
        assert_eq!(result, Some("+1 2025551234".into()));
    }

    #[test]
    fn test_sanitise_empty_returns_none() {
        assert_eq!(sanitise_number("", Some("64")), None);
        assert_eq!(sanitise_number("   ", Some("64")), None);
    }

    #[test]
    fn test_sanitise_no_country() {
        // No calling code resolved — just return cleaned digits.
        let result = sanitise_number("0215551234", None);
        assert_eq!(result, Some("0215551234".into()));
    }

    // ---- resolve_dial_code ------------------------------------------------

    #[test]
    fn test_resolve_dial_code_alpha2() {
        assert_eq!(resolve_dial_code("NZ"), Some("64"));
        assert_eq!(resolve_dial_code("nz"), Some("64"));
        assert_eq!(resolve_dial_code("US"), Some("1"));
        assert_eq!(resolve_dial_code("GB"), Some("44"));
    }

    #[test]
    fn test_resolve_dial_code_alpha3() {
        assert_eq!(resolve_dial_code("NZL"), Some("64"));
        assert_eq!(resolve_dial_code("nzl"), Some("64"));
        assert_eq!(resolve_dial_code("USA"), Some("1"));
        assert_eq!(resolve_dial_code("GBR"), Some("44"));
    }

    #[test]
    fn test_resolve_dial_code_unknown() {
        assert_eq!(resolve_dial_code("ZZZ"), None);
        assert_eq!(resolve_dial_code(""), None);
    }

    // ---- engine / integration tests ---------------------------------------

    #[test]
    fn test_id() {
        let config = CellphoneSanitizerConfig {
            phone_column: "p".into(),
            country_columns: vec!["c".into()],
            output_column: "o".into(),
        };
        assert_eq!(CellphoneSanitizer::new(config).id(), "cellphone_sanitizer");
    }

    #[test]
    fn test_full_execute() {
        let json = serde_json::json!({
            "phone_column": "mobile_phone",
            "country_columns": ["work_country", "home_country"],
            "output_column": "phone_intl"
        });
        let action = CellphoneSanitizer::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let out = df.column("phone_intl").unwrap().str().unwrap();

        // E001: NZ local 02102201202 → +64 2102201202
        assert_eq!(out.get(0).unwrap(), "+64 2102201202");
        // E002: already +44 → kept as-is (noise stripped)
        assert_eq!(out.get(1).unwrap(), "+447911123456");
        // E003: AU local 0412345678 → +61 412345678
        assert_eq!(out.get(2).unwrap(), "+61 412345678");
        // E004: US no leading 0, 2025551234 → +1 2025551234
        assert_eq!(out.get(3).unwrap(), "+1 2025551234");
        // E005: NZ with noise (021) 555-1234 → +64 215551234
        assert_eq!(out.get(4).unwrap(), "+64 215551234");
        // E006: already +61 → kept as-is (noise stripped)
        assert_eq!(out.get(5).unwrap(), "+61412000111");
    }

    #[test]
    fn test_in_place_overwrite() {
        let json = serde_json::json!({
            "phone_column": "mobile_phone",
            "country_columns": ["work_country"],
            "output_column": "mobile_phone"
        });
        let action = CellphoneSanitizer::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let out = df.column("mobile_phone").unwrap().str().unwrap();
        assert_eq!(out.get(0).unwrap(), "+64 2102201202");
    }

    #[test]
    fn test_country_column_priority() {
        // First column has null, second has a valid code — second is used.
        let df = df! {
            "phone"    => &["0215551234"],
            "primary"  => &[None::<&str>],
            "fallback" => &["NZ"],
        }
        .unwrap();

        let json = serde_json::json!({
            "phone_column": "phone",
            "country_columns": ["primary", "fallback"],
            "output_column": "phone_intl"
        });
        let action = CellphoneSanitizer::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).expect("execute");
        let collected = result.data.collect().expect("collect");

        let out = collected.column("phone_intl").unwrap().str().unwrap();
        assert_eq!(out.get(0).unwrap(), "+64 215551234");
    }

    #[test]
    fn test_null_phone_stays_null() {
        let df = df! {
            "phone"   => &[None::<&str>],
            "country" => &["NZ"],
        }
        .unwrap();

        let json = serde_json::json!({
            "phone_column": "phone",
            "country_columns": ["country"],
            "output_column": "phone_intl"
        });
        let action = CellphoneSanitizer::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).expect("execute");
        let collected = result.data.collect().expect("collect");

        let out = collected.column("phone_intl").unwrap().str().unwrap();
        assert!(out.get(0).is_none());
    }

    // ---- validation tests -------------------------------------------------

    #[test]
    fn test_empty_phone_column_rejected() {
        let json = serde_json::json!({
            "phone_column": "",
            "country_columns": ["c"],
            "output_column": "out"
        });
        assert!(CellphoneSanitizer::from_action_config(&json).is_err());
    }

    #[test]
    fn test_empty_country_columns_rejected() {
        let json = serde_json::json!({
            "phone_column": "phone",
            "country_columns": [],
            "output_column": "out"
        });
        assert!(CellphoneSanitizer::from_action_config(&json).is_err());
    }

    #[test]
    fn test_empty_output_column_rejected() {
        let json = serde_json::json!({
            "phone_column": "phone",
            "country_columns": ["c"],
            "output_column": ""
        });
        assert!(CellphoneSanitizer::from_action_config(&json).is_err());
    }

    #[test]
    fn test_field_metadata_provenance() {
        let json = serde_json::json!({
            "phone_column": "mobile_phone",
            "country_columns": ["work_country"],
            "output_column": "phone_intl"
        });
        let action = CellphoneSanitizer::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let meta = result
            .field_metadata
            .get("phone_intl")
            .expect("metadata should exist");
        assert_eq!(meta.source, "cellphone_sanitizer");
    }

    #[test]
    fn test_from_action_config_deserialization() {
        let json = serde_json::json!({
            "phone_column": "ph",
            "country_columns": ["c1", "c2"],
            "output_column": "out"
        });
        let action = CellphoneSanitizer::from_action_config(&json).expect("valid");
        assert_eq!(action.config.phone_column, "ph");
        assert_eq!(action.config.country_columns, vec!["c1", "c2"]);
        assert_eq!(action.config.output_column, "out");
    }

    #[test]
    fn test_handle_diacritics_in_numbers() {
        // Ensure various formatting is handled: spaces, dashes, parens, dots
        let df = df! {
            "phone"   => &["021.555.1234", "+1-202-555-0173"],
            "country" => &["NZ", "US"],
        }
        .unwrap();

        let json = serde_json::json!({
            "phone_column": "phone",
            "country_columns": ["country"],
            "output_column": "phone_intl"
        });
        let action = CellphoneSanitizer::from_action_config(&json).expect("valid config");
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).expect("execute");
        let collected = result.data.collect().expect("collect");

        let out = collected.column("phone_intl").unwrap().str().unwrap();
        assert_eq!(out.get(0).unwrap(), "+64 215551234");
        assert_eq!(out.get(1).unwrap(), "+12025550173"); // already has +, kept as-is
    }
}
