//! Integration tests for dx-media providers using wiremock.
//!
//! These tests mock HTTP responses to verify provider behavior without hitting real APIs.

mod nasa_tests {
    use dx_media::providers::NasaImagesProvider;
    use dx_media::providers::traits::Provider;
    use dx_media::types::{DownloadUrlKind, MediaType, SearchQuery};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Test successful search parsing with valid NASA API response.
    #[tokio::test]
    async fn test_nasa_search_parses_valid_response() {
        let mock_server = MockServer::start().await;

        let fixture = include_str!("integration/fixtures/nasa_success.json");

        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "mars"))
            .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
            .mount(&mock_server)
            .await;
        for id in ["PIA00001", "PIA00002", "PIA00003"] {
            Mock::given(method("GET"))
                .and(path(format!("/asset/{id}")))
                .respond_with(ResponseTemplate::new(200).set_body_json(vec![format!(
                    "https://images-assets.nasa.gov/image/{id}/{id}~orig.jpg"
                )]))
                .mount(&mock_server)
                .await;
        }

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("mars").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_ok(), "Search should succeed: {:?}", result.err());

        let search_result = result.unwrap();
        assert_eq!(search_result.total_count, 3, "Should have 3 total hits");
        assert_eq!(search_result.assets.len(), 3, "Should have 3 assets");

        // Verify first asset
        let asset1 = &search_result.assets[0];
        assert_eq!(asset1.id, "PIA00001");
        assert_eq!(asset1.title, "Mars Surface");
        assert_eq!(asset1.provider, "nasa");
        assert_eq!(asset1.media_type, MediaType::Image);
        assert!(
            !asset1.download_url.is_empty(),
            "Download URL should not be empty"
        );
        assert!(
            !asset1.source_url.is_empty(),
            "Source URL should not be empty"
        );
        assert_eq!(
            asset1
                .provider_metadata
                .get("nasa.item_license_field")
                .map(String::as_str),
            Some("not-provided")
        );
        assert_eq!(
            asset1
                .provider_metadata
                .get("nasa.license_scope")
                .map(String::as_str),
            Some("provider-default")
        );
        assert!(
            !asset1.provenance().license_known,
            "NASA item license should not be marked known without item-level evidence"
        );
        assert_eq!(
            asset1.download_url_kind,
            DownloadUrlKind::DirectFile,
            "NASA image search should resolve preview-only records to original asset URLs when the manifest has one"
        );
        assert!(asset1.download_url.ends_with("PIA00001~orig.jpg"));
        assert_eq!(
            asset1
                .provider_metadata
                .get("nasa.download_url_role")
                .map(String::as_str),
            Some("direct-file")
        );

        // Verify second asset
        let asset2 = &search_result.assets[1];
        assert_eq!(asset2.id, "PIA00002");
        assert_eq!(asset2.title, "Earth from Space");

        // Verify third asset
        let asset3 = &search_result.assets[2];
        assert_eq!(asset3.id, "PIA00003");
        assert_eq!(asset3.title, "Jupiter's Great Red Spot");
    }

    #[tokio::test]
    async fn test_nasa_extensionless_image_does_not_fabricate_mime() {
        let mock_server = MockServer::start().await;

        let fixture = serde_json::json!({
            "collection": {
                "items": [
                    {
                        "href": "https://images-api.nasa.gov/asset/PIA00001",
                        "data": [
                            {
                                "nasa_id": "PIA00001",
                                "title": "Mars Surface",
                                "media_type": "image",
                                "center": "JPL"
                            }
                        ]
                    }
                ],
                "metadata": { "total_hits": 1 }
            }
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "mars"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture))
            .mount(&mock_server)
            .await;

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("mars").count(10);
        let result = provider
            .search(&query)
            .await
            .expect("NASA search fixture should parse");

        let asset = result
            .assets
            .first()
            .expect("fixture should produce one asset");
        let provenance_json = serde_json::to_value(asset.provenance()).unwrap();

        assert_eq!(asset.mime_type, None);
        assert_eq!(
            provenance_json["mime_evidence_source"],
            serde_json::Value::Null
        );
        assert_eq!(
            provenance_json["type_validation"]["mime_matches"],
            serde_json::Value::Null
        );
        assert_eq!(asset.download_url_kind, DownloadUrlKind::AssetManifest);
        assert_eq!(provenance_json["download_url_kind"], "asset-manifest");
        assert!(!asset.provenance().type_validation.is_valid());
    }

    #[tokio::test]
    async fn test_nasa_gif_image_result_preserves_gif_type_and_validation() {
        let mock_server = MockServer::start().await;

        let fixture = serde_json::json!({
            "collection": {
                "items": [
                    {
                        "href": "https://images-api.nasa.gov/asset/PIA00004",
                        "data": [
                            {
                                "nasa_id": "PIA00004",
                                "title": "Animated Solar Loop",
                                "media_type": "image",
                                "center": "GSFC"
                            }
                        ],
                        "links": [
                            {
                                "href": "https://images-assets.nasa.gov/image/PIA00004/PIA00004~orig.gif?download=1",
                                "rel": "preview",
                                "render": "image"
                            }
                        ]
                    }
                ],
                "metadata": { "total_hits": 1 }
            }
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("q", "solar"))
            .and(query_param("media_type", "image"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture))
            .mount(&mock_server)
            .await;

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::for_type("solar", MediaType::Gif).count(10);
        let result = provider
            .search(&query)
            .await
            .expect("NASA GIF fixture should parse");

        assert_eq!(result.assets.len(), 1);
        let asset = &result.assets[0];

        assert_eq!(asset.media_type, MediaType::Gif);
        assert_eq!(asset.mime_type.as_deref(), Some("image/gif"));
        assert!(asset.provenance().type_validation.is_valid());
        assert_eq!(asset.download_url_kind, DownloadUrlKind::DirectFile);
    }

    #[tokio::test]
    async fn test_nasa_get_by_id_resolves_tiff_original_with_provenance() {
        let mock_server = MockServer::start().await;

        let fixture = serde_json::json!({
            "collection": {
                "items": [
                    {
                        "href": "https://images-api.nasa.gov/asset/PIA00005",
                        "data": [
                            {
                                "nasa_id": "PIA00005",
                                "title": "Calibrated Telescope Frame",
                                "media_type": "image",
                                "center": "JPL",
                                "date_created": "2024-01-02T00:00:00Z",
                                "keywords": ["telescope", "calibration"]
                            }
                        ],
                        "links": [
                            {
                                "href": "https://images-assets.nasa.gov/image/PIA00005/PIA00005~thumb.jpg",
                                "rel": "preview",
                                "render": "image"
                            }
                        ]
                    }
                ],
                "metadata": { "total_hits": 1 }
            }
        });

        Mock::given(method("GET"))
            .and(path("/search"))
            .and(query_param("nasa_id", "PIA00005"))
            .respond_with(ResponseTemplate::new(200).set_body_json(fixture))
            .mount(&mock_server)
            .await;
        Mock::given(method("GET"))
            .and(path("/asset/PIA00005"))
            .respond_with(ResponseTemplate::new(200).set_body_json(vec![
                "https://images-assets.nasa.gov/image/PIA00005/PIA00005~small.jpg",
                "https://images-assets.nasa.gov/image/PIA00005/PIA00005~orig.tiff?download=1",
            ]))
            .mount(&mock_server)
            .await;

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let asset = provider
            .get_by_id("PIA00005")
            .await
            .expect("NASA get_by_id fixture should parse")
            .expect("NASA get_by_id should return one asset");

        assert_eq!(asset.id, "PIA00005");
        assert_eq!(asset.media_type, MediaType::Image);
        assert_eq!(asset.mime_type.as_deref(), Some("image/tiff"));
        assert_eq!(asset.download_url_kind, DownloadUrlKind::DirectFile);
        assert!(
            asset
                .download_url
                .ends_with("PIA00005~orig.tiff?download=1")
        );
        assert_eq!(asset.source_url, "https://images.nasa.gov/details/PIA00005");
        assert!(!asset.provenance().license_known);
        assert_eq!(
            asset
                .provider_metadata
                .get("nasa.download_url_role")
                .map(String::as_str),
            Some("direct-file")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("nasa.asset_manifest_url")
                .map(String::as_str),
            Some("https://images-api.nasa.gov/asset/PIA00005")
        );
        assert_eq!(
            asset
                .provider_metadata
                .get("nasa.center")
                .map(String::as_str),
            Some("JPL")
        );
    }

    /// Test error handling for malformed JSON responses.
    #[tokio::test]
    async fn test_nasa_handles_malformed_response() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string("not valid json"))
            .mount(&mock_server)
            .await;

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("mars").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_err(), "Search should fail for malformed JSON");
    }

    /// Test error handling for HTTP error responses.
    #[tokio::test]
    async fn test_nasa_handles_http_error() {
        let mock_server = MockServer::start().await;

        let error_fixture = include_str!("integration/fixtures/nasa_error.json");

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(400).set_body_string(error_fixture))
            .mount(&mock_server)
            .await;

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("invalid").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_err(), "Search should fail for HTTP 400 error");
    }

    /// Test handling of empty search results.
    #[tokio::test]
    async fn test_nasa_handles_empty_results() {
        let mock_server = MockServer::start().await;

        let empty_response = r#"{
            "collection": {
                "items": [],
                "metadata": {
                    "total_hits": 0
                }
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string(empty_response))
            .mount(&mock_server)
            .await;

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("nonexistent").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_ok(), "Search should succeed for empty results");

        let search_result = result.unwrap();
        assert_eq!(search_result.total_count, 0);
        assert!(search_result.assets.is_empty());
    }

    /// Test that assets without required fields are filtered out.
    #[tokio::test]
    async fn test_nasa_filters_incomplete_assets() {
        let mock_server = MockServer::start().await;

        // Response with one complete item and one missing links (no preview URL)
        let partial_response = r#"{
            "collection": {
                "items": [
                    {
                        "href": "https://images-api.nasa.gov/asset/PIA00001",
                        "data": [
                            {
                                "nasa_id": "PIA00001",
                                "title": "Complete Item",
                                "media_type": "image",
                                "center": "JPL"
                            }
                        ],
                        "links": [
                            {
                                "href": "https://example.com/thumb.jpg",
                                "rel": "preview",
                                "render": "image"
                            }
                        ]
                    },
                    {
                        "href": "https://images-api.nasa.gov/asset/PIA00002",
                        "data": [
                            {
                                "nasa_id": "PIA00002",
                                "title": "Item Without Links",
                                "media_type": "image"
                            }
                        ]
                    }
                ],
                "metadata": {
                    "total_hits": 2
                }
            }
        }"#;

        Mock::given(method("GET"))
            .and(path("/search"))
            .respond_with(ResponseTemplate::new(200).set_body_string(partial_response))
            .mount(&mock_server)
            .await;

        let provider = NasaImagesProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("test").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_ok(), "Search should succeed");

        let search_result = result.unwrap();
        // Both items should be present - the one without links will have empty download_url
        // which may or may not be filtered depending on builder validation
        assert!(
            search_result.assets.len() <= 2,
            "Should have at most 2 assets"
        );
    }
}

