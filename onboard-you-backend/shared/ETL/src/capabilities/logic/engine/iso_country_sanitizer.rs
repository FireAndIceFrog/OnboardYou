//! ISO Country Sanitizer: Normalises country strings to ISO 3166-1 codes
//!
//! Accepts an input that may be an ISO alpha-2 code, alpha-3 code, or a
//! well-known country name (case-insensitive) and outputs a standardised
//! ISO 3166-1 alpha-2 or alpha-3 code.
//!
//! # Manifest JSON
//!
//! ```json
//! {
//!   "source_column": "country_raw",
//!   "output_column": "country_code",
//!   "output_format": "alpha2"
//! }
//! ```
//!
//! | Field           | Type   | Description                                      |
//! |-----------------|--------|--------------------------------------------------|
//! | `source_column` | string | Column containing the raw country value           |
//! | `output_column` | string | Column to write the normalised code into          |
//! | `output_format` | string | `"alpha2"` or `"alpha3"` — desired output format  |

use onboard_you_models::{CountryOutputFormat, IsoCountrySanitizerConfig};
use onboard_you_models::ColumnCalculator;
use onboard_you_models::{OnboardingAction, Result, RosterContext};
use polars::prelude::*;
use std::collections::HashMap;
use std::sync::OnceLock;

// ---------------------------------------------------------------------------
// Country lookup table
// ---------------------------------------------------------------------------

/// A single ISO 3166-1 entry.
struct CountryEntry {
    alpha2: &'static str,
    alpha3: &'static str,
    names: &'static [&'static str],
}

