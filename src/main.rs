mod checker;
mod cli;
mod crawler;
mod errors;
mod exclude;
mod export;
mod models;
mod parser;
mod sitemap;
mod state;
mod tui;

use clap::Parser;
use cli::Args;
use crawler::Crawler;
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use exclude::ExcludeFilter;
use ratatui::{backend::CrosstermBackend, Terminal};
use sitemap::fetch_sitemap;
use state::SharedState;
use std::io;
use std::time::Duration;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tui::App;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    // Initialize shared state
    let state = SharedState::new();

    // Create HTTP client with User-Agent
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(args.timeout))
        .user_agent(&args.user_agent)
        .build()?;

    // Create exclude filter
    let exclude_filter = ExcludeFilter::new(args.exclude.clone());
    if !exclude_filter.is_empty() {
        println!("Excluding {} pattern(s) from link checking", exclude_filter.count());
    }

    // Fetch sitemap
    println!("Fetching sitemap from: {}", args.sitemap);
    println!("Processing sitemap (may include nested sitemaps)...");
    let page_urls = fetch_sitemap(&client, &args.sitemap, Duration::from_secs(args.timeout)).await?;
    println!("Found {} total pages to crawl", page_urls.len());

    // Create cancellation flag
    let cancel_flag = Arc::new(AtomicBool::new(false));

    // Create channels
    let (result_tx, result_rx) = mpsc::channel(1000);
    let (ui_tx, mut ui_rx) = mpsc::channel(100);

    // Spawn export task
    let output_path = args.output.clone();
    let export_handle = tokio::spawn(async move {
        if let Err(e) = export::export_results(result_rx, &output_path).await {
            eprintln!("Export error: {}", e);
        }
    });

    // Spawn crawler task
    let crawler = Crawler::new(
        state.clone(),
        Duration::from_secs(args.timeout),
        args.workers,
        args.checkers,
        &args.user_agent,
        exclude_filter,
        args.method,
    );
    let crawl_cancel = cancel_flag.clone();
    let crawl_handle = tokio::spawn(async move {
        crawler.crawl_pages(page_urls, result_tx, crawl_cancel).await;
    });

    // Spawn stats update task
    let stats_state = state.clone();
    let ui_tx_clone = ui_tx.clone();
    let stats_cancel = cancel_flag.clone();
    let stats_handle = tokio::spawn(async move {
        loop {
            if stats_cancel.load(Ordering::Relaxed) {
                break;
            }
            let stats = stats_state.get_stats().await;
            if ui_tx_clone.send(stats).await.is_err() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
    });

    // Setup TUI
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new(args.workers, args.checkers);

    // Main UI loop
    let mut user_quit = false;
    loop {
        // Draw UI
        terminal.draw(|f| tui::ui::render(f, &app))?;

        // Handle input
        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.code == KeyCode::Char('q') {
                    user_quit = true;
                    cancel_flag.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }

        // Update stats
        while let Ok(stats) = ui_rx.try_recv() {
            app.update_stats(stats);
        }

        // Check if crawling is done
        // Note: We only check pages because each page task waits for its link checking tasks
        if app.stats.total_pages > 0 && app.stats.pages_crawled >= app.stats.total_pages {
            // Give a bit more time for final stats to update
            tokio::time::sleep(Duration::from_millis(500)).await;
            break;
        }
    }

    // Cleanup terminal
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    if user_quit {
        println!("\nShutting down gracefully...");
        // Give tasks a moment to notice cancellation
        tokio::time::sleep(Duration::from_millis(200)).await;
    } else {
        println!("\nCrawling complete, finalizing...");
    }

    // Wait for tasks to complete with timeout
    let shutdown_timeout = Duration::from_secs(2);
    let _ = tokio::time::timeout(shutdown_timeout, crawl_handle).await;
    drop(ui_tx); // Close sender
    drop(ui_rx); // Close receiver - this will cause the stats task's send to fail
    let _ = tokio::time::timeout(shutdown_timeout, stats_handle).await;
    let _ = tokio::time::timeout(shutdown_timeout, export_handle).await;

    // Print final stats
    let final_stats = state.get_stats().await;
    println!("\n=== Final Statistics ===");
    println!("Pages crawled: {}/{}", final_stats.pages_crawled, final_stats.total_pages);
    println!("Links checked: {}/{}", final_stats.links_checked, final_stats.total_links);
    println!("2xx Success:   {}", final_stats.get_2xx_count());
    println!("3xx Redirect:  {}", final_stats.get_3xx_count());
    println!("4xx Client:    {}", final_stats.get_4xx_count());
    println!("5xx Server:    {}", final_stats.get_5xx_count());
    println!("\nResults exported to: {}", args.output);

    Ok(())
}
