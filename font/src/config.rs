//! Configuration for dx-font
//!
//! This module provides configuration management with validation for dx-font operations.
//! Use `Config::builder()` for ergonomic construction or `Config::default()` for sensible defaults.

use crate::error::{FontError, FontResult};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Main configuration struct for dx-font operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default output directory for downloaded fonts
    pub output_dir: PathBuf,

    /// Preferred font formats in order of priority
    pub preferred_formats: Vec<String>,

    /// Maximum concurrent downloads
    pub max_concurrent_downloads: usize,

    /// Request timeout in seconds
    pub timeout_seconds: u64,

    /// User agent for HTTP requests
    pub user_agent: String,

    /// Cache directory for API responses
    pub cache_dir: PathBuf,

    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,

    /// Rate limit: requests per second per provider
    pub rate_limit_per_second: f64,

    /// Rate limit: maximum burst size
    pub rate_limit_burst: u32,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Base delay in milliseconds for retry backoff
    pub retry_base_delay_ms: u64,
}

impl Config {
    /// Create a new ConfigBuilder for ergonomic configuration construction
    pub fn builder() -> ConfigBuilder {
        ConfigBuilder::default()
    }

    /// Validate all configuration fields
    ///
    /// Returns `Ok(())` if all fields have valid values, or `Err(FontError::Validation)`
    /// if any field is invalid.
    ///
    /// # Validation Rules
    /// - `timeout_seconds` must be greater than 0
    /// - `rate_limit_per_second` must be positive
    /// - `rate_limit_burst` must be greater than 0
    /// - `max_retries` must be at least 1
    /// - `max_concurrent_downloads` must be greater than 0
    /// - `retry_base_delay_ms` must be greater than 0
    pub fn validate(&self) -> FontResult<()> {
        if self.timeout_seconds == 0 {
            return Err(FontError::validation(
                "timeout_seconds must be greater than 0",
            ));
        }

        if self.rate_limit_per_second <= 0.0 {
            return Err(FontError::validation(
                "rate_limit_per_second must be positive",
            ));
        }

        if self.rate_limit_burst == 0 {
            return Err(FontError::validation(
                "rate_limit_burst must be greater than 0",
            ));
        }

        if self.max_retries == 0 {
            return Err(FontError::validation("max_retries must be at least 1"));
        }

        if self.max_concurrent_downloads == 0 {
            return Err(FontError::validation(
                "max_concurrent_downloads must be greater than 0",
            ));
        }

        if self.retry_base_delay_ms == 0 {
            return Err(FontError::validation(
                "retry_base_delay_ms must be greater than 0",
            ));
        }

        // Validate output_dir is writable (create if needed)
        if let Err(e) = std::fs::create_dir_all(&self.output_dir) {
            return Err(FontError::validation(format!(
                "output_dir '{}' is not writable: {}",
                self.output_dir.display(),
                e
            )));
        }

        // Validate cache_dir is writable (create if needed)
        if let Err(e) = std::fs::create_dir_all(&self.cache_dir) {
            return Err(FontError::validation(format!(
                "cache_dir '{}' is not writable: {}",
                self.cache_dir.display(),
                e
            )));
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            output_dir: PathBuf::from("./fonts"),
            preferred_formats: vec![
                "woff2".to_string(),
                "woff".to_string(),
                "ttf".to_string(),
                "otf".to_string(),
            ],
            max_concurrent_downloads: 5,
            timeout_seconds: 30,
            user_agent: format!("dx-font/{}", env!("CARGO_PKG_VERSION")),
            cache_dir: dirs::cache_dir()
                .unwrap_or_else(|| PathBuf::from("."))
                .join("dx-font"),
            cache_ttl_seconds: 3600, // 1 hour
            rate_limit_per_second: 10.0,
            rate_limit_burst: 20,
            max_retries: 3,
            retry_base_delay_ms: 1000,
        }
    }
}

/// Builder for ergonomic Config construction
#[derive(Debug, Clone, Default)]
pub struct ConfigBuilder {
    config: Config,
}