/// Master table of ISO 3166-1 countries.
///
/// Each entry carries alpha-2, alpha-3 and one or more **lowercase**
/// well-known names.  The first name is the canonical short name.
const COUNTRIES: &[CountryEntry] = &[
    CountryEntry {
        alpha2: "AF",
        alpha3: "AFG",
        names: &["afghanistan"],
    },
    CountryEntry {
        alpha2: "AL",
        alpha3: "ALB",
        names: &["albania"],
    },
    CountryEntry {
        alpha2: "DZ",
        alpha3: "DZA",
        names: &["algeria"],
    },
    CountryEntry {
        alpha2: "AS",
        alpha3: "ASM",
        names: &["american samoa"],
    },
    CountryEntry {
        alpha2: "AD",
        alpha3: "AND",
        names: &["andorra"],
    },
    CountryEntry {
        alpha2: "AO",
        alpha3: "AGO",
        names: &["angola"],
    },
    CountryEntry {
        alpha2: "AG",
        alpha3: "ATG",
        names: &["antigua and barbuda", "antigua"],
    },
    CountryEntry {
        alpha2: "AR",
        alpha3: "ARG",
        names: &["argentina"],
    },
    CountryEntry {
        alpha2: "AM",
        alpha3: "ARM",
        names: &["armenia"],
    },
    CountryEntry {
        alpha2: "AU",
        alpha3: "AUS",
        names: &["australia"],
    },
    CountryEntry {
        alpha2: "AT",
        alpha3: "AUT",
        names: &["austria"],
    },
    CountryEntry {
        alpha2: "AZ",
        alpha3: "AZE",
        names: &["azerbaijan"],
    },
    CountryEntry {
        alpha2: "BS",
        alpha3: "BHS",
        names: &["bahamas", "the bahamas"],
    },
    CountryEntry {
        alpha2: "BH",
        alpha3: "BHR",
        names: &["bahrain"],
    },
    CountryEntry {
        alpha2: "BD",
        alpha3: "BGD",
        names: &["bangladesh"],
    },
    CountryEntry {
        alpha2: "BB",
        alpha3: "BRB",
        names: &["barbados"],
    },
    CountryEntry {
        alpha2: "BY",
        alpha3: "BLR",
        names: &["belarus"],
    },
    CountryEntry {
        alpha2: "BE",
        alpha3: "BEL",
        names: &["belgium"],
    },
    CountryEntry {
        alpha2: "BZ",
        alpha3: "BLZ",
        names: &["belize"],
    },
    CountryEntry {
        alpha2: "BJ",
        alpha3: "BEN",
        names: &["benin"],
    },
    CountryEntry {
        alpha2: "BT",
        alpha3: "BTN",
        names: &["bhutan"],
    },
    CountryEntry {
        alpha2: "BO",
        alpha3: "BOL",
        names: &["bolivia"],
    },
    CountryEntry {
        alpha2: "BA",
        alpha3: "BIH",
        names: &["bosnia and herzegovina", "bosnia"],
    },
    CountryEntry {
        alpha2: "BW",
        alpha3: "BWA",
        names: &["botswana"],
    },
    CountryEntry {
        alpha2: "BR",
        alpha3: "BRA",
        names: &["brazil", "brasil"],
    },
    CountryEntry {
        alpha2: "BN",
        alpha3: "BRN",
        names: &["brunei", "brunei darussalam"],
    },
    CountryEntry {
        alpha2: "BG",
        alpha3: "BGR",
        names: &["bulgaria"],
    },
    CountryEntry {
        alpha2: "BF",
        alpha3: "BFA",
        names: &["burkina faso"],
    },
    CountryEntry {
        alpha2: "BI",
        alpha3: "BDI",
        names: &["burundi"],
    },
    CountryEntry {
        alpha2: "CV",
        alpha3: "CPV",
        names: &["cabo verde", "cape verde"],
    },
    CountryEntry {
        alpha2: "KH",
        alpha3: "KHM",
        names: &["cambodia"],
    },
    CountryEntry {
        alpha2: "CM",
        alpha3: "CMR",
        names: &["cameroon"],
    },
    CountryEntry {
        alpha2: "CA",
        alpha3: "CAN",
        names: &["canada"],
    },
    CountryEntry {
        alpha2: "CF",
        alpha3: "CAF",
        names: &["central african republic"],
    },
    CountryEntry {
        alpha2: "TD",
        alpha3: "TCD",
        names: &["chad"],
    },
    CountryEntry {
        alpha2: "CL",
        alpha3: "CHL",
        names: &["chile"],
    },
    CountryEntry {
        alpha2: "CN",
        alpha3: "CHN",
        names: &["china"],
    },
    CountryEntry {
        alpha2: "CO",
        alpha3: "COL",
        names: &["colombia"],
    },
    CountryEntry {
        alpha2: "KM",
        alpha3: "COM",
        names: &["comoros"],
    },
    CountryEntry {
        alpha2: "CG",
        alpha3: "COG",
        names: &["congo", "republic of the congo"],
    },
    CountryEntry {
        alpha2: "CD",
        alpha3: "COD",
        names: &["democratic republic of the congo", "dr congo", "drc"],
    },
    CountryEntry {
        alpha2: "CR",
        alpha3: "CRI",
        names: &["costa rica"],
    },
    CountryEntry {
        alpha2: "CI",
        alpha3: "CIV",
        names: &["cote d'ivoire", "ivory coast"],
    },
    CountryEntry {
        alpha2: "HR",
        alpha3: "HRV",
        names: &["croatia"],
    },
    CountryEntry {
        alpha2: "CU",
        alpha3: "CUB",
        names: &["cuba"],
    },
    CountryEntry {
        alpha2: "CY",
        alpha3: "CYP",
        names: &["cyprus"],
    },
    CountryEntry {
        alpha2: "CZ",
        alpha3: "CZE",
        names: &["czech republic", "czechia"],
    },
    CountryEntry {
        alpha2: "DK",
        alpha3: "DNK",
        names: &["denmark"],
    },
    CountryEntry {
        alpha2: "DJ",
        alpha3: "DJI",
        names: &["djibouti"],
    },
    CountryEntry {
        alpha2: "DM",
        alpha3: "DMA",
        names: &["dominica"],
    },
    CountryEntry {
        alpha2: "DO",
        alpha3: "DOM",
        names: &["dominican republic"],
    },
    CountryEntry {
        alpha2: "EC",
        alpha3: "ECU",
        names: &["ecuador"],
    },
    CountryEntry {
        alpha2: "EG",
        alpha3: "EGY",
        names: &["egypt"],
    },
    CountryEntry {
        alpha2: "SV",
        alpha3: "SLV",
        names: &["el salvador"],
    },
    CountryEntry {
        alpha2: "GQ",
        alpha3: "GNQ",
        names: &["equatorial guinea"],
    },
    CountryEntry {
        alpha2: "ER",
        alpha3: "ERI",
        names: &["eritrea"],
    },
    CountryEntry {
        alpha2: "EE",
        alpha3: "EST",
        names: &["estonia"],
    },
    CountryEntry {
        alpha2: "SZ",
        alpha3: "SWZ",
        names: &["eswatini", "swaziland"],
    },
    CountryEntry {
        alpha2: "ET",
        alpha3: "ETH",
        names: &["ethiopia"],
    },
    CountryEntry {
        alpha2: "FJ",
        alpha3: "FJI",
        names: &["fiji"],
    },
    CountryEntry {
        alpha2: "FI",
        alpha3: "FIN",
        names: &["finland"],
    },
    CountryEntry {
        alpha2: "FR",
        alpha3: "FRA",
        names: &["france"],
    },
    CountryEntry {
        alpha2: "GA",
        alpha3: "GAB",
        names: &["gabon"],
    },
    CountryEntry {
        alpha2: "GM",
        alpha3: "GMB",
        names: &["gambia", "the gambia"],
    },
    CountryEntry {
        alpha2: "GE",
        alpha3: "GEO",
        names: &["georgia"],
    },
    CountryEntry {
        alpha2: "DE",
        alpha3: "DEU",
        names: &["germany", "deutschland"],
    },
    CountryEntry {
        alpha2: "GH",
        alpha3: "GHA",
        names: &["ghana"],
    },
    CountryEntry {
        alpha2: "GR",
        alpha3: "GRC",
        names: &["greece"],
    },
    CountryEntry {
        alpha2: "GD",
        alpha3: "GRD",
        names: &["grenada"],
    },
    CountryEntry {
        alpha2: "GT",
        alpha3: "GTM",
        names: &["guatemala"],
    },
    CountryEntry {
        alpha2: "GN",
        alpha3: "GIN",
        names: &["guinea"],
    },
    CountryEntry {
        alpha2: "GW",
        alpha3: "GNB",
        names: &["guinea-bissau"],
    },
    CountryEntry {
        alpha2: "GY",
        alpha3: "GUY",
        names: &["guyana"],
    },
    CountryEntry {
        alpha2: "HT",
        alpha3: "HTI",
        names: &["haiti"],
    },
    CountryEntry {
        alpha2: "HN",
        alpha3: "HND",
        names: &["honduras"],
    },
    CountryEntry {
        alpha2: "HK",
        alpha3: "HKG",
        names: &["hong kong"],
    },
    CountryEntry {
        alpha2: "HU",
        alpha3: "HUN",
        names: &["hungary"],
    },
    CountryEntry {
        alpha2: "IS",
        alpha3: "ISL",
        names: &["iceland"],
    },
    CountryEntry {
        alpha2: "IN",
        alpha3: "IND",
        names: &["india"],
    },
    CountryEntry {
        alpha2: "ID",
        alpha3: "IDN",
        names: &["indonesia"],
    },
    CountryEntry {
        alpha2: "IR",
        alpha3: "IRN",
        names: &["iran"],
    },
    CountryEntry {
        alpha2: "IQ",
        alpha3: "IRQ",
        names: &["iraq"],
    },
    CountryEntry {
        alpha2: "IE",
        alpha3: "IRL",
        names: &["ireland"],
    },
    CountryEntry {
        alpha2: "IL",
        alpha3: "ISR",
        names: &["israel"],
    },
    CountryEntry {
        alpha2: "IT",
        alpha3: "ITA",
        names: &["italy", "italia"],
    },
    CountryEntry {
        alpha2: "JM",
        alpha3: "JAM",
        names: &["jamaica"],
    },
    CountryEntry {
        alpha2: "JP",
        alpha3: "JPN",
        names: &["japan"],
    },
    CountryEntry {
        alpha2: "JO",
        alpha3: "JOR",
        names: &["jordan"],
    },
    CountryEntry {
        alpha2: "KZ",
        alpha3: "KAZ",
        names: &["kazakhstan"],
    },
    CountryEntry {
        alpha2: "KE",
        alpha3: "KEN",
        names: &["kenya"],
    },
    CountryEntry {
        alpha2: "KI",
        alpha3: "KIR",
        names: &["kiribati"],
    },
    CountryEntry {
        alpha2: "KP",
        alpha3: "PRK",
        names: &["north korea", "dprk"],
    },
    CountryEntry {
        alpha2: "KR",
        alpha3: "KOR",
        names: &["south korea", "korea", "republic of korea"],
    },
    CountryEntry {
        alpha2: "KW",
        alpha3: "KWT",
        names: &["kuwait"],
    },
    CountryEntry {
        alpha2: "KG",
        alpha3: "KGZ",
        names: &["kyrgyzstan"],
    },
    CountryEntry {
        alpha2: "LA",
        alpha3: "LAO",
        names: &["laos", "lao"],
    },
    CountryEntry {
        alpha2: "LV",
        alpha3: "LVA",
        names: &["latvia"],
    },
    CountryEntry {
        alpha2: "LB",
        alpha3: "LBN",
        names: &["lebanon"],
    },
    CountryEntry {
        alpha2: "LS",
        alpha3: "LSO",
        names: &["lesotho"],
    },
    CountryEntry {
        alpha2: "LR",
        alpha3: "LBR",
        names: &["liberia"],
    },
    CountryEntry {
        alpha2: "LY",
        alpha3: "LBY",
        names: &["libya"],
    },
    CountryEntry {
        alpha2: "LI",
        alpha3: "LIE",
        names: &["liechtenstein"],
    },
    CountryEntry {
        alpha2: "LT",
        alpha3: "LTU",
        names: &["lithuania"],
    },
    CountryEntry {
        alpha2: "LU",
        alpha3: "LUX",
        names: &["luxembourg"],
    },
    CountryEntry {
        alpha2: "MO",
        alpha3: "MAC",
        names: &["macao", "macau"],
    },
    CountryEntry {
        alpha2: "MG",
        alpha3: "MDG",
        names: &["madagascar"],
    },
    CountryEntry {
        alpha2: "MW",
        alpha3: "MWI",
        names: &["malawi"],
    },
    CountryEntry {
        alpha2: "MY",
        alpha3: "MYS",
        names: &["malaysia"],
    },
    CountryEntry {
        alpha2: "MV",
        alpha3: "MDV",
        names: &["maldives"],
    },
    CountryEntry {
        alpha2: "ML",
        alpha3: "MLI",
        names: &["mali"],
    },
    CountryEntry {
        alpha2: "MT",
        alpha3: "MLT",
        names: &["malta"],
    },
    CountryEntry {
        alpha2: "MH",
        alpha3: "MHL",
        names: &["marshall islands"],
    },
    CountryEntry {
        alpha2: "MR",
        alpha3: "MRT",
        names: &["mauritania"],
    },
    CountryEntry {
        alpha2: "MU",
        alpha3: "MUS",
        names: &["mauritius"],
    },
    CountryEntry {
        alpha2: "MX",
        alpha3: "MEX",
        names: &["mexico"],
    },
    CountryEntry {
        alpha2: "FM",
        alpha3: "FSM",
        names: &["micronesia"],
    },
    CountryEntry {
        alpha2: "MD",
        alpha3: "MDA",
        names: &["moldova"],
    },
    CountryEntry {
        alpha2: "MC",
        alpha3: "MCO",
        names: &["monaco"],
    },
    CountryEntry {
        alpha2: "MN",
        alpha3: "MNG",
        names: &["mongolia"],
    },
    CountryEntry {
        alpha2: "ME",
        alpha3: "MNE",
        names: &["montenegro"],
    },
    CountryEntry {
        alpha2: "MA",
        alpha3: "MAR",
        names: &["morocco"],
    },
    CountryEntry {
        alpha2: "MZ",
        alpha3: "MOZ",
        names: &["mozambique"],
    },
    CountryEntry {
        alpha2: "MM",
        alpha3: "MMR",
        names: &["myanmar", "burma"],
    },
    CountryEntry {
        alpha2: "NA",
        alpha3: "NAM",
        names: &["namibia"],
    },
    CountryEntry {
        alpha2: "NR",
        alpha3: "NRU",
        names: &["nauru"],
    },
    CountryEntry {
        alpha2: "NP",
        alpha3: "NPL",
        names: &["nepal"],
    },
    CountryEntry {
        alpha2: "NL",
        alpha3: "NLD",
        names: &["netherlands", "holland", "the netherlands"],
    },
    CountryEntry {
        alpha2: "NZ",
        alpha3: "NZL",
        names: &["new zealand"],
    },
    CountryEntry {
        alpha2: "NI",
        alpha3: "NIC",
        names: &["nicaragua"],
    },
    CountryEntry {
        alpha2: "NE",
        alpha3: "NER",
        names: &["niger"],
    },
    CountryEntry {
        alpha2: "NG",
        alpha3: "NGA",
        names: &["nigeria"],
    },
    CountryEntry {
        alpha2: "MK",
        alpha3: "MKD",
        names: &["north macedonia", "macedonia"],
    },
    CountryEntry {
        alpha2: "NO",
        alpha3: "NOR",
        names: &["norway"],
    },
    CountryEntry {
        alpha2: "OM",
        alpha3: "OMN",
        names: &["oman"],
    },
    CountryEntry {
        alpha2: "PK",
        alpha3: "PAK",
        names: &["pakistan"],
    },
    CountryEntry {
        alpha2: "PW",
        alpha3: "PLW",
        names: &["palau"],
    },
    CountryEntry {
        alpha2: "PS",
        alpha3: "PSE",
        names: &["palestine"],
    },
    CountryEntry {
        alpha2: "PA",
        alpha3: "PAN",
        names: &["panama"],
    },
    CountryEntry {
        alpha2: "PG",
        alpha3: "PNG",
        names: &["papua new guinea"],
    },
    CountryEntry {
        alpha2: "PY",
        alpha3: "PRY",
        names: &["paraguay"],
    },
    CountryEntry {
        alpha2: "PE",
        alpha3: "PER",
        names: &["peru"],
    },
    CountryEntry {
        alpha2: "PH",
        alpha3: "PHL",
        names: &["philippines"],
    },
    CountryEntry {
        alpha2: "PL",
        alpha3: "POL",
        names: &["poland"],
    },
    CountryEntry {
        alpha2: "PT",
        alpha3: "PRT",
        names: &["portugal"],
    },
    CountryEntry {
        alpha2: "PR",
        alpha3: "PRI",
        names: &["puerto rico"],
    },
    CountryEntry {
        alpha2: "QA",
        alpha3: "QAT",
        names: &["qatar"],
    },
    CountryEntry {
        alpha2: "RO",
        alpha3: "ROU",
        names: &["romania"],
    },
    CountryEntry {
        alpha2: "RU",
        alpha3: "RUS",
        names: &["russia", "russian federation"],
    },
    CountryEntry {
        alpha2: "RW",
        alpha3: "RWA",
        names: &["rwanda"],
    },
    CountryEntry {
        alpha2: "KN",
        alpha3: "KNA",
        names: &["saint kitts and nevis"],
    },
    CountryEntry {
        alpha2: "LC",
        alpha3: "LCA",
        names: &["saint lucia"],
    },
    CountryEntry {
        alpha2: "VC",
        alpha3: "VCT",
        names: &["saint vincent and the grenadines", "saint vincent"],
    },
    CountryEntry {
        alpha2: "WS",
        alpha3: "WSM",
        names: &["samoa"],
    },
    CountryEntry {
        alpha2: "SM",
        alpha3: "SMR",
        names: &["san marino"],
    },
    CountryEntry {
        alpha2: "ST",
        alpha3: "STP",
        names: &["sao tome and principe"],
    },
    CountryEntry {
        alpha2: "SA",
        alpha3: "SAU",
        names: &["saudi arabia"],
    },
    CountryEntry {
        alpha2: "SN",
        alpha3: "SEN",
        names: &["senegal"],
    },
    CountryEntry {
        alpha2: "RS",
        alpha3: "SRB",
        names: &["serbia"],
    },
    CountryEntry {
        alpha2: "SC",
        alpha3: "SYC",
        names: &["seychelles"],
    },
    CountryEntry {
        alpha2: "SL",
        alpha3: "SLE",
        names: &["sierra leone"],
    },
    CountryEntry {
        alpha2: "SG",
        alpha3: "SGP",
        names: &["singapore"],
    },
    CountryEntry {
        alpha2: "SK",
        alpha3: "SVK",
        names: &["slovakia"],
    },
    CountryEntry {
        alpha2: "SI",
        alpha3: "SVN",
        names: &["slovenia"],
    },
    CountryEntry {
        alpha2: "SB",
        alpha3: "SLB",
        names: &["solomon islands"],
    },
    CountryEntry {
        alpha2: "SO",
        alpha3: "SOM",
        names: &["somalia"],
    },
    CountryEntry {
        alpha2: "ZA",
        alpha3: "ZAF",
        names: &["south africa"],
    },
    CountryEntry {
        alpha2: "SS",
        alpha3: "SSD",
        names: &["south sudan"],
    },
    CountryEntry {
        alpha2: "ES",
        alpha3: "ESP",
        names: &["spain", "espana"],
    },
    CountryEntry {
        alpha2: "LK",
        alpha3: "LKA",
        names: &["sri lanka"],
    },
    CountryEntry {
        alpha2: "SD",
        alpha3: "SDN",
        names: &["sudan"],
    },
    CountryEntry {
        alpha2: "SR",
        alpha3: "SUR",
        names: &["suriname"],
    },
    CountryEntry {
        alpha2: "SE",
        alpha3: "SWE",
        names: &["sweden"],
    },
    CountryEntry {
        alpha2: "CH",
        alpha3: "CHE",
        names: &["switzerland"],
    },
    CountryEntry {
        alpha2: "SY",
        alpha3: "SYR",
        names: &["syria", "syrian arab republic"],
    },
    CountryEntry {
        alpha2: "TW",
        alpha3: "TWN",
        names: &["taiwan"],
    },
    CountryEntry {
        alpha2: "TJ",
        alpha3: "TJK",
        names: &["tajikistan"],
    },
    CountryEntry {
        alpha2: "TZ",
        alpha3: "TZA",
        names: &["tanzania"],
    },
    CountryEntry {
        alpha2: "TH",
        alpha3: "THA",
        names: &["thailand"],
    },
    CountryEntry {
        alpha2: "TL",
        alpha3: "TLS",
        names: &["timor-leste", "east timor"],
    },
    CountryEntry {
        alpha2: "TG",
        alpha3: "TGO",
        names: &["togo"],
    },
    CountryEntry {
        alpha2: "TO",
        alpha3: "TON",
        names: &["tonga"],
    },
    CountryEntry {
        alpha2: "TT",
        alpha3: "TTO",
        names: &["trinidad and tobago", "trinidad"],
    },
    CountryEntry {
        alpha2: "TN",
        alpha3: "TUN",
        names: &["tunisia"],
    },
    CountryEntry {
        alpha2: "TR",
        alpha3: "TUR",
        names: &["turkey", "turkiye"],
    },
    CountryEntry {
        alpha2: "TM",
        alpha3: "TKM",
        names: &["turkmenistan"],
    },
    CountryEntry {
        alpha2: "TV",
        alpha3: "TUV",
        names: &["tuvalu"],
    },
    CountryEntry {
        alpha2: "UG",
        alpha3: "UGA",
        names: &["uganda"],
    },
    CountryEntry {
        alpha2: "UA",
        alpha3: "UKR",
        names: &["ukraine"],
    },
    CountryEntry {
        alpha2: "AE",
        alpha3: "ARE",
        names: &["united arab emirates", "uae"],
    },
    CountryEntry {
        alpha2: "GB",
        alpha3: "GBR",
        names: &[
            "united kingdom",
            "uk",
            "great britain",
            "england",
            "scotland",
            "wales",
        ],
    },
    CountryEntry {
        alpha2: "US",
        alpha3: "USA",
        names: &[
            "united states",
            "usa",
            "united states of america",
            "us",
            "america",
        ],
    },
    CountryEntry {
        alpha2: "UY",
        alpha3: "URY",
        names: &["uruguay"],
    },
    CountryEntry {
        alpha2: "UZ",
        alpha3: "UZB",
        names: &["uzbekistan"],
    },
    CountryEntry {
        alpha2: "VU",
        alpha3: "VUT",
        names: &["vanuatu"],
    },
    CountryEntry {
        alpha2: "VA",
        alpha3: "VAT",
        names: &["vatican", "holy see"],
    },
    CountryEntry {
        alpha2: "VE",
        alpha3: "VEN",
        names: &["venezuela"],
    },
    CountryEntry {
        alpha2: "VN",
        alpha3: "VNM",
        names: &["vietnam", "viet nam"],
    },
    CountryEntry {
        alpha2: "YE",
        alpha3: "YEM",
        names: &["yemen"],
    },
    CountryEntry {
        alpha2: "ZM",
        alpha3: "ZMB",
        names: &["zambia"],
    },
    CountryEntry {
        alpha2: "ZW",
        alpha3: "ZWE",
        names: &["zimbabwe"],
    },
    // Territories & dependencies commonly seen in HR data
    CountryEntry {
        alpha2: "AW",
        alpha3: "ABW",
        names: &["aruba"],
    },
    CountryEntry {
        alpha2: "BM",
        alpha3: "BMU",
        names: &["bermuda"],
    },
    CountryEntry {
        alpha2: "KY",
        alpha3: "CYM",
        names: &["cayman islands"],
    },
    CountryEntry {
        alpha2: "CW",
        alpha3: "CUW",
        names: &["curacao"],
    },
    CountryEntry {
        alpha2: "GI",
        alpha3: "GIB",
        names: &["gibraltar"],
    },
    CountryEntry {
        alpha2: "GG",
        alpha3: "GGY",
        names: &["guernsey"],
    },
    CountryEntry {
        alpha2: "IM",
        alpha3: "IMN",
        names: &["isle of man"],
    },
    CountryEntry {
        alpha2: "JE",
        alpha3: "JEY",
        names: &["jersey"],
    },
    CountryEntry {
        alpha2: "XK",
        alpha3: "XKX",
        names: &["kosovo"],
    },
    CountryEntry {
        alpha2: "LI",
        alpha3: "LIE",
        names: &["liechtenstein"],
    },
    CountryEntry {
        alpha2: "GU",
        alpha3: "GUM",
        names: &["guam"],
    },
    CountryEntry {
        alpha2: "VI",
        alpha3: "VIR",
        names: &["us virgin islands", "virgin islands"],
    },
];