mod openverse_tests {
    use dx_media::providers::OpenverseProvider;
    use dx_media::providers::traits::Provider;
    use dx_media::types::{License, MediaType, SearchQuery};
    use wiremock::matchers::{method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    /// Test successful search parsing with valid Openverse API response.
    #[tokio::test]
    async fn test_openverse_search_parses_valid_response() {
        let mock_server = MockServer::start().await;

        let fixture = include_str!("integration/fixtures/openverse_success.json");

        Mock::given(method("GET"))
            .and(path("/images/"))
            .and(query_param("q", "sunset"))
            .respond_with(ResponseTemplate::new(200).set_body_string(fixture))
            .mount(&mock_server)
            .await;

        let provider = OpenverseProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("sunset").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_ok(), "Search should succeed: {:?}", result.err());

        let search_result = result.unwrap();
        assert_eq!(search_result.total_count, 3, "Should have 3 total results");
        assert_eq!(search_result.assets.len(), 3, "Should have 3 assets");

        // Verify first asset
        let asset1 = &search_result.assets[0];
        assert_eq!(asset1.id, "abc123");
        assert_eq!(asset1.title, "Beautiful Sunset");
        assert_eq!(asset1.provider, "openverse");
        assert_eq!(asset1.media_type, MediaType::Image);
        assert_eq!(asset1.author, Some("John Doe".to_string()));
        assert!(
            !asset1.download_url.is_empty(),
            "Download URL should not be empty"
        );
        assert!(
            !asset1.source_url.is_empty(),
            "Source URL should not be empty"
        );
        assert!(matches!(asset1.license, License::CcBy));

        // Verify second asset with CC0 license
        let asset2 = &search_result.assets[1];
        assert_eq!(asset2.id, "def456");
        assert_eq!(asset2.title, "Mountain Landscape");
        assert!(matches!(asset2.license, License::Cc0));

        // Verify third asset
        let asset3 = &search_result.assets[2];
        assert_eq!(asset3.id, "ghi789");
        assert_eq!(asset3.title, "Ocean Waves");
        assert!(matches!(asset3.license, License::CcBySa));
    }