impl ConfigBuilder {
    /// Create a new ConfigBuilder with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the output directory for downloaded fonts
    pub fn output_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.output_dir = path.into();
        self
    }

    /// Set the preferred font formats in order of priority
    pub fn preferred_formats(mut self, formats: Vec<String>) -> Self {
        self.config.preferred_formats = formats;
        self
    }

    /// Set the maximum number of concurrent downloads
    pub fn max_concurrent_downloads(mut self, count: usize) -> Self {
        self.config.max_concurrent_downloads = count;
        self
    }

    /// Set the request timeout in seconds
    pub fn timeout_seconds(mut self, seconds: u64) -> Self {
        self.config.timeout_seconds = seconds;
        self
    }

    /// Set the user agent for HTTP requests
    pub fn user_agent(mut self, agent: impl Into<String>) -> Self {
        self.config.user_agent = agent.into();
        self
    }

    /// Set the cache directory for API responses
    pub fn cache_dir(mut self, path: impl Into<PathBuf>) -> Self {
        self.config.cache_dir = path.into();
        self
    }

    /// Set the cache TTL in seconds
    pub fn cache_ttl_seconds(mut self, seconds: u64) -> Self {
        self.config.cache_ttl_seconds = seconds;
        self
    }

    /// Set the rate limit in requests per second per provider
    pub fn rate_limit_per_second(mut self, rate: f64) -> Self {
        self.config.rate_limit_per_second = rate;
        self
    }

    /// Set the rate limit burst size
    pub fn rate_limit_burst(mut self, burst: u32) -> Self {
        self.config.rate_limit_burst = burst;
        self
    }

    /// Set the maximum number of retries for failed requests
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Set the base delay in milliseconds for retry backoff
    pub fn retry_base_delay_ms(mut self, delay_ms: u64) -> Self {
        self.config.retry_base_delay_ms = delay_ms;
        self
    }

    /// Build the Config, validating all fields
    ///
    /// Returns `Ok(Config)` if validation passes, or `Err(FontError::Validation)` if any field is invalid.
    pub fn build(self) -> FontResult<Config> {
        self.config.validate()?;
        Ok(self.config)
    }

    /// Build the Config without validation
    ///
    /// Use this if you want to defer validation or handle it separately.
    pub fn build_unchecked(self) -> Config {
        self.config
    }
}

/// Font source URLs - all free, commercial-use fonts
pub mod sources {
    /// Tier 1: Primary APIs (No Keys Required)
    pub mod primary {
        pub const GOOGLE_FONTS: &str = "https://fonts.google.com";
        pub const GOOGLE_FONTS_API: &str = "https://www.googleapis.com/webfonts/v1/webfonts";
        pub const BUNNY_FONTS: &str = "https://fonts.bunny.net";
        pub const BUNNY_FONTS_API: &str = "https://fonts.bunny.net/list";
        pub const GOOGLE_WEBFONTS_HELPER: &str = "https://gwfh.mranftl.com/api/fonts";
        pub const FONTSOURCE_API: &str = "https://api.fontsource.org/v1/fonts";
        pub const FONT_LIBRARY: &str = "https://fontlibrary.org";
    }

    /// Tier 2: Major Free Font Sites
    pub mod major_sites {
        pub const FONT_SQUIRREL: &str = "https://www.fontsquirrel.com";
        pub const DAFONT: &str = "https://www.dafont.com";
        pub const FONTS_1001: &str = "https://www.1001fonts.com/free-fonts-for-commercial-use";
        pub const FONTSPACE: &str = "https://www.fontspace.com/category/open-source";
        pub const ABSTRACT_FONTS: &str = "https://www.abstractfonts.com";
        pub const URBAN_FONTS: &str = "https://www.urbanfonts.com/free-fonts.htm";
        pub const FONT_ZONE: &str = "https://fontzone.net";
        pub const FFONTS: &str = "https://www.ffonts.net";
        pub const FONT_MEME: &str = "https://fontmeme.com/fonts";
        pub const FONT_RIVER: &str = "https://www.fontriver.com";
    }

