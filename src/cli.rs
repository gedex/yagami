use clap::{Parser, ValueEnum};

const DEFAULT_USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36";

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum HttpMethod {
    Get,
    Head,
}

#[derive(Parser, Debug)]
#[command(name = "yagami")]
#[command(about = "A concurrent web link checker", long_about = None)]
pub struct Args {
    /// Sitemap.xml URL
    pub sitemap: String,

    /// Number of concurrent page crawlers
    #[arg(short, long, default_value = "10")]
    pub workers: usize,

    /// Number of concurrent link checkers
    #[arg(short, long, default_value = "50")]
    pub checkers: usize,

    /// CSV output file path
    #[arg(short, long, default_value = "results.csv")]
    pub output: String,

    /// Request timeout in seconds
    #[arg(short, long, default_value = "30")]
    pub timeout: u64,

    /// User-Agent header for HTTP requests
    #[arg(short = 'A', long, default_value = DEFAULT_USER_AGENT)]
    pub user_agent: String,

    /// Exclude patterns for links to skip checking (can be specified multiple times)
    /// Examples: example.com, https://example.com/foo/*, https://example.com/*
    #[arg(short = 'e', long = "exclude")]
    pub exclude: Vec<String>,

    /// HTTP method to use for link checking (get or head)
    #[arg(short = 'X', long = "method", default_value = "get")]
    pub method: HttpMethod,
}
