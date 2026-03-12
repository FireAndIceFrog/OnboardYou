use serde::Serialize;
use utoipa::ToSchema;

/// Paginated list response wrapper.
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct ListResponse<T: Serialize> {
    /// Number of items per page.
    pub count_per_page: i64,
    /// Current page number (1-based).
    pub current_page: i64,
    /// Last page number (1-based).
    pub last_page: i64,
    /// Items on this page.
    pub data: Vec<T>,
}

impl<T: Serialize> ListResponse<T> {
    /// Build a `ListResponse` from a pre-sliced page of data and totals.
    pub fn new(data: Vec<T>, total: i64, page: i64, per_page: i64) -> Self {
        let last_page = if total == 0 {
            1
        } else {
            (total + per_page - 1) / per_page
        };
        Self {
            count_per_page: per_page,
            current_page: page,
            last_page,
            data,
        }
    }

    /// Build from a full in-memory list, slicing to the requested page.
    pub fn from_vec(all: Vec<T>, page: i64, per_page: i64) -> Self {
        let total = all.len() as i64;
        let start = ((page - 1) * per_page) as usize;
        let data: Vec<T> = all.into_iter().skip(start).take(per_page as usize).collect();
        Self::new(data, total, page, per_page)
    }
}