    /// Tier 3: Curated Foundries (High Quality)
    pub mod curated {
        pub const FONTSHARE: &str = "https://www.fontshare.com";
        pub const FONTSHARE_API: &str = "https://api.fontshare.com/v2/fonts";
        pub const VELVETYNE: &str = "https://velvetyne.fr";
        pub const OPEN_FOUNDRY: &str = "https://open-foundry.com";
        pub const LEAGUE_OF_MOVEABLE_TYPE: &str = "https://www.theleagueofmoveabletype.com";
        pub const UNCUT: &str = "https://uncut.wtf";
        pub const COLLLETTTIVO: &str = "https://www.collletttivo.it";
        pub const OMNIBUS_TYPE: &str = "https://www.omnibus-type.com";
        pub const FREE_FACES_GALLERY: &str = "https://www.freefaces.gallery";
        pub const USE_MODIFY: &str = "https://usemodify.com";
        pub const BEAUTIFUL_WEB_TYPE: &str = "https://beautifulwebtype.com";
        pub const FONTAIN: &str = "https://fontain.org";
        pub const GOOD_FONTS: &str = "https://goodfonts.io";
        pub const BEFONTS: &str = "https://befonts.com";
        pub const LOST_TYPE: &str = "https://www.losttype.com";
        pub const ATIPO_FOUNDRY: &str = "https://www.atipofoundry.com";
    }

    /// Tier 4: GitHub Repositories
    pub mod github {
        pub const GOOGLE_FONTS_REPO: &str = "https://github.com/google/fonts";
        pub const FONTSOURCE_REPO: &str = "https://github.com/fontsource/fontsource";
        pub const ADOBE_FONTS: &str = "https://github.com/adobe-fonts";
        pub const NOTO_FONTS: &str = "https://github.com/notofonts";
        pub const MOZILLA_FIRA: &str = "https://github.com/mozilla/Fira";
        pub const IBM_PLEX: &str = "https://github.com/IBM/plex";
        pub const INTER: &str = "https://github.com/rsms/inter";
        pub const JETBRAINS_MONO: &str = "https://github.com/JetBrains/JetBrainsMono";
        pub const CASCADIA_CODE: &str = "https://github.com/microsoft/cascadia-code";
        pub const FIRA_CODE: &str = "https://github.com/tonsky/FiraCode";
        pub const VICTOR_MONO: &str = "https://github.com/rubjo/victor-mono";
        pub const HACK: &str = "https://github.com/source-foundry/Hack";
        pub const IOSEVKA: &str = "https://github.com/be5invis/Iosevka";
        pub const RECURSIVE: &str = "https://github.com/arrowtype/recursive";
        pub const MANROPE: &str = "https://github.com/sharanda/manrope";
        pub const PUBLIC_SANS: &str = "https://github.com/uswds/public-sans";
        pub const WORK_SANS: &str = "https://github.com/weiweihuanghuang/Work-Sans";
        pub const OVERPASS: &str = "https://github.com/RedHatOfficial/Overpass";
        pub const LEXEND: &str = "https://github.com/googlefonts/lexend";
        pub const ATKINSON_HYPERLEGIBLE: &str =
            "https://github.com/googlefonts/atkinson-hyperlegible";
        pub const MONONOKI: &str = "https://github.com/madmalik/mononoki";
        pub const FANTASQUE_SANS: &str = "https://github.com/belluzj/fantasque-sans";
        pub const MONOID: &str = "https://github.com/larsenwork/monoid";
        pub const HASKLIG: &str = "https://github.com/i-tu/Hasklig";
        pub const LIBERATION_FONTS: &str = "https://github.com/liberationfonts/liberation-fonts";
    }

    /// Tier 5: International/Multi-Language
    pub mod international {
        pub const NOTO_FONTS: &str = "https://fonts.google.com/noto";
        pub const ARABIC_FONTS: &str = "https://arabicfonts.net";
        pub const CHINAZ_FONTS: &str = "https://font.chinaz.com";
        pub const FREE_JAPANESE_FONTS: &str = "https://freejapanesefont.com";
        pub const NOONNU: &str = "https://noonnu.cc";
        pub const HINDI_FONTS: &str = "https://hindityping.com/fonts";
        pub const THAI_FONTS: &str = "https://f0nt.com";
        pub const FONTER_RU: &str = "https://fonter.ru";
        pub const FONTS_IR: &str = "https://fonts.ir";
        pub const TAMIL_FONTS: &str = "https://tamilfonts.net";
        pub const BENGALI_FONTS: &str = "https://banglafonts.net";
        pub const SMC_MALAYALAM: &str = "https://smc.org.in/fonts";
    }

