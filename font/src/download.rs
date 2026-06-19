//! Font download functionality
//!
//! Handles downloading fonts from various providers with progress indication
//! and file verification.

use futures::StreamExt;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use reqwest::Client;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::fs::{self, File};
use tokio::io::AsyncWriteExt;
use tracing::{Instrument, info_span, instrument};

use crate::error::{FontError, FontResult};
use crate::models::{DownloadOptions, FontProvider};
use crate::providers::{ProviderRegistry, create_http_client};
use crate::verify::FileVerifier;

/// Font downloader with progress indication
pub struct FontDownloader {
    client: Client,
    registry: Arc<ProviderRegistry>,
    multi_progress: MultiProgress,
}

impl FontDownloader {
    /// Create a new font downloader
    pub fn new() -> FontResult<Self> {
        let client = create_http_client()?;
        let registry = ProviderRegistry::with_defaults()?;

        Ok(Self {
            client,
            registry: Arc::new(registry),
            multi_progress: MultiProgress::new(),
        })
    }

    /// Download a font by ID from a specific provider
    #[instrument(skip(self, options), fields(provider = %provider.name(), font_id = %font_id))]
    pub async fn download_font(
        &self,
        provider: &FontProvider,
        font_id: &str,
        options: &DownloadOptions,
    ) -> FontResult<Vec<PathBuf>> {
        tracing::info!(
            provider = %provider.name(),
            font_id = %font_id,
            output_dir = %options.output_dir.display(),
            "Starting font download"
        );

        // Ensure output directory exists
        fs::create_dir_all(&options.output_dir).await.map_err(|e| {
            FontError::download(font_id, format!("Failed to create output directory: {}", e))
        })?;

        // Get download URL from provider
        let download_url = self.get_download_url(provider, font_id).await?;

        // Download the font
        let result = self
            .download_file(&download_url, &options.output_dir, font_id)
            .instrument(info_span!("download_file", url = %download_url))
            .await;

        match &result {
            Ok(paths) => {
                tracing::info!(
                    provider = %provider.name(),
                    font_id = %font_id,
                    files_downloaded = paths.len(),
                    "Download completed successfully"
                );
            }
            Err(e) => {
                tracing::warn!(
                    provider = %provider.name(),
                    font_id = %font_id,
                    error = %e,
                    "Download failed"
                );
            }
        }

        result
    }

    /// Download a font using a direct URL
    #[instrument(skip(self), fields(url = %url, filename = %filename))]
    pub async fn download_from_url(
        &self,
        url: &str,
        output_dir: &Path,
        filename: &str,
    ) -> FontResult<PathBuf> {
        tracing::info!(url = %url, filename = %filename, "Starting direct URL download");

        fs::create_dir_all(output_dir).await.map_err(|e| {
            FontError::download(
                filename,
                format!("Failed to create output directory: {}", e),
            )
        })?;

        let paths = self.download_file(url, output_dir, filename).await?;
        let result = paths
            .into_iter()
            .next()
            .ok_or_else(|| FontError::download(filename, "No files downloaded"));

        match &result {
            Ok(path) => {
                tracing::info!(filename = %filename, path = %path.display(), "Direct URL download completed");
            }
            Err(e) => {
                tracing::warn!(filename = %filename, error = %e, "Direct URL download failed");
            }
        }

        result
    }

    /// Download using Google Webfonts Helper (provides zip with all formats)
    #[instrument(skip(self), fields(font_id = %font_id))]
    pub async fn download_google_font(
        &self,
        font_id: &str,
        output_dir: &Path,
        formats: &[&str],
        subsets: &[&str],
    ) -> FontResult<PathBuf> {
        fs::create_dir_all(output_dir).await.map_err(|e| {
            FontError::download(font_id, format!("Failed to create output directory: {}", e))
        })?;

        let formats_str = formats.join(",");
        let subsets_str = subsets.join(",");

        let url = format!(
            "https://gwfh.mranftl.com/api/fonts/{}?download=zip&subsets={}&formats={}",
            font_id, subsets_str, formats_str
        );

        let output_path = output_dir.join(format!("{}.zip", font_id));

        let pb = self.create_progress_bar(font_id);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| FontError::network(&url, e))?;

