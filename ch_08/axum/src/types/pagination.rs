use serde::{Deserialize, Serialize};

/// Pagination struct which is getting extract
/// from query params
#[derive(Default, Debug, Deserialize, Serialize)]
pub struct Pagination {
    /// The index of the last item which has to be returned
    pub limit: Option<i32>,
    /// The index of the first item which has to be returned
    pub offset: i32,
}