    /// CDN Direct Access URLs
    pub mod cdn {
        pub const JSDELIVR_FONTSOURCE: &str = "https://cdn.jsdelivr.net/npm/@fontsource";
        pub const UNPKG_FONTSOURCE: &str = "https://unpkg.com/@fontsource";
        pub const BUNNY_FONTS_CDN: &str = "https://fonts.bunny.net/css";
        pub const GOOGLE_FONTS_CDN: &str = "https://fonts.googleapis.com/css2";
        pub const GITHUB_RAW: &str = "https://raw.githubusercontent.com/google/fonts/main/ofl";
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_is_valid() {
        // Default config should produce valid values (except for directory creation in tests)
        let config = Config::default();

        // Check all numeric fields are valid
        assert!(config.timeout_seconds > 0);
        assert!(config.rate_limit_per_second > 0.0);
        assert!(config.rate_limit_burst > 0);
        assert!(config.max_retries > 0);
        assert!(config.max_concurrent_downloads > 0);
        assert!(config.retry_base_delay_ms > 0);
    }

    #[test]
    fn test_config_builder() {
        let temp_dir = tempdir().unwrap();
        let output_dir = temp_dir.path().join("fonts");
        let cache_dir = temp_dir.path().join("cache");

        let config = Config::builder()
            .output_dir(&output_dir)
            .cache_dir(&cache_dir)
            .timeout_seconds(60)
            .rate_limit_per_second(5.0)
            .rate_limit_burst(10)
            .max_retries(5)
            .retry_base_delay_ms(500)
            .build()
            .unwrap();

        assert_eq!(config.output_dir, output_dir);
        assert_eq!(config.cache_dir, cache_dir);
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(config.rate_limit_per_second, 5.0);
        assert_eq!(config.rate_limit_burst, 10);
        assert_eq!(config.max_retries, 5);
        assert_eq!(config.retry_base_delay_ms, 500);
    }

    #[test]
    fn test_validation_rejects_zero_timeout() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .timeout_seconds(0)
            .build_unchecked();

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("timeout_seconds"));
    }

    #[test]
    fn test_validation_rejects_zero_rate_limit() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .rate_limit_per_second(0.0)
            .build_unchecked();

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("rate_limit_per_second"));
    }

    #[test]
    fn test_validation_rejects_negative_rate_limit() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .rate_limit_per_second(-1.0)
            .build_unchecked();

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("rate_limit_per_second"));
    }

    #[test]
    fn test_validation_rejects_zero_max_retries() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .max_retries(0)
            .build_unchecked();

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("max_retries"));
    }

    #[test]
    fn test_validation_rejects_zero_rate_limit_burst() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .rate_limit_burst(0)
            .build_unchecked();

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("rate_limit_burst"));
    }

    #[test]
    fn test_validation_rejects_zero_concurrent_downloads() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .max_concurrent_downloads(0)
            .build_unchecked();

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("max_concurrent_downloads"));
    }

    #[test]
    fn test_validation_rejects_zero_retry_delay() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .retry_base_delay_ms(0)
            .build_unchecked();

        let result = config.validate();
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("retry_base_delay_ms"));
    }

    #[test]
    fn test_valid_config_passes_validation() {
        let temp_dir = tempdir().unwrap();
        let config = Config::builder()
            .output_dir(temp_dir.path().join("fonts"))
            .cache_dir(temp_dir.path().join("cache"))
            .timeout_seconds(30)
            .rate_limit_per_second(10.0)
            .rate_limit_burst(20)
            .max_retries(3)
            .max_concurrent_downloads(5)
            .retry_base_delay_ms(1000)
            .build();

        assert!(config.is_ok());
    }

    #[test]
    fn test_builder_build_unchecked() {
        let config = Config::builder()
            .timeout_seconds(0) // Invalid but should not fail
            .build_unchecked();

        assert_eq!(config.timeout_seconds, 0);
    }
}

#[cfg(test)]
mod property_tests {
    use super::*;
    use proptest::prelude::*;
    use tempfile::tempdir;