/// Resolved pair for fast lookup.
#[derive(Clone)]
struct Resolved {
    alpha2: &'static str,
    alpha3: &'static str,
}

/// Build a case-insensitive lookup map.  Keyed by lowercase alpha-2,
/// alpha-3, and every name variant.
fn build_lookup() -> HashMap<String, Resolved> {
    let mut map = HashMap::with_capacity(COUNTRIES.len() * 4);
    for entry in COUNTRIES {
        let resolved = Resolved {
            alpha2: entry.alpha2,
            alpha3: entry.alpha3,
        };
        map.insert(entry.alpha2.to_ascii_lowercase(), resolved.clone());
        map.insert(entry.alpha3.to_ascii_lowercase(), resolved.clone());
        for name in entry.names {
            map.insert((*name).to_string(), resolved.clone());
        }
    }
    map
}

/// Global singleton — initialised on first access.
fn lookup() -> &'static HashMap<String, Resolved> {
    static INSTANCE: OnceLock<HashMap<String, Resolved>> = OnceLock::new();
    INSTANCE.get_or_init(build_lookup)
}

/// Resolve a raw country string to an `(alpha2, alpha3)` pair.
fn resolve_country(raw: &str) -> Option<(&'static str, &'static str)> {
    let key = raw.trim().to_ascii_lowercase();
    lookup().get(&key).map(|r| (r.alpha2, r.alpha3))
}

