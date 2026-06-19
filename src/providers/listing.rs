//! Shared provider listing labels for CLI output.

use crate::providers::traits::Provider;

use serde_json::{Value, json};

#[allow(dead_code)]
pub(crate) fn source_kind(_provider: &dyn Provider) -> &'static str {
    "provider-backed"
}

#[allow(dead_code)]
pub(crate) fn credential_status(provider: &dyn Provider) -> &'static str {
    if !provider.requires_api_key() {
        "not-required"
    } else if provider.is_available() {
        "configured"
    } else {
        "missing"
    }
}

#[allow(dead_code)]
pub(crate) fn unavailable_reason(provider: &dyn Provider) -> Value {
    if provider.is_available() {
        Value::Null
    } else if provider.requires_api_key() {
        json!("missing credentials")
    } else {
        json!("provider disabled or unavailable")
    }
}

#[allow(dead_code)]
pub(crate) fn provider_json_row(provider: &dyn Provider, supported_types_key: &str) -> Value {
    let requires_credentials = provider.requires_api_key();
    json!({
        "name": provider.name(),
        "display_name": provider.display_name(),
        "available": provider.is_available(),
        "requires_api_key": requires_credentials,
        "requires_credentials": requires_credentials,
        "credential_status": credential_status(provider),
        "source_kind": source_kind(provider),
        "unavailable_reason": unavailable_reason(provider),
        supported_types_key: provider
            .supported_media_types()
            .iter()
            .map(|media_type| media_type.as_str())
            .collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use async_trait::async_trait;

    use crate::{
        MediaType, SearchQuery, SearchResult,
        error::Result,
        providers::{
            listing::{credential_status, provider_json_row, source_kind, unavailable_reason},
            traits::Provider,
        },
        types::RateLimitConfig,
    };

    struct ListingProvider {
        available: bool,
        requires_api_key: bool,
    }

    #[async_trait]
    impl Provider for ListingProvider {
        fn name(&self) -> &'static str {
            "listing-test"
        }

        fn display_name(&self) -> &'static str {
            "Listing Test"
        }

        fn base_url(&self) -> &'static str {
            "https://example.test"
        }

        fn supported_media_types(&self) -> &[MediaType] {
            &[MediaType::Image, MediaType::Gif]
        }

        fn requires_api_key(&self) -> bool {
            self.requires_api_key
        }

        fn is_available(&self) -> bool {
            self.available
        }

        async fn search(&self, _query: &SearchQuery) -> Result<SearchResult> {
            unreachable!("provider listing tests do not execute provider searches")
        }

        fn rate_limit(&self) -> RateLimitConfig {
            RateLimitConfig::unlimited()
        }
    }

    #[test]
    fn provider_json_row_exposes_source_and_credential_fields_separately() {
        let provider = ListingProvider {
            available: false,
            requires_api_key: true,
        };

        let row = provider_json_row(&provider, "supported_types");

        assert_eq!(row["source_kind"], "provider-backed");
        assert_eq!(row["credential_status"], "missing");
        assert_eq!(row["requires_api_key"], true);
        assert_eq!(row["requires_credentials"], true);
        assert_eq!(row["supported_types"][0], "image");
        assert_eq!(row["supported_types"][1], "gif");
        assert_eq!(row["unavailable_reason"], "missing credentials");
    }

    #[test]
    fn provider_listing_labels_do_not_conflate_credentials_with_provenance() {
        let provider = ListingProvider {
            available: true,
            requires_api_key: false,
        };

        assert_eq!(source_kind(&provider), "provider-backed");
        assert_eq!(credential_status(&provider), "not-required");
        assert!(unavailable_reason(&provider).is_null());
    }
}