        if !response.status().is_success() {
            return Err(FontError::download(
                font_id,
                format!("HTTP {}", response.status()),
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        pb.set_length(total_size);

        let mut file = File::create(&output_path)
            .await
            .map_err(|e| FontError::download(font_id, format!("Failed to create file: {}", e)))?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| FontError::download(font_id, format!("Error reading chunk: {}", e)))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| FontError::download(font_id, format!("Error writing chunk: {}", e)))?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        // Ensure file is flushed before verification
        drop(file);

        // Verify the downloaded zip file
        if let Err(e) = FileVerifier::verify_and_cleanup(&output_path, "zip") {
            pb.finish_with_message(format!("Verification failed: {}", font_id));
            return Err(e);
        }

        pb.finish_with_message(format!("Downloaded and verified {}", font_id));

        // Auto-extract zip file
        let extract_dir = output_dir.join(font_id);
        if let Err(e) = self.extract_zip(&output_path, &extract_dir).await {
            tracing::warn!(
                font_id = %font_id,
                error = %e,
                "Failed to extract zip, returning zip file path"
            );
            return Ok(output_path);
        }

        // Remove the zip file after successful extraction
        if let Err(e) = fs::remove_file(&output_path).await {
            tracing::warn!(
                font_id = %font_id,
                error = %e,
                "Failed to remove zip file after extraction"
            );
        }

        Ok(extract_dir)
    }

    /// Download font from Fontsource via CDN
    #[instrument(skip(self), fields(font_id = %font_id, weight = %weight, style = %style))]
    pub async fn download_fontsource_font(
        &self,
        font_id: &str,
        output_dir: &Path,
        weight: u16,
        style: &str,
    ) -> FontResult<PathBuf> {
        fs::create_dir_all(output_dir).await.map_err(|e| {
            FontError::download(font_id, format!("Failed to create output directory: {}", e))
        })?;

        let url = format!(
            "https://cdn.jsdelivr.net/npm/@fontsource/{}/files/{}-latin-{}-{}.woff2",
            font_id, font_id, weight, style
        );

        let filename = format!("{}-{}-{}.woff2", font_id, weight, style);
        let output_path = output_dir.join(&filename);

        let pb = self.create_progress_bar(&filename);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| FontError::network(&url, e))?;

        if !response.status().is_success() {
            pb.finish_with_message(format!("Failed: {}", filename));
            return Err(FontError::download(
                font_id,
                format!("HTTP {}", response.status()),
            ));
        }

        let total_size = response.content_length().unwrap_or(0);
        pb.set_length(total_size);

