use std::collections::HashMap;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct CrawlResult {
    pub page_url: String,
    pub link_url: String,
    pub status_code: u16,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct CrawlStats {
    pub total_pages: usize,
    pub pages_crawled: usize,
    pub total_links: usize,
    pub links_checked: usize,
    pub status_counts: HashMap<u16, usize>,
}

impl CrawlStats {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn increment_status(&mut self, status_code: u16) {
        *self.status_counts.entry(status_code).or_insert(0) += 1;
    }

    pub fn get_2xx_count(&self) -> usize {
        self.status_counts
            .iter()
            .filter(|(code, _)| **code >= 200 && **code < 300)
            .map(|(_, count)| count)
            .sum()
    }

    pub fn get_3xx_count(&self) -> usize {
        self.status_counts
            .iter()
            .filter(|(code, _)| **code >= 300 && **code < 400)
            .map(|(_, count)| count)
            .sum()
    }

    pub fn get_4xx_count(&self) -> usize {
        self.status_counts
            .iter()
            .filter(|(code, _)| **code >= 400 && **code < 500)
            .map(|(_, count)| count)
            .sum()
    }

    pub fn get_5xx_count(&self) -> usize {
        self.status_counts
            .iter()
            .filter(|(code, _)| **code >= 500 && **code < 600)
            .map(|(_, count)| count)
            .sum()
    }
}
