//! HTTP client with rate limiting and retry logic.
//!
//! Provides a shared HTTP client for all provider implementations.

use crate::USER_AGENT;
use crate::constants::{BASE_BACKOFF_MS, MAX_BACKOFF_JITTER_MS};
use crate::error::{DxError, Result};
use crate::types::{MediaType, RateLimitConfig};
use reqwest::header::{ACCEPT_LANGUAGE, HeaderMap, HeaderValue, LOCATION};
use reqwest::{Client, Response, StatusCode, redirect::Policy};
use std::net::{IpAddr, SocketAddr, ToSocketAddrs};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

const MAX_VALIDATED_REDIRECTS: usize = 10;

/// HTTP client with built-in rate limiting and retry logic.
#[derive(Debug, Clone)]
pub struct HttpClient {
    client: Client,
    default_headers: HeaderMap,
    timeout: Duration,
    rate_limiter: Arc<RateLimiter>,
    max_retries: u32,
}

impl HttpClient {
    /// Create a new HTTP client with default settings.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.
    pub fn new() -> Result<Self> {
        Self::with_config(RateLimitConfig::default(), 3, Duration::from_secs(30))
    }

    /// Create a new HTTP client with custom configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.
    pub fn with_config(
        rate_limit: RateLimitConfig,
        max_retries: u32,
        timeout: Duration,
    ) -> Result<Self> {
        let mut headers = HeaderMap::new();
        // NOTE: Don't set Accept header globally - let each request specify it
        // API providers need application/json while scrapers need text/html
        headers.insert(ACCEPT_LANGUAGE, HeaderValue::from_static("en-US,en;q=0.9"));

        let client = Self::build_client(
            headers.clone(),
            timeout,
            Policy::limited(MAX_VALIDATED_REDIRECTS),
        )?;

        Ok(Self {
            client,
            default_headers: headers,
            timeout,
            rate_limiter: Arc::new(RateLimiter::new(rate_limit)),
            max_retries,
        })
    }

    fn build_client(
        headers: HeaderMap,
        timeout: Duration,
        redirect_policy: Policy,
    ) -> Result<Client> {
        // PERFORMANCE OPTIMIZATIONS:
        // - pool_max_idle_per_host: Keep 10 connections warm per API host
        // - pool_idle_timeout: Keep connections alive for 30s between requests
        // - tcp_nodelay: Disable Nagle's algorithm for faster small requests
        // - http2_adaptive_window: Optimize HTTP/2 flow control
        // - connection_verbose: Disabled for production
        Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(headers)
            .timeout(timeout)
            .connect_timeout(Duration::from_secs(5)) // Fast connection or fail
            .pool_max_idle_per_host(10) // Keep 10 connections warm per host
            .pool_idle_timeout(Duration::from_secs(30)) // Connections stay alive 30s
            .tcp_nodelay(true) // Disable Nagle's algorithm
            .gzip(true)
            .brotli(true)
            .redirect(redirect_policy)
            .build()
            .map_err(|e| DxError::http(e.to_string()))
    }

    fn build_client_pinned_to_addrs(
        &self,
        host: &str,
        addrs: &[SocketAddr],
        redirect_policy: Policy,
    ) -> Result<Client> {
        let mut builder = Client::builder()
            .user_agent(USER_AGENT)
            .default_headers(self.default_headers.clone())
            .timeout(self.timeout)
            .connect_timeout(Duration::from_secs(5))
            .pool_max_idle_per_host(0)
            .pool_idle_timeout(Duration::from_secs(0))
            .tcp_nodelay(true)
            .gzip(true)
            .brotli(true)
            .redirect(redirect_policy);

        builder = builder.resolve_to_addrs(host, addrs);

        builder.build().map_err(|e| DxError::http(e.to_string()))
    }

    /// Create a client with a specific rate limit.
    #[must_use]
    pub fn with_rate_limit(mut self, config: RateLimitConfig) -> Self {
        self.rate_limiter = Arc::new(RateLimiter::new(config));
        self
    }

    /// Execute a GET request with rate limiting and retries.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails after all retries.
    pub async fn get(&self, url: &str) -> Result<Response> {
        self.request_with_retry(|| self.client.get(url)).await
    }

    /// Execute a GET request with custom headers.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails after all retries.
    pub async fn get_with_headers(&self, url: &str, headers: &[(&str, &str)]) -> Result<Response> {
        self.request_with_retry(|| {
            let mut req = self.client.get(url);
            for (key, value) in headers {
                req = req.header(*key, *value);
            }
            req
        })
        .await
    }

