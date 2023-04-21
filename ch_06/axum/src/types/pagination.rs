use serde::{Deserialize, Serialize};

/// Pagination struct which is getting extract
/// from query params
#[derive(Debug, Deserialize, Serialize)]
pub struct Pagination {
    /// The index of the first item which has to be returned
    pub start: usize,
    /// The index of the last item which has to be returned
    pub end: usize,
}
