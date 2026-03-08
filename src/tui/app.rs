use crate::models::CrawlStats;

pub struct App {
    pub stats: CrawlStats,
    pub page_workers: usize,
    pub link_checkers: usize,
}

impl App {
    pub fn new(page_workers: usize, link_checkers: usize) -> Self {
        Self {
            stats: CrawlStats::new(),
            page_workers,
            link_checkers,
        }
    }

    pub fn update_stats(&mut self, stats: CrawlStats) {
        self.stats = stats;
    }
}