    /// Execute a GET request with query parameters.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails after all retries.
    pub async fn get_with_query<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        query: &T,
        headers: &[(&str, &str)],
    ) -> Result<Response> {
        self.request_with_retry(|| {
            let mut req = self.client.get(url).query(query);
            for (key, value) in headers {
                // Convert to HeaderName and HeaderValue for proper handling
                if let (Ok(name), Ok(val)) = (
                    reqwest::header::HeaderName::from_bytes(key.as_bytes()),
                    reqwest::header::HeaderValue::from_str(value),
                ) {
                    req = req.header(name, val);
                }
            }
            req
        })
        .await
    }

    /// Execute a raw GET request (no automatic JSON parsing).
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails after all retries.
    pub async fn get_raw(&self, url: &str) -> Result<Response> {
        self.request_with_retry(|| self.client.get(url)).await
    }

    /// Execute a raw GET while validating each redirect target before following it.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails, a redirect is malformed, or a
    /// redirect target fails URL safety validation.
    pub async fn get_raw_validating_redirects(&self, url: &str) -> Result<Response> {
        self.request_with_retry_validating_redirects(url).await
    }

    /// Execute a POST request with JSON body.
    ///
    /// # Errors
    ///
    /// Returns an error if the request fails after all retries.
    pub async fn post_json<T: serde::Serialize + ?Sized>(
        &self,
        url: &str,
        body: &T,
    ) -> Result<Response> {
        self.request_with_retry(|| self.client.post(url).json(body))
            .await
    }

    /// Execute a request with rate limiting and retry logic.
    async fn request_with_retry<F>(&self, build_request: F) -> Result<Response>
    where
        F: Fn() -> reqwest::RequestBuilder,
    {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            // Wait for rate limit
            self.rate_limiter.acquire().await;

            let request = build_request();
            debug!(
                "HTTP request attempt {}/{}",
                attempt + 1,
                self.max_retries + 1
            );

            match request.send().await {
                Ok(response) => {
                    let status = response.status();

                    // Handle rate limiting
                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = response
                            .headers()
                            .get("retry-after")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(60);

                        warn!("Rate limited, waiting {} seconds", retry_after);
                        sleep(Duration::from_secs(retry_after)).await;
                        continue;
                    }

                    // Handle server errors with retry
                    if status.is_server_error() && attempt < self.max_retries {
                        let delay = Self::exponential_backoff(attempt);
                        warn!("Server error {}, retrying in {:?}", status.as_u16(), delay);
                        sleep(delay).await;
                        continue;
                    }

                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let delay = Self::exponential_backoff(attempt);
                        warn!("Request failed, retrying in {:?}", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error
            .map(DxError::from)
            .unwrap_or_else(|| DxError::http("Request failed after all retries")))
    }

    async fn request_with_retry_validating_redirects(&self, url: &str) -> Result<Response> {
        let mut last_error = None;

        for attempt in 0..=self.max_retries {
            self.rate_limiter.acquire().await;

            debug!(
                "HTTP request attempt {}/{} with redirect validation",
                attempt + 1,
                self.max_retries + 1
            );

            match self.send_following_validated_redirects(url).await {
                Ok(response) => {
                    let status = response.status();

                    if status == StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = response
                            .headers()
                            .get("retry-after")
                            .and_then(|h| h.to_str().ok())
                            .and_then(|s| s.parse::<u64>().ok())
                            .unwrap_or(60);

                        warn!("Rate limited, waiting {} seconds", retry_after);
                        sleep(Duration::from_secs(retry_after)).await;
                        continue;
                    }

                    if status.is_server_error() && attempt < self.max_retries {
                        let delay = Self::exponential_backoff(attempt);
                        warn!("Server error {}, retrying in {:?}", status.as_u16(), delay);
                        sleep(delay).await;
                        continue;
                    }

                    return Ok(response);
                }
                Err(e) => {
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        let delay = Self::exponential_backoff(attempt);
                        warn!("Request failed, retrying in {:?}", delay);
                        sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap_or_else(|| DxError::http("Request failed after all retries")))
    }

    async fn send_following_validated_redirects(&self, url: &str) -> Result<Response> {
        let mut current_url = validate_url(url)?;
        let mut addrs = resolve_public_socket_addrs(&current_url)?;

        for redirect_count in 0..=MAX_VALIDATED_REDIRECTS {
            let host = current_url
                .host_str()
                .ok_or_else(|| DxError::invalid_url(current_url.as_str(), "URL must have a host"))?
                .to_string();
            let response = self
                .build_client_pinned_to_addrs(&host, &addrs, Policy::none())?
                .get(current_url.clone())
                .send()
                .await
                .map_err(DxError::from)?;

            if !response.status().is_redirection() {
                return Ok(response);
            }

            if redirect_count == MAX_VALIDATED_REDIRECTS {
                return Err(DxError::Download {
                    url: url.to_string(),
                    message: format!("Too many redirects; limit is {MAX_VALIDATED_REDIRECTS}"),
                });
            }

            let location = response
                .headers()
                .get(LOCATION)
                .ok_or_else(|| DxError::Download {
                    url: current_url.to_string(),
                    message: "Redirect response did not include a Location header".to_string(),
                })?
                .to_str()
                .map_err(|_| DxError::Download {
                    url: current_url.to_string(),
                    message: "Redirect Location header is not valid UTF-8".to_string(),
                })?;

            current_url = validate_redirect_location(current_url.as_str(), location)?;
            addrs = resolve_public_socket_addrs(&current_url)?;
        }

        Err(DxError::Download {
            url: url.to_string(),
            message: format!("Too many redirects; limit is {MAX_VALIDATED_REDIRECTS}"),
        })
    }

    /// Calculate exponential backoff delay.
    fn exponential_backoff(attempt: u32) -> Duration {
        let delay = BASE_BACKOFF_MS * 2u64.pow(attempt);
        // Simple jitter without external rand crate
        let jitter = (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .subsec_nanos() as u64)
            % MAX_BACKOFF_JITTER_MS;
        Duration::from_millis(delay + jitter)
    }

    /// Get the underlying reqwest client.
    #[must_use]
    pub fn inner(&self) -> &Client {
        &self.client
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default HTTP client")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RATE LIMITER
// ═══════════════════════════════════════════════════════════════════════════════

/// Simple token bucket rate limiter.
#[derive(Debug)]
struct RateLimiter {
    config: RateLimitConfig,
    last_request: AtomicU64,
}

impl RateLimiter {
    fn new(config: RateLimitConfig) -> Self {
        Self {
            config,
            last_request: AtomicU64::new(0),
        }
    }

    async fn acquire(&self) {
        let delay_ms = self.config.delay_ms();
        if delay_ms == 0 {
            return;
        }

        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let last = self.last_request.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last);

        if elapsed < delay_ms {
            let wait = delay_ms - elapsed;
            sleep(Duration::from_millis(wait)).await;
        }

        self.last_request.store(now, Ordering::Relaxed);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// RESPONSE HELPERS
// ═══════════════════════════════════════════════════════════════════════════════

/// Extension trait for response handling.
pub trait ResponseExt {
    /// Check if the response indicates success and return body as JSON.
    fn json_or_error<T: serde::de::DeserializeOwned>(
        self,
    ) -> impl std::future::Future<Output = Result<T>> + Send;
}

impl ResponseExt for Response {
    async fn json_or_error<T: serde::de::DeserializeOwned>(self) -> Result<T> {
        let status = self.status();

        if !status.is_success() {
            let error_body = self.text().await.unwrap_or_default();
            return Err(DxError::Http {
                message: format!("HTTP {}: {}", status.as_u16(), error_body),
                status_code: Some(status.as_u16()),
                source: None,
            });
        }

        self.json::<T>().await.map_err(|e| DxError::JsonParse {
            message: e.to_string(),
            source: None,
        })
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// URL VALIDATION (SSRF Prevention)
// ═══════════════════════════════════════════════════════════════════════════════

/// Validate a URL before making requests.
///
/// This function performs security checks to prevent SSRF (Server-Side Request Forgery)
/// attacks by blocking requests to private/local IP addresses and non-HTTP(S) schemes.
///
/// # Arguments
///
/// * `url` - The URL string to validate.
///
/// # Returns
///
/// Returns `Ok(url::Url)` if the URL is valid and safe to request,
/// or `Err(DxError::InvalidUrl)` if the URL is invalid or points to a private address.
///
/// # Examples
///
/// ```rust
/// use dx_media::http::validate_url;
///
/// // Valid public URL
/// assert!(validate_url("https://example.com/image.jpg").is_ok());
///
/// // Invalid: private IP
/// assert!(validate_url("http://192.168.1.1/secret").is_err());
///
/// // Invalid: localhost
/// assert!(validate_url("http://localhost/admin").is_err());
///
/// // Invalid: non-HTTP scheme
/// assert!(validate_url("file:///etc/passwd").is_err());
/// ```
pub fn validate_url(url: &str) -> Result<url::Url> {
    let parsed = url::Url::parse(url).map_err(|e| DxError::invalid_url(url, e.to_string()))?;

    // Only allow HTTP(S) schemes
    if !matches!(parsed.scheme(), "http" | "https") {
        return Err(DxError::invalid_url(
            url,
            format!(
                "Invalid scheme '{}', only http/https allowed",
                parsed.scheme()
            ),
        ));
    }

    match parsed.host() {
        Some(url::Host::Ipv4(ip)) if is_private_ip(&IpAddr::V4(ip)) => {
            return Err(DxError::invalid_url(
                url,
                "Private/local addresses not allowed",
            ));
        }
        Some(url::Host::Ipv6(ip)) if is_private_ip(&IpAddr::V6(ip)) => {
            return Err(DxError::invalid_url(
                url,
                "Private/local addresses not allowed",
            ));
        }
        Some(url::Host::Domain(host)) if is_private_host(host) => {
            return Err(DxError::invalid_url(
                url,
                "Private/local addresses not allowed",
            ));
        }
        Some(_) => {}
        None => return Err(DxError::invalid_url(url, "URL must have a host")),
    }

    Ok(parsed)
}

/// Resolve and validate an HTTP redirect target before a follow-up request.
///
/// Relative `Location` values are resolved against the current public URL.
/// Absolute targets must pass the same URL safety checks as first-party
/// requests.
pub fn validate_redirect_location(current_url: &str, location: &str) -> Result<url::Url> {
    let current = validate_url(current_url)?;
    let redirected = current.join(location).map_err(|e| {
        DxError::invalid_url(location, format!("Invalid redirect Location header: {e}"))
    })?;

    validate_url(redirected.as_str())
}

/// Resolve a URL host and reject private/local addresses before network fetches.
///
/// `validate_url` catches literal private hosts. This second pass protects
/// direct downloads and redirects from public-looking hostnames that resolve to
/// loopback, private, link-local, multicast, or unspecified addresses.
pub fn validate_url_resolves_public(url: &url::Url) -> Result<()> {
    resolve_public_socket_addrs(url).map(|_| ())
}

fn resolve_public_socket_addrs(url: &url::Url) -> Result<Vec<SocketAddr>> {
    let host = url
        .host_str()
        .ok_or_else(|| DxError::invalid_url(url.as_str(), "URL must have a host"))?;
    let port = url.port_or_known_default().ok_or_else(|| {
        DxError::invalid_url(url.as_str(), "URL must have a known http/https port")
    })?;

    let addresses: Vec<SocketAddr> = (host, port)
        .to_socket_addrs()
        .map_err(|e| DxError::invalid_url(url.as_str(), format!("Failed to resolve host: {e}")))?
        .collect();

    validate_resolved_socket_addrs_public(url.as_str(), &addresses)
}

fn validate_resolved_socket_addrs_public(
    url: &str,
    addresses: &[SocketAddr],
) -> Result<Vec<SocketAddr>> {
    let ips: Vec<IpAddr> = addresses.iter().map(|addr| addr.ip()).collect();
    validate_resolved_ips_public(url, &ips)?;
    Ok(addresses.to_vec())
}

fn validate_resolved_ips_public(url: &str, addresses: &[IpAddr]) -> Result<()> {
    if addresses.is_empty() {
        return Err(DxError::invalid_url(
            url,
            "Host did not resolve to any address",
        ));
    }

    if addresses.iter().any(is_private_ip) {
        return Err(DxError::invalid_url(
            url,
            "Resolved private/local addresses not allowed",
        ));
    }

    Ok(())
}

/// Check if a host is a private/local address.
fn is_private_host(host: &str) -> bool {
    // Check for localhost variants
    if host == "localhost" || host == "127.0.0.1" || host == "::1" {
        return true;
    }

    // Check for common local hostnames
    if host.ends_with(".local") || host.ends_with(".localhost") {
        return true;
    }

    // Check for private IP ranges
    if let Ok(ip) = host.parse::<IpAddr>() {
        return is_private_ip(&ip);
    }

    false
}

/// Check if an IP address is private/local.
fn is_private_ip(ip: &IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => {
            v4.is_private()           // 10.x.x.x, 172.16-31.x.x, 192.168.x.x
                || v4.is_loopback()   // 127.x.x.x
                || v4.is_link_local() // 169.254.x.x
                || v4.is_broadcast()  // 255.255.255.255
                || v4.is_unspecified() // 0.0.0.0
                || v4.octets()[0] == 0 // 0.x.x.x (current network)
        }
        IpAddr::V6(v6) => {
            let segments = v6.segments();
            v6.is_loopback() // ::1
                || v6.is_unspecified() // ::
                || (segments[0] & 0xfe00) == 0xfc00 // fc00::/7 unique local
                || (segments[0] & 0xffc0) == 0xfe80 // fe80::/10 link-local
                || (segments[0] & 0xff00) == 0xff00 // ff00::/8 multicast
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// CONTENT-TYPE VERIFICATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Verify that the content-type matches the expected media type.
///
/// This function checks that the content-type header from a response matches
/// what we expect for the given media type, preventing content-type confusion attacks.
///
/// # Arguments
///
/// * `content_type` - The content-type header value from the response.
/// * `expected` - The expected media type.
///
/// # Returns
///
/// Returns `Ok(())` if the content-type matches, or `Err(DxError::ContentTypeMismatch)`
/// if there's a mismatch.
///
/// # Examples
///
/// ```rust
/// use dx_media::http::verify_content_type;
/// use dx_media::MediaType;
///
/// // Valid: image content-type for image media type
/// assert!(verify_content_type("image/jpeg", MediaType::Image).is_ok());
///
/// // Invalid: HTML content-type for image media type
/// assert!(verify_content_type("text/html", MediaType::Image).is_err());
/// ```
pub fn verify_content_type(content_type: &str, expected: MediaType) -> Result<()> {
    // Extract the main content type (before any parameters like charset)
    let main_type = content_type
        .split(';')
        .next()
        .unwrap_or(content_type)
        .trim()
        .to_ascii_lowercase();

    let valid = match expected {
        MediaType::Image => {
            (main_type.starts_with("image/")
                && main_type != "image/gif"
                && main_type != "image/svg+xml")
                || main_type == "application/octet-stream"
            // Some servers don't set proper type
        }
        MediaType::Video => {
            main_type.starts_with("video/") || main_type == "application/octet-stream"
        }
        MediaType::Audio => {
            main_type.starts_with("audio/") || main_type == "application/octet-stream"
        }
        MediaType::Gif => main_type == "image/gif" || main_type == "application/octet-stream",
        MediaType::Vector => {
            main_type == "image/svg+xml"
                || main_type == "application/postscript"
                || main_type == "application/octet-stream"
        }
        MediaType::Document => {
            main_type == "application/pdf"
                || main_type.starts_with("application/msword")
                || main_type.starts_with("application/vnd.")
                || main_type == "text/plain"
                || main_type == "application/octet-stream"
        }
        MediaType::Data => {
            main_type == "application/json"
                || main_type == "text/csv"
                || main_type == "application/xml"
                || main_type == "text/xml"
                || main_type == "application/octet-stream"
        }
        MediaType::Model3D => {
            main_type == "model/gltf-binary"
                || main_type == "model/gltf+json"
                || main_type == "application/octet-stream"
        }
        MediaType::Code | MediaType::Text => {
            main_type.starts_with("text/")
                || main_type == "application/javascript"
                || main_type == "application/json"
                || main_type == "application/octet-stream"
        }
    };

    if valid {
        Ok(())
    } else {
        Err(DxError::content_type_mismatch(
            expected.as_str(),
            main_type.as_str(),
        ))
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// FILENAME SANITIZATION
// ═══════════════════════════════════════════════════════════════════════════════

/// Sanitize a filename to ensure it's safe for filesystem use.
///
/// This function removes or replaces unsafe characters and ensures the filename
/// meets filesystem requirements.
///
/// # Arguments
///
/// * `filename` - The filename to sanitize.
///
/// # Returns
///
/// A sanitized filename that:
/// - Contains only alphanumeric characters, underscores, hyphens, and periods
/// - Does not start with a period (hidden file)
/// - Is not empty
/// - Is at most 255 characters
///
/// # Examples
///
/// ```rust
/// use dx_media::http::sanitize_filename;
///
/// assert_eq!(sanitize_filename("my file.jpg"), "my_file.jpg");
/// assert_eq!(sanitize_filename("../../../etc/passwd"), "etc_passwd");
/// assert_eq!(sanitize_filename(".hidden"), "hidden");
/// ```
pub fn sanitize_filename(filename: &str) -> String {
    // First, remove path traversal sequences
    let sanitized = filename
        .replace("../", "")
        .replace("..\\", "")
        .replace("..", "");

    // Replace unsafe characters with underscores
    let sanitized: String = sanitized
        .chars()
        .map(|c| {
            if c.is_alphanumeric() || c == '_' || c == '-' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();

    // Remove leading dots (hidden files)
    let sanitized = sanitized.trim_start_matches('.');

    // Remove consecutive underscores
    let mut result = String::with_capacity(sanitized.len());
    let mut last_was_underscore = false;
    for c in sanitized.chars() {
        if c == '_' {
            if !last_was_underscore {
                result.push(c);
            }
            last_was_underscore = true;
        } else {
            result.push(c);
            last_was_underscore = false;
        }
    }

    // Trim leading/trailing underscores
    let result = result.trim_matches('_');

    // Remove leading dots again (in case trimming underscores exposed them)
    let result = result.trim_start_matches('.');

    // Ensure not empty
    let result = if result.is_empty() { "unnamed" } else { result };

    // Truncate to 255 characters
    if result.len() > 255 {
        result[..255].to_string()
    } else {
        result.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_url_valid() {
        assert!(validate_url("https://example.com/image.jpg").is_ok());
        assert!(validate_url("http://api.example.com/v1/search").is_ok());
        assert!(validate_url("https://cdn.example.com:8080/file.png").is_ok());
    }

    #[test]
    fn test_validate_url_invalid_scheme() {
        assert!(validate_url("file:///etc/passwd").is_err());
        assert!(validate_url("ftp://example.com/file").is_err());
        assert!(validate_url("javascript:alert(1)").is_err());
    }

    #[test]
    fn test_validate_url_private_addresses() {
        // Localhost
        assert!(validate_url("http://localhost/admin").is_err());
        assert!(validate_url("http://127.0.0.1/secret").is_err());
        assert!(validate_url("http://[::1]/api").is_err());

        // Private IP ranges
        assert!(validate_url("http://192.168.1.1/").is_err());
        assert!(validate_url("http://10.0.0.1/").is_err());
        assert!(validate_url("http://172.16.0.1/").is_err());

        // Link-local
        assert!(validate_url("http://169.254.1.1/").is_err());
        assert!(validate_url("http://[fc00::1]/").is_err());
        assert!(validate_url("http://[fd12:3456:789a::1]/").is_err());
        assert!(validate_url("http://[fe80::1]/").is_err());
        assert!(validate_url("http://[ff02::1]/").is_err());
    }

    #[test]
    fn resolved_private_addresses_are_rejected() {
        let url = "https://public-looking.example/media.jpg";
        let private = ["127.0.0.1".parse().unwrap(), "10.0.0.4".parse().unwrap()];

        let err = validate_resolved_ips_public(url, &private)
            .expect_err("DNS results that point to private addresses must be rejected");

        assert!(
            err.to_string()
                .contains("Resolved private/local addresses not allowed"),
            "{err}"
        );
    }

    #[test]
    fn resolved_public_addresses_are_allowed() {
        let public = ["93.184.216.34".parse().unwrap()];

        validate_resolved_ips_public("https://example.com/media.jpg", &public)
            .expect("public DNS results should pass SSRF resolution validation");
    }

    #[test]
    fn public_socket_addrs_are_returned_for_request_pinning() {
        let public = [std::net::SocketAddr::from(([93, 184, 216, 34], 443))];

        let resolved =
            validate_resolved_socket_addrs_public("https://example.com/media.jpg", &public)
                .expect("public socket addresses should be usable for reqwest DNS pinning");

        assert_eq!(resolved, public);
    }

    #[test]
    fn redirect_location_rejects_private_target() {
        let err = validate_redirect_location(
            "https://example.com/media/photo.jpg",
            "http://127.0.0.1/admin",
        )
        .expect_err("redirect targets must be revalidated before fetch");

        assert!(err.to_string().contains("Private/local addresses"));
    }

    #[test]
    fn redirect_location_resolves_relative_target() {
        let redirected =
            validate_redirect_location("https://example.com/media/photo.jpg", "../cdn/photo.jpg")
                .expect("relative redirects should resolve against the current URL");

        assert_eq!(redirected.as_str(), "https://example.com/cdn/photo.jpg");
    }

    #[test]
    fn test_verify_content_type_valid() {
        assert!(verify_content_type("image/jpeg", MediaType::Image).is_ok());
        assert!(verify_content_type("image/png; charset=utf-8", MediaType::Image).is_ok());
        assert!(verify_content_type("Image/JPEG", MediaType::Image).is_ok());
        assert!(verify_content_type("IMAGE/PNG; charset=utf-8", MediaType::Image).is_ok());
        assert!(verify_content_type("video/mp4", MediaType::Video).is_ok());
        assert!(verify_content_type("audio/mpeg", MediaType::Audio).is_ok());
        assert!(verify_content_type("image/gif", MediaType::Gif).is_ok());
        assert!(verify_content_type("image/svg+xml", MediaType::Vector).is_ok());
        assert!(verify_content_type("application/postscript", MediaType::Vector).is_ok());
        assert!(verify_content_type("application/pdf", MediaType::Document).is_ok());
    }

    #[test]
    fn test_verify_content_type_invalid() {
        assert!(verify_content_type("text/html", MediaType::Image).is_err());
        assert!(verify_content_type("image/gif", MediaType::Image).is_err());
        assert!(verify_content_type("image/gif; charset=binary", MediaType::Image).is_err());
        assert!(verify_content_type("image/svg+xml", MediaType::Image).is_err());
        assert!(verify_content_type("image/jpeg", MediaType::Vector).is_err());
        assert!(verify_content_type("application/javascript", MediaType::Video).is_err());
    }

    #[test]
    fn test_verify_content_type_octet_stream() {
        // application/octet-stream should be allowed for all types
        assert!(verify_content_type("application/octet-stream", MediaType::Image).is_ok());
        assert!(verify_content_type("application/octet-stream", MediaType::Video).is_ok());
        assert!(verify_content_type("application/octet-stream", MediaType::Audio).is_ok());
    }

    #[test]
    fn test_sanitize_filename_basic() {
        assert_eq!(sanitize_filename("my file.jpg"), "my_file.jpg");
        assert_eq!(sanitize_filename("image.png"), "image.png");
        assert_eq!(sanitize_filename("test-file_123.txt"), "test-file_123.txt");
    }

    #[test]
    fn test_sanitize_filename_path_traversal() {
        assert_eq!(sanitize_filename("../../../etc/passwd"), "etc_passwd");
        assert_eq!(
            sanitize_filename("..\\..\\windows\\system32"),
            "windows_system32"
        );
    }

    #[test]
    fn test_sanitize_filename_hidden() {
        assert_eq!(sanitize_filename(".hidden"), "hidden");
        assert_eq!(sanitize_filename("...dots"), "dots");
    }

    #[test]
    fn test_sanitize_filename_empty() {
        assert_eq!(sanitize_filename(""), "unnamed");
        assert_eq!(sanitize_filename("..."), "unnamed");
        assert_eq!(sanitize_filename("___"), "unnamed");
    }

    #[test]
    fn test_sanitize_filename_special_chars() {
        assert_eq!(sanitize_filename("file<>:\"|?*.jpg"), "file_.jpg");
        assert_eq!(sanitize_filename("hello@world!.png"), "hello_world_.png");
    }

    #[test]
    fn test_sanitize_filename_length() {
        let long_name = "a".repeat(300);
        let sanitized = sanitize_filename(&long_name);
        assert!(sanitized.len() <= 255);
    }

    #[test]
    fn test_sanitize_filename_exposed_dot_after_trim() {
        // Edge case: trimming underscores can expose a leading dot
        // Input "{." -> "_." -> "." after trim -> should become "unnamed"
        assert_eq!(sanitize_filename("{."), "unnamed");
        // Similar cases
        assert_eq!(sanitize_filename("_."), "unnamed");
        assert_eq!(sanitize_filename("__."), "unnamed");
        assert_eq!(sanitize_filename("_.."), "unnamed");
        assert_eq!(sanitize_filename("{..}"), "unnamed");
    }
}
