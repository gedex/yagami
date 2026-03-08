use crate::checker::check_link;
use crate::cli::HttpMethod;
use crate::exclude::ExcludeFilter;
use crate::models::CrawlResult;
use crate::parser::extract_links;
use crate::state::SharedState;
use reqwest::Client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Semaphore};

pub struct Crawler {
    client: Client,
    state: SharedState,
    timeout: Duration,
    page_workers: usize,
    link_checkers: usize,
    exclude_filter: ExcludeFilter,
    http_method: HttpMethod,
}

impl Crawler {
    pub fn new(
        state: SharedState,
        timeout: Duration,
        page_workers: usize,
        link_checkers: usize,
        user_agent: &str,
        exclude_filter: ExcludeFilter,
        http_method: HttpMethod,
    ) -> Self {
        let client = Client::builder()
            .timeout(timeout)
            .user_agent(user_agent)
            .build()
            .unwrap();

        Self {
            client,
            state,
            timeout,
            page_workers,
            link_checkers,
            exclude_filter,
            http_method,
        }
    }

    pub async fn crawl_pages(
        &self,
        page_urls: Vec<String>,
        result_tx: mpsc::Sender<CrawlResult>,
        cancel_flag: Arc<AtomicBool>,
    ) {
        let total_pages = page_urls.len();
        self.state.set_total_pages(total_pages).await;

        let page_semaphore = Arc::new(Semaphore::new(self.page_workers));
        let link_semaphore = Arc::new(Semaphore::new(self.link_checkers));

        let mut handles = Vec::new();

        for page_url in page_urls {
            // Check for cancellation
            if cancel_flag.load(Ordering::Relaxed) {
                break;
            }

            if !self.state.mark_page_visited(&page_url) {
                continue; // Already visited
            }

            let client = self.client.clone();
            let state = self.state.clone();
            let timeout = self.timeout;
            let result_tx = result_tx.clone();
            let page_sem = page_semaphore.clone();
            let link_sem = link_semaphore.clone();
            let exclude_filter = self.exclude_filter.clone();
            let cancel = cancel_flag.clone();
            let http_method = self.http_method;

            let handle = tokio::spawn(async move {
                let _permit = page_sem.acquire().await.unwrap();

                // Check for cancellation before fetching
                if cancel.load(Ordering::Relaxed) {
                    return;
                }

                // Fetch and parse page
                match fetch_page(&client, &page_url, timeout).await {
                    Ok(html) => {
                        let links = extract_links(&html, &page_url);

                        // Spawn link checkers
                        let mut link_handles = Vec::new();
                        for link_url in links {
                            // Check for cancellation
                            if cancel.load(Ordering::Relaxed) {
                                break;
                            }

                            if !state.mark_link_checked(&link_url) {
                                continue; // Already checked - skip duplicates
                            }

                            // Check if link should be excluded
                            if exclude_filter.should_exclude(&link_url) {
                                continue; // Skip excluded links
                            }

                            // Only count unique links (that we actually check)
                            state.add_links(1).await;

                            let client = client.clone();
                            let page_url = page_url.clone();
                            let state = state.clone();
                            let result_tx = result_tx.clone();
                            let link_sem = link_sem.clone();

                            let link_handle = tokio::spawn(async move {
                                let _permit = link_sem.acquire().await.unwrap();

                                let check_result = check_link(&client, &link_url, timeout, http_method).await;

                                let crawl_result = CrawlResult {
                                    page_url: page_url.clone(),
                                    link_url: link_url.clone(),
                                    status_code: check_result.status_code,
                                    error: check_result.error,
                                };

                                state.update_with_result(&crawl_result).await;
                                let _ = result_tx.send(crawl_result).await;
                            });

                            link_handles.push(link_handle);
                        }

                        // Wait for all links from this page to be checked
                        for handle in link_handles {
                            let _ = handle.await;
                        }

                        state.increment_pages_crawled().await;
                    }
                    Err(_) => {
                        // If page fetch fails, still mark as crawled
                        state.increment_pages_crawled().await;
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all page crawlers to finish
        for handle in handles {
            let _ = handle.await;
        }

        // Explicitly drop result_tx to close the channel
        drop(result_tx);
    }
}

async fn fetch_page(client: &Client, url: &str, timeout: Duration) -> Result<String, reqwest::Error> {
    let response = client.get(url).timeout(timeout).send().await?;
    let html = response.text().await?;
    Ok(html)
}
