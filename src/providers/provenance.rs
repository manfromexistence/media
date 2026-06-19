use std::collections::HashMap;

use crate::types::{License, MediaType};

pub(crate) const LICENSE_EVIDENCE_NOT_PROVIDED: &str = "not-provided-by-api-response";
pub(crate) const LICENSE_NOT_PROVIDED: &str = "License not provided by provider API response";

pub(crate) fn license_not_provided() -> License {
    License::Other(LICENSE_NOT_PROVIDED.to_string())
}

pub(crate) fn parse_known_license_label(value: &str) -> Option<License> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return None;
    }

    if normalized.contains("publicdomain/zero") || normalized == "cc0" {
        return Some(License::Cc0);
    }

    if normalized.contains("public domain") || normalized.contains("publicdomain/mark") {
        return Some(License::PublicDomain);
    }

    let attribution = normalized.contains("creativecommons.org/licenses/by")
        || normalized.contains("cc-by")
        || normalized.contains("cc by")
        || normalized.contains("creative commons attribution");
    if !attribution {
        return None;
    }

    let non_commercial = normalized.contains("by-nc") || normalized.contains("noncommercial");
    let share_alike = normalized.contains("by-sa") || normalized.contains("sharealike");
    let non_commercial_share_alike =
        normalized.contains("by-nc-sa") || normalized.contains("noncommercial-sharealike");
    let no_derivatives = normalized.contains("by-nd")
        || normalized.contains("by-nc-nd")
        || normalized.contains("noderiv")
        || normalized.contains("no derivatives");

    if no_derivatives || non_commercial_share_alike || (non_commercial && share_alike) {
        return None;
    }

    if non_commercial {
        Some(License::CcByNc)
    } else if share_alike {
        Some(License::CcBySa)
    } else {
        Some(License::CcBy)
    }
}

pub(crate) fn direct_asset_metadata(provider: &str, asset_url: &str) -> HashMap<String, String> {
    HashMap::from([
        (
            format!("{provider}.source_url_kind"),
            "direct-asset-url".to_string(),
        ),
        (format!("{provider}.asset_url"), asset_url.to_string()),
        (
            format!("{provider}.license_evidence"),
            LICENSE_EVIDENCE_NOT_PROVIDED.to_string(),
        ),
    ])
}

pub(crate) fn mime_type_from_url(media_type: MediaType, url: &str) -> Option<&'static str> {
    let path = url.split('?').next().unwrap_or(url).to_ascii_lowercase();
    match path.rsplit('.').next()? {
        "jpg" | "jpeg" if matches!(media_type, MediaType::Image) => Some("image/jpeg"),
        "png" if matches!(media_type, MediaType::Image) => Some("image/png"),
        "webp" if matches!(media_type, MediaType::Image) => Some("image/webp"),
        "avif" if matches!(media_type, MediaType::Image) => Some("image/avif"),
        "bmp" if matches!(media_type, MediaType::Image) => Some("image/bmp"),
        "tif" | "tiff" if matches!(media_type, MediaType::Image) => Some("image/tiff"),
        "gif" if matches!(media_type, MediaType::Gif) => Some("image/gif"),
        "svg" if matches!(media_type, MediaType::Vector) => Some("image/svg+xml"),
        _ => None,
    }
}
