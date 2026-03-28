# Yagami

A concurrent web link checker that parses sitemap.xml files, crawls pages, extracts links, checks their HTTP status, displays real-time TUI progress, and exports results to CSV.

## Features

- **Sitemap Parsing**: Automatically fetches and parses sitemap.xml files
  - Supports sitemap index files (recursively follows nested sitemaps)
  - Automatically detects and processes both `<sitemapindex>` and `<urlset>` formats
  - Includes recursion depth protection and duplicate sitemap detection
- **Concurrent Crawling**: Worker pool with configurable concurrency for page crawling
- **Link Extraction**: Extracts links from `<a>`, `<img>`, `<script>`, `<link>`, and `<iframe>` tags
- **HTTP Status Checking**: Checks link availability with configurable timeout
- **Real-time TUI**: Terminal UI showing live progress and statistics
- **CSV Export**: Streams results to CSV format for analysis
- **Deduplication**: Automatically avoids re-checking duplicate URLs
- **Error Handling**: Distinguishes between connection errors, timeouts, and HTTP errors

## Installation

### From Source

```bash
git clone <repository-url>
cd yagami
cargo build --release
```

The binary will be available at `./target/release/yagami`

## Usage

### Basic Usage

```bash
yagami https://example.com/sitemap.xml
```

### Advanced Options

```bash
yagami <SITEMAP> [OPTIONS]

Arguments:
  <SITEMAP>                   Sitemap.xml URL (required)

Options:
  -w, --workers <N>           Number of concurrent page crawlers (default: 10)
  -c, --checkers <N>          Number of concurrent link checkers (default: 50)
  -o, --output <FILE>         CSV output file path (default: results.csv)
  -t, --timeout <SECONDS>     Request timeout in seconds (default: 30)
  -A, --user-agent <STRING>   User-Agent header for HTTP requests
                              (default: Chrome browser User-Agent)
  -e, --exclude <PATTERN>     Exclude patterns for links to skip checking
                              (can be specified multiple times)
  -h, --help                  Print help information
```

### Examples

Check a sitemap with custom concurrency:
```bash
yagami https://example.com/sitemap.xml --workers 20 --checkers 100
```

Save results to a custom file:
```bash
yagami https://example.com/sitemap.xml --output my-results.csv
```

Use shorter timeout for faster results:
```bash
yagami https://example.com/sitemap.xml --timeout 10
```

Use a custom User-Agent (e.g., identify yourself):
```bash
yagami https://example.com/sitemap.xml --user-agent "MyBot/1.0 (+https://mysite.com/bot)"
```

Use a different browser User-Agent:
```bash
yagami https://example.com/sitemap.xml --user-agent "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15"
```

Exclude specific domains or URL patterns:
```bash
# Skip all links from example.com (both http and https)
yagami https://mysite.com/sitemap.xml -e example.com

# Skip links matching a specific path pattern
yagami https://mysite.com/sitemap.xml -e "https://example.com/api/*"

# Multiple exclude patterns - use -e flag multiple times
yagami https://mysite.com/sitemap.xml \
  -e example.com \
  -e "https://other.com/private/*" \
  -e "https://third.com/*"

# Skip all social media links
yagami https://mysite.com/sitemap.xml \
  -e facebook.com \
  -e twitter.com \
  -e linkedin.com \
  -e instagram.com \
  -e youtube.com

# Skip external services and CDN
yagami https://mysite.com/sitemap.xml \
  -e google-analytics.com \
  -e googletagmanager.com \
  -e "https://cdn.example.com/*" \
  -e doubleclick.net

# Mix domain and path patterns
yagami https://mysite.com/sitemap.xml \
  -e example.com \
  -e "https://api.mysite.com/*" \
  -e "https://mysite.com/admin/*"
```

## Output

### Terminal UI

The TUI displays:
- **Progress bar**: Pages crawled with percentage
- **Statistics**: Real-time counts of status codes (2xx, 3xx, 4xx, 5xx)
- **Workers**: Configuration and activity metrics
  - Page workers and link checkers configured
  - Total links checked
  - Pending pages in queue

Press `q` to quit the application (it will finish processing before exiting).

### CSV Output

Results are saved in CSV format with the following columns:

| Column       | Description                                      |
|--------------|--------------------------------------------------|
| page_url     | The page where the link was found                |
| link_url     | The link that was checked                        |
| status_code  | HTTP status code (0 if request failed)           |
| error        | Error message if the request failed (optional)   |

Example:
```csv
page_url,link_url,status_code,error
https://example.com/page1,https://example.com/about,200,
https://example.com/page1,https://example.com/broken,404,
https://example.com/page2,https://invalid.domain,0,Connection failed
```

## Exclude Patterns

You can exclude certain links from being checked using the `--exclude` (or `-e`) option. This is useful for:
- **Skipping external domains** you don't want to check
- **Avoiding rate limits** on specific APIs or services
- **Excluding authenticated areas** that require login
- **Skipping known broken links** that you can't fix

### Pattern Syntax

