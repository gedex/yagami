use thiserror::Error;

#[derive(Error, Debug)]
pub enum YagamiError {
    #[error("HTTP request failed: {0}")]
    HttpError(#[from] reqwest::Error),

    #[error("XML parsing failed: {0}")]
    XmlError(#[from] quick_xml::Error),

    #[error("URL parsing failed: {0}")]
    UrlError(#[from] url::ParseError),

    #[error("CSV writing failed: {0}")]
    CsvError(#[from] csv::Error),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Invalid sitemap: {0}")]
    InvalidSitemap(String),
}

pub type Result<T> = std::result::Result<T, YagamiError>;
