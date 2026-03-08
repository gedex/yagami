use url::Url;

#[derive(Debug, Clone)]
pub struct ExcludePattern {
    pattern: String,
}

impl ExcludePattern {
    pub fn new(pattern: String) -> Self {
        Self { pattern }
    }

    pub fn matches(&self, url: &str) -> bool {
        let pattern = &self.pattern;

        // Parse the URL to check
        let parsed_url = match Url::parse(url) {
            Ok(u) => u,
            Err(_) => return false,
        };

        // Case 1: Pattern is just a domain (e.g., "example.com")
        // Should match both http://example.com and https://example.com
        if !pattern.contains("://") && !pattern.contains('/') {
            if let Some(host) = parsed_url.host_str() {
                return host == pattern || host.ends_with(&format!(".{}", pattern));
            }
            return false;
        }

        // Case 2: Pattern has wildcard (e.g., "https://example.com/*")
        if pattern.ends_with("/*") {
            let prefix = &pattern[..pattern.len() - 1]; // Remove the *
            return url.starts_with(prefix);
        }

        // Case 3: Exact match
        url == pattern
    }
}

#[derive(Debug, Clone)]
pub struct ExcludeFilter {
    patterns: Vec<ExcludePattern>,
}

impl ExcludeFilter {
    pub fn new(patterns: Vec<String>) -> Self {
        let patterns = patterns.into_iter().map(ExcludePattern::new).collect();
        Self { patterns }
    }

    pub fn should_exclude(&self, url: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.matches(url))
    }

    pub fn is_empty(&self) -> bool {
        self.patterns.is_empty()
    }

    pub fn count(&self) -> usize {
        self.patterns.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_domain_only_pattern() {
        let pattern = ExcludePattern::new("example.com".to_string());

        assert!(pattern.matches("https://example.com"));
        assert!(pattern.matches("http://example.com"));
        assert!(pattern.matches("https://example.com/"));
        assert!(pattern.matches("https://example.com/foo"));
        assert!(pattern.matches("https://example.com/foo?bar=baz"));

        assert!(!pattern.matches("https://other.com"));
        assert!(!pattern.matches("https://notexample.com"));
    }

    #[test]
    fn test_subdomain_pattern() {
        let pattern = ExcludePattern::new("example.com".to_string());

        // Should match subdomains
        assert!(pattern.matches("https://www.example.com"));
        assert!(pattern.matches("https://api.example.com/v1"));
    }

    #[test]
    fn test_wildcard_pattern() {
        let pattern = ExcludePattern::new("https://example.com/foo/*".to_string());

        assert!(pattern.matches("https://example.com/foo/"));
        assert!(pattern.matches("https://example.com/foo/bar"));
        assert!(pattern.matches("https://example.com/foo/bar/baz"));
        assert!(pattern.matches("https://example.com/foo/?bar=baz"));

        assert!(!pattern.matches("https://example.com/"));
        assert!(!pattern.matches("https://example.com/bar"));
        assert!(!pattern.matches("http://example.com/foo/bar")); // Different scheme
    }

    #[test]
    fn test_domain_wildcard_pattern() {
        let pattern = ExcludePattern::new("https://example.com/*".to_string());

        assert!(pattern.matches("https://example.com/"));
        assert!(pattern.matches("https://example.com/foo"));
        assert!(pattern.matches("https://example.com/foo?bar=baz"));
        assert!(pattern.matches("https://example.com/foo/bar/baz"));

        assert!(!pattern.matches("http://example.com/foo")); // Different scheme
        assert!(!pattern.matches("https://other.com/"));
    }

    #[test]
    fn test_exact_match() {
        let pattern = ExcludePattern::new("https://example.com/exact".to_string());

        assert!(pattern.matches("https://example.com/exact"));

        assert!(!pattern.matches("https://example.com/exact/"));
        assert!(!pattern.matches("https://example.com/exact/path"));
        assert!(!pattern.matches("https://example.com/other"));
    }

    #[test]
    fn test_exclude_filter() {
        let filter = ExcludeFilter::new(vec![
            "example.com".to_string(),
            "https://other.com/api/*".to_string(),
        ]);

        assert!(filter.should_exclude("https://example.com/foo"));
        assert!(filter.should_exclude("https://other.com/api/v1"));

        assert!(!filter.should_exclude("https://different.com"));
        assert!(!filter.should_exclude("https://other.com/docs"));
    }

    #[test]
    fn test_empty_filter() {
        let filter = ExcludeFilter::new(vec![]);

        assert!(!filter.should_exclude("https://example.com"));
        assert!(filter.is_empty());
    }
}