// ---------------------------------------------------------------------------
// Engine
// ---------------------------------------------------------------------------

/// Normalises country values to ISO 3166-1 alpha-2 or alpha-3 codes.
///
/// Unrecognised values are left as `null` in the output column so that
/// downstream steps can apply their own fallback logic.
#[derive(Debug, Clone)]
pub struct IsoCountrySanitizer {
    config: IsoCountrySanitizerConfig,
}

impl IsoCountrySanitizer {
    pub fn new(config: IsoCountrySanitizerConfig) -> Self {
        Self { config }
    }

    /// Construct from a deserialised config.
    pub fn from_action_config(config: &IsoCountrySanitizerConfig) -> Result<Self> {
        config.validate()?;
        Ok(Self {
            config: config.clone(),
        })
    }
}

impl ColumnCalculator for IsoCountrySanitizer {
    fn calculate_columns(&self, mut context: RosterContext) -> Result<RosterContext> {
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());
        // Add the output column (schema-only — content does not matter for
        // column calculation).
        context.data =
            lf.with_column(col(&self.config.source_column).alias(&self.config.output_column));
        context.set_field_source(
            self.config.output_column.clone(),
            "iso_country_sanitizer".into(),
        );
        Ok(context)
    }
}

impl OnboardingAction for IsoCountrySanitizer {
    fn id(&self) -> &str {
        "iso_country_sanitizer"
    }