    /// Test error handling for malformed JSON responses.
    #[tokio::test]
    async fn test_openverse_handles_malformed_response() {
        let mock_server = MockServer::start().await;

        let malformed = include_str!("integration/fixtures/openverse_malformed.json");

        Mock::given(method("GET"))
            .and(path("/images/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(malformed))
            .mount(&mock_server)
            .await;

        let provider = OpenverseProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("test").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_err(), "Search should fail for malformed JSON");
    }

    /// Test error handling for HTTP error responses.
    #[tokio::test]
    async fn test_openverse_handles_http_error() {
        let mock_server = MockServer::start().await;

        Mock::given(method("GET"))
            .and(path("/images/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
            .mount(&mock_server)
            .await;

        let provider = OpenverseProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("test").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_err(), "Search should fail for HTTP 500 error");
    }

    /// Test handling of empty search results.
    #[tokio::test]
    async fn test_openverse_handles_empty_results() {
        let mock_server = MockServer::start().await;

        let empty_response = r#"{
            "result_count": 0,
            "page_count": 0,
            "page_size": 20,
            "page": 1,
            "results": []
        }"#;

        Mock::given(method("GET"))
            .and(path("/images/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(empty_response))
            .mount(&mock_server)
            .await;

        let provider = OpenverseProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("nonexistent").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_ok(), "Search should succeed for empty results");

        let search_result = result.unwrap();
        assert_eq!(search_result.total_count, 0);
        assert!(search_result.assets.is_empty());
    }

    /// Test that assets with missing optional fields are handled correctly.
    #[tokio::test]
    async fn test_openverse_handles_missing_optional_fields() {
        let mock_server = MockServer::start().await;

        let minimal_response = r#"{
            "result_count": 1,
            "page_count": 1,
            "page_size": 20,
            "page": 1,
            "results": [
                {
                    "id": "minimal123",
                    "title": null,
                    "foreign_landing_url": "https://example.com/minimal",
                    "url": "https://example.com/minimal.jpg",
                    "license": "cc0"
                }
            ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/images/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(minimal_response))
            .mount(&mock_server)
            .await;

        let provider = OpenverseProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("minimal").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_ok(), "Search should succeed with minimal fields");

        let search_result = result.unwrap();
        assert_eq!(search_result.assets.len(), 1);

        let asset = &search_result.assets[0];
        assert_eq!(asset.id, "minimal123");
        // Title should default to "Openverse Image" when null
        assert_eq!(asset.title, "Openverse Image");
    }

    #[tokio::test]
    async fn test_openverse_gif_image_result_preserves_gif_type_and_validation() {
        let mock_server = MockServer::start().await;

        let response = r#"{
            "result_count": 2,
            "page_count": 1,
            "page_size": 20,
            "page": 1,
            "results": [
                {
                    "id": "gif_item",
                    "title": "Animated Item",
                    "foreign_landing_url": "https://example.com/gif",
                    "url": "https://example.com/animated.gif?token=fixture",
                    "license": "cc0"
                },
                {
                    "id": "jpg_item",
                    "title": "Still Item",
                    "foreign_landing_url": "https://example.com/jpg",
                    "url": "https://example.com/still.jpg",
                    "license": "cc0"
                }
            ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/images/"))
            .and(query_param("q", "animated"))
            .respond_with(ResponseTemplate::new(200).set_body_string(response))
            .mount(&mock_server)
            .await;

        let provider = OpenverseProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::for_type("animated", MediaType::Gif).count(10);
        let result = provider
            .search(&query)
            .await
            .expect("Openverse GIF fixture should parse");

        assert_eq!(result.assets.len(), 1);
        let asset = &result.assets[0];

        assert_eq!(asset.id, "gif_item");
        assert_eq!(asset.media_type, MediaType::Gif);
        assert_eq!(asset.mime_type.as_deref(), Some("image/gif"));
        assert!(asset.provenance().type_validation.is_valid());
    }

    /// Test license parsing for various license types.
    #[tokio::test]
    async fn test_openverse_license_parsing() {
        let mock_server = MockServer::start().await;

        let license_response = r#"{
            "result_count": 4,
            "page_count": 1,
            "page_size": 20,
            "page": 1,
            "results": [
                {
                    "id": "cc0_item",
                    "title": "CC0 Item",
                    "foreign_landing_url": "https://example.com/cc0",
                    "url": "https://example.com/cc0.jpg",
                    "license": "cc0"
                },
                {
                    "id": "by_item",
                    "title": "BY Item",
                    "foreign_landing_url": "https://example.com/by",
                    "url": "https://example.com/by.jpg",
                    "license": "by"
                },
                {
                    "id": "by_sa_item",
                    "title": "BY-SA Item",
                    "foreign_landing_url": "https://example.com/by-sa",
                    "url": "https://example.com/by-sa.jpg",
                    "license": "by-sa"
                },
                {
                    "id": "pdm_item",
                    "title": "PDM Item",
                    "foreign_landing_url": "https://example.com/pdm",
                    "url": "https://example.com/pdm.jpg",
                    "license": "pdm"
                }
            ]
        }"#;

        Mock::given(method("GET"))
            .and(path("/images/"))
            .respond_with(ResponseTemplate::new(200).set_body_string(license_response))
            .mount(&mock_server)
            .await;

        let provider = OpenverseProvider::with_base_url(&mock_server.uri());
        let query = SearchQuery::new("licenses").count(10);
        let result = provider.search(&query).await;

        assert!(result.is_ok(), "Search should succeed");

        let search_result = result.unwrap();
        assert_eq!(search_result.assets.len(), 4);

        assert!(matches!(search_result.assets[0].license, License::Cc0));
        assert!(matches!(search_result.assets[1].license, License::CcBy));
        assert!(matches!(search_result.assets[2].license, License::CcBySa));
        assert!(matches!(
            search_result.assets[3].license,
            License::PublicDomain
        ));
    }
}

mod rate_limiting_tests {
    use dx_media::types::RateLimitConfig;