    // Feature: dx-font-production-ready, Property 9: Config Validation Completeness
    // **Validates: Requirements 11.1, 11.2, 11.4**
    //
    // For any Config instance, calling validate() SHALL:
    // - Return Ok(()) if and only if all fields have valid values
    // - Return Err(FontError::Validation) if any field is invalid
    // - Specifically reject: timeout=0, rate_limit<=0, max_retries=0

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(100))]

        #[test]
        fn valid_config_passes_validation(
            timeout in 1u64..3600u64,
            rate_limit in 0.1f64..100.0f64,
            burst in 1u32..100u32,
            retries in 1u32..10u32,
            concurrent in 1usize..20usize,
            delay_ms in 1u64..10000u64
        ) {
            let temp_dir = tempdir().unwrap();
            let config = Config::builder()
                .output_dir(temp_dir.path().join("fonts"))
                .cache_dir(temp_dir.path().join("cache"))
                .timeout_seconds(timeout)
                .rate_limit_per_second(rate_limit)
                .rate_limit_burst(burst)
                .max_retries(retries)
                .max_concurrent_downloads(concurrent)
                .retry_base_delay_ms(delay_ms)
                .build_unchecked();

            let result = config.validate();
            prop_assert!(
                result.is_ok(),
                "Valid config should pass validation: {:?}",
                result
            );
        }

        #[test]
        fn zero_timeout_fails_validation(
            rate_limit in 0.1f64..100.0f64,
            burst in 1u32..100u32,
            retries in 1u32..10u32
        ) {
            let temp_dir = tempdir().unwrap();
            let config = Config::builder()
                .output_dir(temp_dir.path().join("fonts"))
                .cache_dir(temp_dir.path().join("cache"))
                .timeout_seconds(0)
                .rate_limit_per_second(rate_limit)
                .rate_limit_burst(burst)
                .max_retries(retries)
                .build_unchecked();

            let result = config.validate();
            prop_assert!(result.is_err(), "Zero timeout should fail validation");

            if let Err(FontError::Validation { message }) = result {
                prop_assert!(
                    message.contains("timeout"),
                    "Error message should mention timeout: {}",
                    message
                );
            } else {
                prop_assert!(false, "Expected Validation error");
            }
        }

        #[test]
        fn non_positive_rate_limit_fails_validation(
            rate_limit in -100.0f64..=0.0f64,
            timeout in 1u64..3600u64,
            burst in 1u32..100u32,
            retries in 1u32..10u32
        ) {
            let temp_dir = tempdir().unwrap();
            let config = Config::builder()
                .output_dir(temp_dir.path().join("fonts"))
                .cache_dir(temp_dir.path().join("cache"))
                .timeout_seconds(timeout)
                .rate_limit_per_second(rate_limit)
                .rate_limit_burst(burst)
                .max_retries(retries)
                .build_unchecked();

            let result = config.validate();
            prop_assert!(result.is_err(), "Non-positive rate limit should fail validation");

            if let Err(FontError::Validation { message }) = result {
                prop_assert!(
                    message.contains("rate_limit"),
                    "Error message should mention rate_limit: {}",
                    message
                );
            } else {
                prop_assert!(false, "Expected Validation error");
            }
        }

        #[test]
        fn zero_max_retries_fails_validation(
            timeout in 1u64..3600u64,
            rate_limit in 0.1f64..100.0f64,
            burst in 1u32..100u32
        ) {
            let temp_dir = tempdir().unwrap();
            let config = Config::builder()
                .output_dir(temp_dir.path().join("fonts"))
                .cache_dir(temp_dir.path().join("cache"))
                .timeout_seconds(timeout)
                .rate_limit_per_second(rate_limit)
                .rate_limit_burst(burst)
                .max_retries(0)
                .build_unchecked();

            let result = config.validate();
            prop_assert!(result.is_err(), "Zero max_retries should fail validation");

            if let Err(FontError::Validation { message }) = result {
                prop_assert!(
                    message.contains("max_retries"),
                    "Error message should mention max_retries: {}",
                    message
                );
            } else {
                prop_assert!(false, "Expected Validation error");
            }
        }

        #[test]
        fn zero_rate_limit_burst_fails_validation(
            timeout in 1u64..3600u64,
            rate_limit in 0.1f64..100.0f64,
            retries in 1u32..10u32
        ) {
            let temp_dir = tempdir().unwrap();
            let config = Config::builder()
                .output_dir(temp_dir.path().join("fonts"))
                .cache_dir(temp_dir.path().join("cache"))
                .timeout_seconds(timeout)
                .rate_limit_per_second(rate_limit)
                .rate_limit_burst(0)
                .max_retries(retries)
                .build_unchecked();

            let result = config.validate();
            prop_assert!(result.is_err(), "Zero rate_limit_burst should fail validation");

            if let Err(FontError::Validation { message }) = result {
                prop_assert!(
                    message.contains("rate_limit_burst"),
                    "Error message should mention rate_limit_burst: {}",
                    message
                );
            } else {
                prop_assert!(false, "Expected Validation error");
            }
        }
    }
}
