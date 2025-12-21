//! GitHub-style pagination with Link headers.

use serde::{Deserialize, Serialize};

/// Default number of items per page.
pub const DEFAULT_PER_PAGE: u32 = 30;

/// Maximum number of items per page.
pub const MAX_PER_PAGE: u32 = 100;

/// Pagination parameters from query string.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PaginationParams {
    /// Page number (1-indexed).
    #[serde(default = "default_page")]
    pub page: u32,

    /// Items per page.
    #[serde(default = "default_per_page")]
    pub per_page: u32,
}

fn default_page() -> u32 {
    1
}

fn default_per_page() -> u32 {
    DEFAULT_PER_PAGE
}

impl PaginationParams {
    /// Create new pagination parameters.
    pub fn new(page: u32, per_page: u32) -> Self {
        Self {
            page: page.max(1),
            per_page: per_page.clamp(1, MAX_PER_PAGE),
        }
    }

    /// Get the offset for database queries.
    pub fn offset(&self) -> u32 {
        (self.page.saturating_sub(1)) * self.per_page
    }

    /// Get the limit for database queries.
    pub fn limit(&self) -> u32 {
        self.per_page
    }

    /// Normalize parameters to valid ranges.
    pub fn normalize(&mut self) {
        self.page = self.page.max(1);
        self.per_page = self.per_page.clamp(1, MAX_PER_PAGE);
    }
}

/// Pagination links for the Link header.
#[derive(Debug, Clone, Default)]
pub struct PaginationLinks {
    /// URL to the first page.
    pub first: Option<String>,
    /// URL to the previous page.
    pub prev: Option<String>,
    /// URL to the next page.
    pub next: Option<String>,
    /// URL to the last page.
    pub last: Option<String>,
}

impl PaginationLinks {
    /// Create pagination links from current state.
    pub fn new(base_url: &str, current_page: u32, per_page: u32, total_items: u32) -> Self {
        let total_pages = total_items.div_ceil(per_page);

        if total_pages <= 1 {
            return Self::default();
        }

        let make_url = |page: u32| {
            if base_url.contains('?') {
                format!("{}&page={}&per_page={}", base_url, page, per_page)
            } else {
                format!("{}?page={}&per_page={}", base_url, page, per_page)
            }
        };

        Self {
            first: if current_page > 1 {
                Some(make_url(1))
            } else {
                None
            },
            prev: if current_page > 1 {
                Some(make_url(current_page - 1))
            } else {
                None
            },
            next: if current_page < total_pages {
                Some(make_url(current_page + 1))
            } else {
                None
            },
            last: if current_page < total_pages {
                Some(make_url(total_pages))
            } else {
                None
            },
        }
    }

    /// Check if there are any links.
    pub fn is_empty(&self) -> bool {
        self.first.is_none() && self.prev.is_none() && self.next.is_none() && self.last.is_none()
    }

    /// Format as a Link header value.
    pub fn to_header_value(&self) -> Option<String> {
        if self.is_empty() {
            return None;
        }

        let mut parts = Vec::new();

        if let Some(ref url) = self.first {
            parts.push(format!("<{}>; rel=\"first\"", url));
        }
        if let Some(ref url) = self.prev {
            parts.push(format!("<{}>; rel=\"prev\"", url));
        }
        if let Some(ref url) = self.next {
            parts.push(format!("<{}>; rel=\"next\"", url));
        }
        if let Some(ref url) = self.last {
            parts.push(format!("<{}>; rel=\"last\"", url));
        }

        Some(parts.join(", "))
    }
}

/// Paginated response wrapper.
#[derive(Debug, Clone, Serialize)]
pub struct PaginatedResponse<T> {
    /// The data items.
    pub items: Vec<T>,
    /// Total number of items.
    pub total_count: u32,
    /// Current page number.
    pub page: u32,
    /// Items per page.
    pub per_page: u32,
    /// Total number of pages.
    pub total_pages: u32,
}

impl<T> PaginatedResponse<T> {
    /// Create a new paginated response.
    pub fn new(items: Vec<T>, total_count: u32, page: u32, per_page: u32) -> Self {
        let total_pages = total_count.div_ceil(per_page);
        Self {
            items,
            total_count,
            page,
            per_page,
            total_pages,
        }
    }

    /// Get pagination links for this response.
    pub fn links(&self, base_url: &str) -> PaginationLinks {
        PaginationLinks::new(base_url, self.page, self.per_page, self.total_count)
    }

