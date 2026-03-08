use scraper::{Html, Selector};
use url::Url;

pub fn extract_links(html: &str, base_url: &str) -> Vec<String> {
    let document = Html::parse_document(html);
    let mut links = Vec::new();

    let base = match Url::parse(base_url) {
        Ok(url) => url,
        Err(_) => return links,
    };

    // Extract href attributes from <a>, <link>
    if let Ok(selector) = Selector::parse("a[href], link[href]") {
        for element in document.select(&selector) {
            if let Some(href) = element.value().attr("href") {
                if let Some(url) = normalize_url(href, &base) {
                    links.push(url);
                }
            }
        }
    }

    // Extract src attributes from <img>, <script>, <iframe>
    if let Ok(selector) = Selector::parse("img[src], script[src], iframe[src]") {
        for element in document.select(&selector) {
            if let Some(src) = element.value().attr("src") {
                if let Some(url) = normalize_url(src, &base) {
                    links.push(url);
                }
            }
        }
    }

    links
}

fn normalize_url(href: &str, base: &Url) -> Option<String> {
    let trimmed = href.trim();

    // Filter out non-http(s) schemes
    if trimmed.starts_with("javascript:")
        || trimmed.starts_with("mailto:")
        || trimmed.starts_with("tel:")
        || trimmed.starts_with("data:")
        || trimmed.starts_with("#")
        || trimmed.is_empty()
    {
        return None;
    }

    // Parse and resolve relative URLs
    let url = match base.join(trimmed) {
        Ok(url) => url,
        Err(_) => return None,
    };

    // Only keep http(s) URLs
    if url.scheme() != "http" && url.scheme() != "https" {
        return None;
    }

    // Remove fragment
    let mut url = url;
    url.set_fragment(None);

    Some(url.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_links() {
        let html = r#"
            <html>
                <body>
                    <a href="https://example.com/page1">Link 1</a>
                    <a href="/page2">Link 2</a>
                    <img src="/image.png">
                    <script src="https://example.com/script.js"></script>
                </body>
            </html>
        "#;

        let links = extract_links(html, "https://example.com/");
        assert!(links.len() >= 3);
        assert!(links.contains(&"https://example.com/page1".to_string()));
        assert!(links.contains(&"https://example.com/page2".to_string()));
    }

    #[test]
    fn test_filter_invalid_schemes() {
        let html = r#"
            <html>
                <body>
                    <a href="javascript:void(0)">JS</a>
                    <a href="mailto:test@example.com">Email</a>
                    <a href="https://example.com/valid">Valid</a>
                </body>
            </html>
        "#;

        let links = extract_links(html, "https://example.com/");
        assert_eq!(links.len(), 1);
        assert_eq!(links[0], "https://example.com/valid");
    }

    #[test]
    fn test_normalize_relative_urls() {
        let base = Url::parse("https://example.com/path/page.html").unwrap();

        assert_eq!(
            normalize_url("/absolute", &base),
            Some("https://example.com/absolute".to_string())
        );

        assert_eq!(
            normalize_url("relative", &base),
            Some("https://example.com/path/relative".to_string())
        );

        assert_eq!(
            normalize_url("https://other.com/page", &base),
            Some("https://other.com/page".to_string())
        );
    }

    #[test]
    fn test_remove_fragments() {
        let base = Url::parse("https://example.com/").unwrap();

        assert_eq!(
            normalize_url("page#section", &base),
            Some("https://example.com/page".to_string())
        );
    }
}