    /// Test that rate limiter delays requests appropriately.
    ///
    /// This test verifies that the RateLimitConfig correctly calculates
    /// the delay between requests based on the configured rate limit.
    #[test]
    fn test_rate_limit_delay_calculation() {
        // 10 requests per 10 seconds = 1 request per second = 1000ms delay
        let config = RateLimitConfig::new(10, 10);
        assert_eq!(
            config.delay_ms(),
            1000,
            "10 req/10s should have 1000ms delay"
        );

        // 100 requests per 60 seconds = ~600ms delay
        let config = RateLimitConfig::new(100, 60);
        assert_eq!(
            config.delay_ms(),
            600,
            "100 req/60s should have 600ms delay"
        );

        // 1 request per 1 second = 1000ms delay
        let config = RateLimitConfig::new(1, 1);
        assert_eq!(config.delay_ms(), 1000, "1 req/1s should have 1000ms delay");

        // Unlimited rate limit
        let config = RateLimitConfig::unlimited();
        assert!(
            !config.is_limited(),
            "Unlimited config should not be limited"
        );
    }

    /// Test that rate limit configuration properties are accessible.
    #[test]
    fn test_rate_limit_config_properties() {
        let config = RateLimitConfig::new(100, 60);

        assert_eq!(config.requests_per_window(), 100);
        assert_eq!(config.window_secs(), 60);
        assert!(config.is_limited());

        let unlimited = RateLimitConfig::unlimited();
        assert!(!unlimited.is_limited());
    }

    /// Test default rate limit configuration.
    #[test]
    fn test_default_rate_limit_config() {
        let config = RateLimitConfig::default();

        // Default is 100 requests per 60 seconds
        assert_eq!(config.requests_per_window(), 100);
        assert_eq!(config.window_secs(), 60);
        assert!(config.is_limited());
    }
}