    /// Check if there are more pages.
    pub fn has_next_page(&self) -> bool {
        self.page < self.total_pages
    }

    /// Check if there is a previous page.
    pub fn has_prev_page(&self) -> bool {
        self.page > 1
    }
}

/// Paginate a slice of items.
pub fn paginate<T: Clone>(items: &[T], params: &PaginationParams) -> PaginatedResponse<T> {
    let total_count = items.len() as u32;
    let start = params.offset() as usize;
    let end = (start + params.limit() as usize).min(items.len());

    let page_items = if start < items.len() {
        items[start..end].to_vec()
    } else {
        Vec::new()
    };

    PaginatedResponse::new(page_items, total_count, params.page, params.per_page)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pagination_params() {
        let params = PaginationParams::new(2, 10);
        assert_eq!(params.page, 2);
        assert_eq!(params.per_page, 10);
        assert_eq!(params.offset(), 10);
        assert_eq!(params.limit(), 10);
    }

    #[test]
    fn test_pagination_params_bounds() {
        let params = PaginationParams::new(0, 200);
        assert_eq!(params.page, 1);
        assert_eq!(params.per_page, 100);
    }

    #[test]
    fn test_pagination_links() {
        let links = PaginationLinks::new("https://api.example.com/items", 2, 10, 50);

        assert!(links.first.is_some());
        assert!(links.prev.is_some());
        assert!(links.next.is_some());
        assert!(links.last.is_some());

        assert_eq!(
            links.first.unwrap(),
            "https://api.example.com/items?page=1&per_page=10"
        );
        assert_eq!(
            links.prev.unwrap(),
            "https://api.example.com/items?page=1&per_page=10"
        );
        assert_eq!(
            links.next.unwrap(),
            "https://api.example.com/items?page=3&per_page=10"
        );
        assert_eq!(
            links.last.unwrap(),
            "https://api.example.com/items?page=5&per_page=10"
        );
    }

    #[test]
    fn test_pagination_links_first_page() {
        let links = PaginationLinks::new("https://api.example.com/items", 1, 10, 50);

        assert!(links.first.is_none());
        assert!(links.prev.is_none());
        assert!(links.next.is_some());
        assert!(links.last.is_some());
    }

    #[test]
    fn test_pagination_links_last_page() {
        let links = PaginationLinks::new("https://api.example.com/items", 5, 10, 50);

        assert!(links.first.is_some());
        assert!(links.prev.is_some());
        assert!(links.next.is_none());
        assert!(links.last.is_none());
    }

    #[test]
    fn test_pagination_links_single_page() {
        let links = PaginationLinks::new("https://api.example.com/items", 1, 10, 5);

        assert!(links.is_empty());
    }

    #[test]
    fn test_link_header_value() {
        let links = PaginationLinks::new("https://api.example.com/items", 2, 10, 30);
        let header = links.to_header_value();

        assert!(header.is_some());
        let value = header.unwrap();
        assert!(value.contains("rel=\"first\""));
        assert!(value.contains("rel=\"prev\""));
        assert!(value.contains("rel=\"next\""));
        assert!(value.contains("rel=\"last\""));
    }

    #[test]
    fn test_paginate() {
        let items: Vec<i32> = (1..=25).collect();
        let params = PaginationParams::new(2, 10);
        let response = paginate(&items, &params);

        assert_eq!(response.items, vec![11, 12, 13, 14, 15, 16, 17, 18, 19, 20]);
        assert_eq!(response.total_count, 25);
        assert_eq!(response.page, 2);
        assert_eq!(response.per_page, 10);
        assert_eq!(response.total_pages, 3);
        assert!(response.has_next_page());
        assert!(response.has_prev_page());
    }

    #[test]
    fn test_paginate_last_page() {
        let items: Vec<i32> = (1..=25).collect();
        let params = PaginationParams::new(3, 10);
        let response = paginate(&items, &params);

        assert_eq!(response.items, vec![21, 22, 23, 24, 25]);
        assert!(!response.has_next_page());
        assert!(response.has_prev_page());
    }

    #[test]
    fn test_paginate_empty() {
        let items: Vec<i32> = Vec::new();
        let params = PaginationParams::new(1, 10);
        let response = paginate(&items, &params);

        assert!(response.items.is_empty());
        assert_eq!(response.total_count, 0);
    }
}
