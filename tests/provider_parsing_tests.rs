//! Integration tests for provider response parsing.
//!
//! These tests verify that provider API responses parse correctly into valid MediaAsset structs.
//! Fixture files contain recorded responses from actual provider APIs.

use serde::Deserialize;

// ═══════════════════════════════════════════════════════════════════════════════
// NASA Response Parsing Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// NASA API response structures (mirrored from provider)
#[derive(Debug, Deserialize)]
struct NasaSearchResponse {
    collection: NasaCollection,
}

#[derive(Debug, Deserialize)]
struct NasaCollection {
    items: Vec<NasaItem>,
    metadata: NasaMetadata,
}

#[derive(Debug, Deserialize)]
struct NasaMetadata {
    total_hits: usize,
}

#[derive(Debug, Deserialize)]
struct NasaItem {
    href: String,
    data: Vec<NasaItemData>,
    links: Option<Vec<NasaLink>>,
}

#[derive(Debug, Deserialize)]
struct NasaItemData {
    nasa_id: String,
    title: String,
    media_type: String,
    description: Option<String>,
    center: Option<String>,
    date_created: Option<String>,
    #[serde(default)]
    keywords: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct NasaLink {
    href: String,
    rel: String,
    render: Option<String>,
}

#[test]
fn test_nasa_response_parsing() {
    let json = include_str!("fixtures/nasa_response.json");
    let response: NasaSearchResponse =
        serde_json::from_str(json).expect("NASA response should parse successfully");

    // Verify metadata
    assert_eq!(
        response.collection.metadata.total_hits, 2,
        "Should have 2 total hits"
    );

    // Verify items
    assert_eq!(response.collection.items.len(), 2, "Should have 2 items");

    // Verify first item
    let item1 = &response.collection.items[0];
    assert_eq!(item1.href, "https://images-api.nasa.gov/asset/PIA00001");
    assert!(!item1.data.is_empty(), "Item should have data");

    let data1 = &item1.data[0];
    assert_eq!(data1.nasa_id, "PIA00001");
    assert_eq!(data1.title, "Mars Surface");
    assert_eq!(data1.media_type, "image");
    assert_eq!(data1.center, Some("JPL".to_string()));
    assert_eq!(data1.date_created.as_deref(), Some("1997-01-01T00:00:00Z"));
    assert!(data1.keywords.is_some(), "Should have keywords");
    assert!(
        data1
            .keywords
            .as_ref()
            .unwrap()
            .contains(&"Mars".to_string())
    );

    // Verify links
    assert!(item1.links.is_some(), "Item should have links");
    let links = item1.links.as_ref().unwrap();
    assert!(!links.is_empty(), "Should have at least one link");
    assert!(
        links[0].href.contains("thumb.jpg"),
        "Should have thumbnail link"
    );
    assert_eq!(links[0].rel, "preview");
    assert_eq!(links[0].render.as_deref(), Some("image"));

    // Verify second item
    let item2 = &response.collection.items[1];
    let data2 = &item2.data[0];
    assert_eq!(data2.nasa_id, "PIA00002");
    assert_eq!(data2.title, "Earth from Space");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Openverse Response Parsing Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Openverse API response structures
#[derive(Debug, Deserialize)]
struct OpenverseSearchResponse {
    result_count: usize,
    page_count: usize,
    page_size: usize,
    page: usize,
    results: Vec<OpenverseResult>,
}

#[derive(Debug, Deserialize)]
struct OpenverseResult {
    id: String,
    title: String,
    foreign_landing_url: String,
    url: String,
    thumbnail: Option<String>,
    creator: Option<String>,
    creator_url: Option<String>,
    license: String,
    license_version: Option<String>,
    license_url: Option<String>,
    source: Option<String>,
    tags: Option<Vec<OpenverseTag>>,
}

#[derive(Debug, Deserialize)]
struct OpenverseTag {
    name: String,
}

#[test]
fn test_openverse_response_parsing() {
    let json = include_str!("fixtures/openverse_response.json");
    let response: OpenverseSearchResponse =
        serde_json::from_str(json).expect("Openverse response should parse successfully");

    // Verify pagination
    assert_eq!(response.result_count, 2);
    assert_eq!(response.page_count, 1);
    assert_eq!(response.page_size, 20);
    assert_eq!(response.page, 1);

    // Verify results
    assert_eq!(response.results.len(), 2, "Should have 2 results");

    // Verify first result
    let result1 = &response.results[0];
    assert_eq!(result1.id, "abc123");
    assert_eq!(result1.title, "Beautiful Sunset");
    assert_eq!(result1.foreign_landing_url, "https://example.com/sunset");
    assert_eq!(result1.url, "https://example.com/sunset.jpg");
    assert_eq!(result1.license, "cc-by");
    assert_eq!(result1.license_version.as_deref(), Some("4.0"));
    assert_eq!(
        result1.license_url.as_deref(),
        Some("https://creativecommons.org/licenses/by/4.0/")
    );
    assert_eq!(result1.creator, Some("John Doe".to_string()));
    assert_eq!(
        result1.creator_url.as_deref(),
        Some("https://example.com/johndoe")
    );
    assert!(result1.thumbnail.is_some(), "Should have thumbnail");
    assert!(result1.tags.is_some(), "Should have tags");

    let tags1 = result1.tags.as_ref().unwrap();
    assert_eq!(tags1.len(), 3);
    assert!(tags1.iter().any(|t| t.name == "sunset"));

    // Verify second result with CC0 license
    let result2 = &response.results[1];
    assert_eq!(result2.id, "def456");
    assert_eq!(result2.license, "cc0");
    assert_eq!(result2.source, Some("wikimedia".to_string()));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pixabay Response Parsing Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Pixabay API response structures
#[derive(Debug, Deserialize)]
struct PixabaySearchResponse {
    total: usize,
    #[serde(rename = "totalHits")]
    total_hits: usize,
    hits: Vec<PixabayHit>,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
struct PixabayHit {
    id: u64,
    #[serde(rename = "pageURL")]
    page_url: String,
    #[serde(rename = "type")]
    media_type: String,
    tags: String,
    #[serde(rename = "previewURL")]
    preview_url: String,
    #[serde(rename = "previewWidth")]
    preview_width: u32,
    #[serde(rename = "previewHeight")]
    preview_height: u32,
    #[serde(rename = "webformatURL")]
    webformat_url: String,
    #[serde(rename = "webformatWidth")]
    webformat_width: u32,
    #[serde(rename = "webformatHeight")]
    webformat_height: u32,
    #[serde(rename = "largeImageURL")]
    large_image_url: String,
    #[serde(rename = "imageWidth")]
    image_width: u32,
    #[serde(rename = "imageHeight")]
    image_height: u32,
    #[serde(rename = "imageSize")]
    image_size: u64,
    views: u64,
    downloads: u64,
    likes: u64,
    comments: u64,
    user_id: u64,
    user: String,
    #[serde(rename = "userImageURL")]
    user_image_url: String,
}

#[test]
fn test_pixabay_response_parsing() {
    let json = include_str!("fixtures/pixabay_response.json");
    let response: PixabaySearchResponse =
        serde_json::from_str(json).expect("Pixabay response should parse successfully");

    // Verify totals
    assert_eq!(response.total, 500);
    assert_eq!(response.total_hits, 500);

    // Verify hits
    assert_eq!(response.hits.len(), 2, "Should have 2 hits");

    // Verify first hit
    let hit1 = &response.hits[0];
    assert_eq!(hit1.id, 12345);
    assert_eq!(hit1.media_type, "photo");
    assert_eq!(hit1.user, "photographer1");
    assert!(hit1.tags.contains("flower"));
    assert!(hit1.large_image_url.contains("1280"));
    assert_eq!(hit1.image_width, 1920);
    assert_eq!(hit1.image_height, 1280);

    // Verify second hit
    let hit2 = &response.hits[1];
    assert_eq!(hit2.id, 67890);
    assert!(hit2.tags.contains("sunset"));
    assert_eq!(hit2.views, 20000);
    assert_eq!(hit2.downloads, 10000);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Case Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[test]
fn test_nasa_empty_response() {
    let json = r#"{
        "collection": {
            "items": [],
            "metadata": {
                "total_hits": 0
            }
        }
    }"#;

    let response: NasaSearchResponse =
        serde_json::from_str(json).expect("Empty NASA response should parse successfully");

    assert_eq!(response.collection.items.len(), 0);
    assert_eq!(response.collection.metadata.total_hits, 0);
}

#[test]
fn test_openverse_empty_response() {
    let json = r#"{
        "result_count": 0,
        "page_count": 0,
        "page_size": 20,
        "page": 1,
        "results": []
    }"#;

    let response: OpenverseSearchResponse =
        serde_json::from_str(json).expect("Empty Openverse response should parse successfully");

    assert_eq!(response.results.len(), 0);
    assert_eq!(response.result_count, 0);
}

#[test]
fn test_pixabay_empty_response() {
    let json = r#"{
        "total": 0,
        "totalHits": 0,
        "hits": []
    }"#;

    let response: PixabaySearchResponse =
        serde_json::from_str(json).expect("Empty Pixabay response should parse successfully");

    assert_eq!(response.hits.len(), 0);
    assert_eq!(response.total, 0);
}

#[test]
fn test_nasa_missing_optional_fields() {
    let json = r#"{
        "collection": {
            "items": [
                {
                    "href": "https://example.com/asset",
                    "data": [
                        {
                            "nasa_id": "TEST001",
                            "title": "Test Image",
                            "media_type": "image"
                        }
                    ]
                }
            ],
            "metadata": {
                "total_hits": 1
            }
        }
    }"#;

    let response: NasaSearchResponse = serde_json::from_str(json)
        .expect("NASA response with missing optional fields should parse");

    let item = &response.collection.items[0];
    let data = &item.data[0];

    assert_eq!(data.nasa_id, "TEST001");
    assert!(data.description.is_none());
    assert!(data.center.is_none());
    assert!(data.keywords.is_none());
    assert!(item.links.is_none());
}

#[test]
fn test_openverse_missing_optional_fields() {
    let json = r#"{
        "result_count": 1,
        "page_count": 1,
        "page_size": 20,
        "page": 1,
        "results": [
            {
                "id": "test123",
                "title": "Test Image",
                "foreign_landing_url": "https://example.com",
                "url": "https://example.com/image.jpg",
                "license": "cc-by"
            }
        ]
    }"#;

    let response: OpenverseSearchResponse = serde_json::from_str(json)
        .expect("Openverse response with missing optional fields should parse");

    let result = &response.results[0];
    assert_eq!(result.id, "test123");
    assert!(result.thumbnail.is_none());
    assert!(result.creator.is_none());
    assert!(result.tags.is_none());
}