        let mut file = File::create(&output_path)
            .await
            .map_err(|e| FontError::download(font_id, format!("Failed to create file: {}", e)))?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| FontError::download(font_id, format!("Error reading chunk: {}", e)))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| FontError::download(font_id, format!("Error writing chunk: {}", e)))?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        // Ensure file is flushed before verification
        drop(file);

        // Verify the downloaded woff2 file
        if let Err(e) = FileVerifier::verify_and_cleanup(&output_path, "woff2") {
            pb.finish_with_message(format!("Verification failed: {}", filename));
            return Err(e);
        }

        pb.finish_with_message(format!("Downloaded and verified {}", filename));

        Ok(output_path)
    }

    async fn get_download_url(&self, provider: &FontProvider, font_id: &str) -> FontResult<String> {
        for p in self.registry.providers() {
            if p.name() == provider.name() {
                return p.get_download_url(font_id).await;
            }
        }

        Err(FontError::provider(
            provider.name(),
            format!("Provider not found: {:?}", provider),
        ))
    }

    async fn download_file(
        &self,
        url: &str,
        output_dir: &Path,
        name: &str,
    ) -> FontResult<Vec<PathBuf>> {
        let pb = self.create_progress_bar(name);

        let response = self
            .client
            .get(url)
            .send()
            .await
            .map_err(|e| FontError::network(url, e))?;

        if !response.status().is_success() {
            pb.finish_with_message(format!("Failed: {}", name));
            return Err(FontError::download(
                name,
                format!("HTTP {}", response.status()),
            ));
        }

        // Determine file extension from content-type or URL
        let extension = self.get_extension_from_response(&response, url);
        let filename = format!("{}.{}", name, extension);
        let output_path = output_dir.join(&filename);

        let total_size = response.content_length().unwrap_or(0);
        pb.set_length(total_size);

        let mut file = File::create(&output_path)
            .await
            .map_err(|e| FontError::download(name, format!("Failed to create file: {}", e)))?;
        let mut stream = response.bytes_stream();
        let mut downloaded = 0u64;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk
                .map_err(|e| FontError::download(name, format!("Error reading chunk: {}", e)))?;
            file.write_all(&chunk)
                .await
                .map_err(|e| FontError::download(name, format!("Error writing chunk: {}", e)))?;
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }

        // Ensure file is flushed before verification
        drop(file);

        // Verify the downloaded file
        if let Err(e) = FileVerifier::verify_and_cleanup(&output_path, &extension) {
            pb.finish_with_message(format!("Verification failed: {}", name));
            return Err(e);
        }

        pb.finish_with_message(format!("Downloaded and verified {}", filename));

        // Auto-extract if it's a ZIP file
        if extension == "zip" {
            let extract_dir = output_dir.join(name);
            if let Err(e) = self.extract_zip(&output_path, &extract_dir).await {
                tracing::warn!(
                    name = %name,
                    error = %e,
                    "Failed to extract zip, returning zip file path"
                );
                return Ok(vec![output_path]);
            }

            // Remove the zip file after successful extraction
            if let Err(e) = fs::remove_file(&output_path).await {
                tracing::warn!(
                    name = %name,
                    error = %e,
                    "Failed to remove zip file after extraction"
                );
            }

            // Return all extracted font files
            let mut extracted_files = Vec::new();
            if let Ok(mut entries) = fs::read_dir(&extract_dir).await {
                while let Ok(Some(entry)) = entries.next_entry().await {
                    extracted_files.push(entry.path());
                }
            }

            return Ok(extracted_files);
        }

        Ok(vec![output_path])
    }

    fn create_progress_bar(&self, name: &str) -> ProgressBar {
        let pb = self.multi_progress.add(ProgressBar::new(0));
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        pb.set_message(format!("Downloading {}", name));
        pb
    }

    /// Auto-extract a ZIP file to a target directory
    async fn extract_zip(&self, zip_path: &Path, target_dir: &Path) -> FontResult<()> {
        // Create target directory
        fs::create_dir_all(target_dir).await.map_err(|e| {
            FontError::verification(format!("Failed to create extraction directory: {}", e))
        })?;

        // Extract in blocking task to avoid blocking async runtime
        let zip_path = zip_path.to_path_buf();
        let target_dir = target_dir.to_path_buf();

        tokio::task::spawn_blocking(move || crate::extract::extract_zip(&zip_path, &target_dir))
            .await
            .map_err(|e| FontError::verification(format!("Extraction task failed: {}", e)))?
    }

    fn get_extension_from_response(&self, response: &reqwest::Response, url: &str) -> String {
        // Try to get from content-type
        if let Some(content_type) = response.headers().get("content-type")
            && let Ok(ct) = content_type.to_str()
        {
            if ct.contains("zip") {
                return "zip".to_string();
            } else if ct.contains("woff2") {
                return "woff2".to_string();
            } else if ct.contains("woff") {
                return "woff".to_string();
            } else if ct.contains("ttf") || ct.contains("truetype") {
                return "ttf".to_string();
            } else if ct.contains("otf") || ct.contains("opentype") {
                return "otf".to_string();
            }
        }

        // Try to get from URL
        if url.contains(".zip") {
            "zip".to_string()
        } else if url.contains(".woff2") {
            "woff2".to_string()
        } else if url.contains(".woff") {
            "woff".to_string()
        } else if url.contains(".ttf") {
            "ttf".to_string()
        } else if url.contains(".otf") {
            "otf".to_string()
        } else {
            "zip".to_string() // Default to zip for font packages
        }
    }
}

/// Download result with verification status
#[derive(Debug)]
pub struct DownloadResult {
    /// ID of the downloaded font
    pub font_id: String,
    /// Provider the font was downloaded from
    pub provider: FontProvider,
    /// List of downloaded files
    pub files: Vec<PathBuf>,
    /// Whether the download was successful
    pub success: bool,
    /// Error message if download failed
    pub error: Option<String>,
    /// Whether the downloaded files were verified
    pub verified: bool,
    /// Total bytes downloaded
    pub bytes_downloaded: u64,
    /// Duration of the download
    pub duration: Duration,
}

impl DownloadResult {
    /// Create a successful download result
    pub fn success(
        font_id: String,
        provider: FontProvider,
        files: Vec<PathBuf>,
        bytes_downloaded: u64,
        duration: Duration,
    ) -> Self {
        Self {
            font_id,
            provider,
            files,
            success: true,
            error: None,
            verified: true,
            bytes_downloaded,
            duration,
        }
    }

    /// Create a failed download result
    pub fn failure(
        font_id: String,
        provider: FontProvider,
        error: String,
        duration: Duration,
    ) -> Self {
        Self {
            font_id,
            provider,
            files: Vec::new(),
            success: false,
            error: Some(error),
            verified: false,
            bytes_downloaded: 0,
            duration,
        }
    }
}
