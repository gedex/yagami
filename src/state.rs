use crate::models::{CrawlResult, CrawlStats};
use dashmap::DashSet;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct SharedState {
    pub stats: Arc<Mutex<CrawlStats>>,
    pub visited_pages: Arc<DashSet<String>>,
    pub checked_links: Arc<DashSet<String>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            stats: Arc::new(Mutex::new(CrawlStats::new())),
            visited_pages: Arc::new(DashSet::new()),
            checked_links: Arc::new(DashSet::new()),
        }
    }

    pub async fn update_with_result(&self, result: &CrawlResult) {
        let mut stats = self.stats.lock().await;
        stats.links_checked += 1;
        stats.increment_status(result.status_code);
    }

    pub async fn increment_pages_crawled(&self) {
        let mut stats = self.stats.lock().await;
        stats.pages_crawled += 1;
    }

    pub async fn set_total_pages(&self, total: usize) {
        let mut stats = self.stats.lock().await;
        stats.total_pages = total;
    }

    pub async fn add_links(&self, count: usize) {
        let mut stats = self.stats.lock().await;
        stats.total_links += count;
    }

    pub fn mark_page_visited(&self, url: &str) -> bool {
        self.visited_pages.insert(url.to_string())
    }

    pub fn mark_link_checked(&self, url: &str) -> bool {
        self.checked_links.insert(url.to_string())
    }

    pub async fn get_stats(&self) -> CrawlStats {
        self.stats.lock().await.clone()
    }
}

impl Default for SharedState {
    fn default() -> Self {
        Self::new()
    }
}