**Domain-only pattern** (matches both http and https):
```bash
-e example.com
```
Matches:
- `https://example.com`
- `http://example.com`
- `https://example.com/any/path`
- `https://www.example.com` (subdomains too)

**Wildcard pattern** (matches URL prefix):
```bash
-e "https://example.com/api/*"
```
Matches:
- `https://example.com/api/`
- `https://example.com/api/v1`
- `https://example.com/api/users?id=123`

Does NOT match:
- `http://example.com/api/v1` (different scheme)
- `https://example.com/docs` (different path)

**Full domain wildcard**:
```bash
-e "https://example.com/*"
```
Matches all URLs under `https://example.com/`

### Multiple Patterns

**How to specify**: Use the `-e` (or `--exclude`) flag multiple times - once for each pattern you want to exclude.

```bash
# Basic syntax - repeat the -e flag
yagami https://mysite.com/sitemap.xml \
  -e example.com \
  -e "https://api.other.com/*" \
  -e "https://third.com/private/*"

# Or with long form
yagami https://mysite.com/sitemap.xml \
  --exclude example.com \
  --exclude "https://api.other.com/*" \
  --exclude "https://third.com/private/*"
```

**Important**: Each pattern needs its own `-e` flag. You cannot combine patterns with commas or spaces.

When you run with multiple patterns, you'll see:
```
Excluding 3 pattern(s) from link checking
```

### Use Cases

**Skip social media links:**
```bash
yagami sitemap.xml \
  -e facebook.com \
  -e twitter.com \
  -e linkedin.com \
  -e instagram.com \
  -e youtube.com
```

**Skip CDN and analytics:**
```bash
yagami sitemap.xml \
  -e "https://cdn.example.com/*" \
  -e google-analytics.com \
  -e googletagmanager.com \
  -e doubleclick.net
```

**Skip API endpoints and admin areas:**
```bash
yagami sitemap.xml \
  -e "https://api.mysite.com/*" \
  -e "https://mysite.com/admin/*" \
  -e "https://mysite.com/wp-admin/*"
```

**Mix domain and path patterns:**
```bash
yagami sitemap.xml \
  -e example.com \
  -e other-domain.com \
  -e "https://mysite.com/private/*" \
  -e "https://cdn.mysite.com/*"
```

## User-Agent

Yagami uses a browser-like User-Agent by default (Chrome) to avoid being blocked by websites. Many sites (like Google, Facebook, etc.) return different content or block requests from non-browser User-Agents.

**Default User-Agent:**
```
Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/121.0.0.0 Safari/537.36
```

**Why this matters:**
- Sites like Google Support may return 404 for bot User-Agents but 200 for browsers
- Accurate link checking requires seeing what real users see
- Using a browser User-Agent gives you the true status of links

**When to customize:**
- **Identify yourself**: Use a custom User-Agent if you want to identify your bot for transparency
- **Mobile testing**: Test with mobile User-Agents to check mobile-specific URLs
- **Specific browser**: Test with Safari, Firefox, or other browser User-Agents

Example with custom User-Agent:
```bash
yagami https://example.com/sitemap.xml -A "MyCompanyBot/1.0 (+https://mycompany.com/bot-info)"
```

## Sitemap Index Support

Yagami automatically handles sitemap index files that reference other sitemaps. When you provide a sitemap URL, Yagami will:

1. **Detect the type**: Automatically determine if the file is a `<sitemapindex>` or `<urlset>`
2. **Recursively fetch**: If it's a sitemap index, follow all `<sitemap><loc>` references
3. **Aggregate results**: Collect all page URLs from all nested sitemaps
4. **Protection**: Includes recursion depth limits (max 10 levels) and duplicate detection

Example sitemap index structure:
```xml
<sitemapindex xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
 <sitemap>
  <loc>https://example.com/sitemap-1.xml</loc>
 </sitemap>
 <sitemap>
  <loc>https://example.com/sitemap-2.xml</loc>
 </sitemap>
</sitemapindex>
```

Each referenced sitemap is fetched and parsed for `<url><loc>` entries automatically.

## Link Extraction

Yagami extracts and checks links from:
- `<a href="...">`
- `<img src="...">`
- `<link href="...">`
- `<script src="...">`
- `<iframe src="...">`

It automatically:
- Filters out `javascript:`, `mailto:`, `tel:`, and `data:` schemes
- Resolves relative URLs to absolute URLs
- Removes URL fragments
- Normalizes URLs to prevent duplicates

## Testing

Run the test suite:

```bash
cargo test
```

Tests cover:
- Sitemap parsing
- Link extraction and normalization
- HTTP status checking
- Error handling

## Release Process

To create a new release:

1. **Update version** in `Cargo.toml`:
   ```toml
   [package]
   version = "x.y.z"
   ```

2. **Create and push a git tag**:
   ```bash
   git tag x.y.z
   git push origin x.y.z
   ```

3. **GitHub Action handles the rest**: The CI/CD pipeline will automatically:
   - Build binaries for all platforms
   - Create a GitHub release
   - Publish release artifacts

## License

MIT
