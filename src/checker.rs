use crate::cli::HttpMethod;
use reqwest::Client;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub status_code: u16,
    pub error: Option<String>,
}

pub async fn check_link(
    client: &Client,
    url: &str,
    timeout: Duration,
    method: HttpMethod,
) -> CheckResult {
    let request = match method {
        HttpMethod::Get => client.get(url),
        HttpMethod::Head => client.head(url),
    };

    match request.timeout(timeout).send().await {
        Ok(response) => CheckResult {
            status_code: response.status().as_u16(),
            error: None,
        },
        Err(e) => CheckResult {
            status_code: 0,
            error: Some(format_error(&e)),
        },
    }
}

fn format_error(err: &reqwest::Error) -> String {
    if err.is_timeout() {
        "Timeout".to_string()
    } else if err.is_connect() {
        "Connection failed".to_string()
    } else if err.is_status() {
        format!("HTTP {}", err.status().map_or("error".to_string(), |s| s.to_string()))
    } else {
        "Request failed".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_check_valid_link() {
        let client = Client::new();
        let result = check_link(
            &client,
            "https://www.google.com/",
            Duration::from_secs(10),
            HttpMethod::Get,
        )
        .await;

        assert_eq!(result.status_code, 200);
        assert!(result.error.is_none());
    }

    #[tokio::test]
    async fn test_check_invalid_link() {
        let client = Client::new();
        let result = check_link(
            &client,
            "https://httpbin.org/status/404",
            Duration::from_secs(10),
            HttpMethod::Get,
        )
        .await;

        assert_eq!(result.status_code, 404);
    }

    #[tokio::test]
    async fn test_check_nonexistent_domain() {
        let client = Client::new();
        let result = check_link(
            &client,
            "https://this-domain-definitely-does-not-exist-12345.com/",
            Duration::from_secs(5),
            HttpMethod::Get,
        )
        .await;

        assert_eq!(result.status_code, 0);
        assert!(result.error.is_some());
    }
}
