use crate::errors::{Result, YagamiError};
use quick_xml::events::Event;
use quick_xml::Reader;
use reqwest::Client;
use std::collections::HashSet;
use std::time::Duration;
use std::pin::Pin;
use std::future::Future;

const MAX_RECURSION_DEPTH: usize = 10;

pub async fn fetch_sitemap(client: &Client, sitemap_url: &str, timeout: Duration) -> Result<Vec<String>> {
    let mut visited_sitemaps = HashSet::new();
    fetch_sitemap_recursive(client, sitemap_url, timeout, 0, &mut visited_sitemaps).await
}

fn fetch_sitemap_recursive<'a>(
    client: &'a Client,
    sitemap_url: &'a str,
    timeout: Duration,
    depth: usize,
    visited: &'a mut HashSet<String>,
) -> Pin<Box<dyn Future<Output = Result<Vec<String>>> + 'a>> {
    Box::pin(async move {
        if depth > MAX_RECURSION_DEPTH {
            return Err(YagamiError::InvalidSitemap(
                "Maximum recursion depth exceeded".to_string(),
            ));
        }

        if visited.contains(sitemap_url) {
            return Ok(Vec::new());
        }

        visited.insert(sitemap_url.to_string());

        let response = client
            .get(sitemap_url)
            .timeout(timeout)
            .send()
            .await?
            .error_for_status()?;

        let body = response.text().await?;
        let sitemap_type = detect_sitemap_type(&body)?;

        match sitemap_type {
            SitemapType::Index => {
                let sitemap_urls = parse_sitemap_index(&body)?;
                let mut all_urls = Vec::new();

                for nested_sitemap_url in sitemap_urls {
                    match fetch_sitemap_recursive(client, &nested_sitemap_url, timeout, depth + 1, visited).await {
                        Ok(mut urls) => all_urls.append(&mut urls),
                        Err(e) => {
                            eprintln!("Warning: Failed to fetch sitemap {}: {}", nested_sitemap_url, e);
                            continue;
                        }
                    }
                }

                if all_urls.is_empty() {
                    return Err(YagamiError::InvalidSitemap(
                        "No URLs found in sitemap index".to_string(),
                    ));
                }

                Ok(all_urls)
            }
            SitemapType::UrlSet => parse_urlset(&body),
        }
    })
}

#[derive(Debug, PartialEq)]
enum SitemapType {
    Index,
    UrlSet,
}

fn detect_sitemap_type(xml_content: &str) -> Result<SitemapType> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);
    let mut buf = Vec::new();

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"sitemapindex" {
                    return Ok(SitemapType::Index);
                } else if name.as_ref() == b"urlset" {
                    return Ok(SitemapType::UrlSet);
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(YagamiError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    Err(YagamiError::InvalidSitemap(
        "Unable to determine sitemap type".to_string(),
    ))
}

fn parse_sitemap_index(xml_content: &str) -> Result<Vec<String>> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut sitemap_urls = Vec::new();
    let mut buf = Vec::new();
    let mut in_sitemap = false;
    let mut in_loc = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"sitemap" {
                    in_sitemap = true;
                } else if name.as_ref() == b"loc" && in_sitemap {
                    in_loc = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_loc {
                    let url = e.unescape()?.to_string();
                    sitemap_urls.push(url);
                    in_loc = false;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"loc" {
                    in_loc = false;
                } else if name.as_ref() == b"sitemap" {
                    in_sitemap = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(YagamiError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    if sitemap_urls.is_empty() {
        return Err(YagamiError::InvalidSitemap(
            "No sitemap references found in sitemap index".to_string(),
        ));
    }

    Ok(sitemap_urls)
}

fn parse_urlset(xml_content: &str) -> Result<Vec<String>> {
    let mut reader = Reader::from_str(xml_content);
    reader.config_mut().trim_text(true);

    let mut urls = Vec::new();
    let mut buf = Vec::new();
    let mut in_url = false;
    let mut in_loc = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) | Ok(Event::Empty(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"url" {
                    in_url = true;
                } else if name.as_ref() == b"loc" && in_url {
                    in_loc = true;
                }
            }
            Ok(Event::Text(e)) => {
                if in_loc {
                    let url = e.unescape()?.to_string();
                    urls.push(url);
                    in_loc = false;
                }
            }
            Ok(Event::End(ref e)) => {
                let name = e.name();
                if name.as_ref() == b"loc" {
                    in_loc = false;
                } else if name.as_ref() == b"url" {
                    in_url = false;
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(YagamiError::XmlError(e)),
            _ => {}
        }
        buf.clear();
    }

    if urls.is_empty() {
        return Err(YagamiError::InvalidSitemap(
            "No URLs found in urlset".to_string(),
        ));
    }

    Ok(urls)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_urlset() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url>
                <loc>https://example.com/page1</loc>
            </url>
        </urlset>"#;

        let sitemap_type = detect_sitemap_type(xml).unwrap();
        assert_eq!(sitemap_type, SitemapType::UrlSet);
    }

    #[test]
    fn test_detect_sitemapindex() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sitemap>
                <loc>https://example.com/sitemap1.xml</loc>
            </sitemap>
        </sitemapindex>"#;

        let sitemap_type = detect_sitemap_type(xml).unwrap();
        assert_eq!(sitemap_type, SitemapType::Index);
    }

    #[test]
    fn test_parse_urlset() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url>
                <loc>https://example.com/page1</loc>
            </url>
            <url>
                <loc>https://example.com/page2</loc>
            </url>
        </urlset>"#;

        let urls = parse_urlset(xml).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://example.com/page1");
        assert_eq!(urls[1], "https://example.com/page2");
    }

    #[test]
    fn test_parse_empty_urlset() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        </urlset>"#;

        let result = parse_urlset(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_sitemap_index() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <sitemap>
                <loc>https://example.com/sitemap1.xml</loc>
                <lastmod>2026-03-01</lastmod>
            </sitemap>
            <sitemap>
                <loc>https://example.com/sitemap2.xml</loc>
                <lastmod>2026-03-02</lastmod>
            </sitemap>
        </sitemapindex>"#;

        let sitemap_urls = parse_sitemap_index(xml).unwrap();
        assert_eq!(sitemap_urls.len(), 2);
        assert_eq!(sitemap_urls[0], "https://example.com/sitemap1.xml");
        assert_eq!(sitemap_urls[1], "https://example.com/sitemap2.xml");
    }

    #[test]
    fn test_parse_empty_sitemap_index() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
        </sitemapindex>"#;

        let result = parse_sitemap_index(xml);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_urlset_with_namespace() {
        let xml = r#"<?xml version="1.0" encoding="UTF-8"?>
        <urlset xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance"
                xsi:schemaLocation="http://www.sitemaps.org/schemas/sitemap/0.9 http://www.sitemaps.org/schemas/sitemap/0.9/sitemap.xsd"
                xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
            <url>
                <loc>https://developer.woocommerce.com/about/</loc>
                <lastmod>2025-02-06T11:27:59Z</lastmod>
            </url>
            <url>
                <loc>https://developer.woocommerce.com/blog/</loc>
                <lastmod>2024-02-17T13:12:30Z</lastmod>
            </url>
        </urlset>"#;

        let urls = parse_urlset(xml).unwrap();
        assert_eq!(urls.len(), 2);
        assert_eq!(urls[0], "https://developer.woocommerce.com/about/");
        assert_eq!(urls[1], "https://developer.woocommerce.com/blog/");
    }
}