    fn execute(&self, mut context: RosterContext) -> Result<RosterContext> {
        tracing::info!(
            source = %self.config.source_column,
            output = %self.config.output_column,
            format = ?self.config.output_format,
            "IsoCountrySanitizer: normalising country column"
        );

        let fmt = self.config.output_format;
        let lf = std::mem::replace(&mut context.data, LazyFrame::default());

        context.data = lf.with_column(
            col(&self.config.source_column)
                .map(
                    move |column| {
                        let ca = column.str().map_err(|e| {
                            polars::error::PolarsError::ComputeError(
                                format!("iso_country_sanitizer: column is not string: {e}").into(),
                            )
                        })?;

                        let mut unrecognised: HashMap<String, usize> = HashMap::new();

                        let result: StringChunked = ca
                            .into_iter()
                            .map(|opt: Option<&str>| {
                                opt.and_then(|raw| match resolve_country(raw) {
                                    Some((a2, a3)) => Some(match fmt {
                                        CountryOutputFormat::Alpha2 => a2,
                                        CountryOutputFormat::Alpha3 => a3,
                                    }),
                                    None => {
                                        let trimmed = raw.trim();
                                        if !trimmed.is_empty() {
                                            *unrecognised
                                                .entry(trimmed.to_string())
                                                .or_insert(0) += 1;
                                        }
                                        None
                                    }
                                })
                            })
                            .collect();

                        if !unrecognised.is_empty() {
                            for (value, count) in &unrecognised {
                                tracing::warn!(
                                    raw_value = %value,
                                    occurrences = count,
                                    "IsoCountrySanitizer: unrecognised country value — \
                                     consider adding to the country register if common"
                                );
                            }
                            tracing::warn!(
                                distinct_unrecognised = unrecognised.len(),
                                total_unrecognised = unrecognised.values().sum::<usize>(),
                                "IsoCountrySanitizer: summary of unrecognised country values"
                            );
                        }

                        Ok(result.into_column())
                    },
                    |_: &Schema, _: &Field| Ok(Field::new("".into(), DataType::String)),
                )
                .alias(&self.config.output_column),
        );

        context.set_field_source(
            self.config.output_column.clone(),
            "iso_country_sanitizer".into(),
        );
        context.mark_field_modified(
            self.config.output_column.clone(),
            "iso_country_sanitizer".into(),
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

    fn sample_df() -> DataFrame {
        df! {
            "employee_id" => &["E001", "E002", "E003", "E004", "E005"],
            "country_raw" => &["US", "GBR", "Germany", "france", "UNKNOWN"],
        }
        .unwrap()
    }

    #[test]
    fn test_id() {
        let config = IsoCountrySanitizerConfig {
            source_column: "c".into(),
            output_column: "o".into(),
            output_format: CountryOutputFormat::Alpha2,
        };
        let act = IsoCountrySanitizer::new(config);
        assert_eq!(act.id(), "iso_country_sanitizer");
    }

    #[test]
    fn test_alpha2_output() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "country_raw",
            "output_column": "country_code",
            "output_format": "alpha2"
        }))
        .unwrap();
        let action = IsoCountrySanitizer::from_action_config(&cfg).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let out = df.column("country_code").unwrap().str().unwrap();
        assert_eq!(out.get(0).unwrap(), "US"); // alpha-2 passthrough
        assert_eq!(out.get(1).unwrap(), "GB"); // alpha-3 → alpha-2
        assert_eq!(out.get(2).unwrap(), "DE"); // name → alpha-2
        assert_eq!(out.get(3).unwrap(), "FR"); // lowercase name → alpha-2
        assert!(out.get(4).is_none()); // unrecognised → null
    }

    #[test]
    fn test_alpha3_output() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "country_raw",
            "output_column": "country_iso3",
            "output_format": "alpha3"
        }))
        .unwrap();
        let action = IsoCountrySanitizer::from_action_config(&cfg).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let out = df.column("country_iso3").unwrap().str().unwrap();
        assert_eq!(out.get(0).unwrap(), "USA"); // alpha-2 → alpha-3
        assert_eq!(out.get(1).unwrap(), "GBR"); // alpha-3 passthrough
        assert_eq!(out.get(2).unwrap(), "DEU"); // name → alpha-3
        assert_eq!(out.get(3).unwrap(), "FRA"); // lowercase name → alpha-3
        assert!(out.get(4).is_none()); // unrecognised → null
    }

    #[test]
    fn test_in_place_overwrite() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "country_raw",
            "output_column": "country_raw",
            "output_format": "alpha2"
        }))
        .unwrap();
        let action = IsoCountrySanitizer::from_action_config(&cfg).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let out = df.column("country_raw").unwrap().str().unwrap();
        assert_eq!(out.get(0).unwrap(), "US");
        assert_eq!(out.get(2).unwrap(), "DE");
    }

    #[test]
    fn test_well_known_aliases() {
        let df = df! {
            "c" => &["UK", "America", "Holland", "Brasil"],
        }
        .unwrap();

        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "c",
            "output_column": "iso",
            "output_format": "alpha2"
        }))
        .unwrap();
        let action = IsoCountrySanitizer::from_action_config(&cfg).expect("valid config");
        let ctx = RosterContext::new(df.lazy());
        let result = action.execute(ctx).expect("execute");
        let df = result.data.collect().expect("collect");

        let out = df.column("iso").unwrap().str().unwrap();
        assert_eq!(out.get(0).unwrap(), "GB");
        assert_eq!(out.get(1).unwrap(), "US");
        assert_eq!(out.get(2).unwrap(), "NL");
        assert_eq!(out.get(3).unwrap(), "BR");
    }

    #[test]
    fn test_empty_source_column_rejected() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "",
            "output_column": "out",
            "output_format": "alpha2"
        }))
        .unwrap();
        assert!(IsoCountrySanitizer::from_action_config(&cfg).is_err());
    }

    #[test]
    fn test_empty_output_column_rejected() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "in",
            "output_column": "",
            "output_format": "alpha2"
        }))
        .unwrap();
        assert!(IsoCountrySanitizer::from_action_config(&cfg).is_err());
    }

    #[test]
    fn test_invalid_format_rejected() {
        // Invalid enum value is now caught at deserialization
        assert!(
            serde_json::from_value::<IsoCountrySanitizerConfig>(serde_json::json!({
                "source_column": "in",
                "output_column": "out",
                "output_format": "alpha4"
            }))
            .is_err()
        );
    }

    #[test]
    fn test_missing_column_errors_on_collect() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "nonexistent",
            "output_column": "out",
            "output_format": "alpha2"
        }))
        .unwrap();
        let action = IsoCountrySanitizer::from_action_config(&cfg).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        // With lazy execution the error is deferred until the LazyFrame is collected.
        let result = action.execute(ctx).expect("lazy execute succeeds");
        assert!(result.data.collect().is_err());
    }

    #[test]
    fn test_field_metadata_provenance() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "country_raw",
            "output_column": "country_code",
            "output_format": "alpha2"
        }))
        .unwrap();
        let action = IsoCountrySanitizer::from_action_config(&cfg).expect("valid config");
        let ctx = RosterContext::new(sample_df().lazy());
        let result = action.execute(ctx).expect("execute");
        let meta = result
            .field_metadata
            .get("country_code")
            .expect("metadata should exist");
        assert_eq!(meta.source, "iso_country_sanitizer");
    }

    #[test]
    fn test_resolve_country_helper() {
        // alpha-2
        assert_eq!(resolve_country("US"), Some(("US", "USA")));
        assert_eq!(resolve_country("us"), Some(("US", "USA")));
        // alpha-3
        assert_eq!(resolve_country("GBR"), Some(("GB", "GBR")));
        assert_eq!(resolve_country("gbr"), Some(("GB", "GBR")));
        // name
        assert_eq!(resolve_country("Germany"), Some(("DE", "DEU")));
        assert_eq!(resolve_country("  france  "), Some(("FR", "FRA")));
        // unknown
        assert_eq!(resolve_country("Atlantis"), None);
    }

    #[test]
    fn test_from_action_config_deserialization() {
        let cfg: IsoCountrySanitizerConfig = serde_json::from_value(serde_json::json!({
            "source_column": "src",
            "output_column": "dst",
            "output_format": "alpha3"
        }))
        .unwrap();
        let action = IsoCountrySanitizer::from_action_config(&cfg).expect("valid");
        assert_eq!(action.config.source_column, "src");
        assert_eq!(action.config.output_column, "dst");
        assert_eq!(action.config.output_format, CountryOutputFormat::Alpha3);
    }
}
