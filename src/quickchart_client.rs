use reqwest::{Client, Url};
use std::path::Path;
use thiserror::Error;

const BASE_URL: &str = "https://quickchart.io";

/// Client for interacting with the QuickChart.io API.
///
/// This client provides a builder-pattern API for configuring and generating charts.
/// Use [`new()`](QuickchartClient::new) to create a new client instance, then chain
/// builder methods to configure your chart before calling one of the generation methods.
///
/// # Example
///
/// ```
/// use quickchart_rs::QuickchartClient;
///
/// let client = QuickchartClient::new()
///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
///     .width(800)
///     .height(400);
///
/// let url = client.get_url().unwrap();
/// assert!(url.starts_with("https://quickchart.io/chart"));
/// ```
pub struct QuickchartClient {
    client: Client,
    base_url: Url,
    chart: String,
    width: Option<usize>,
    height: Option<usize>,
    device_pixel_ratio: Option<f32>,
    background_color: Option<String>,
    version: Option<String>,
    format: Option<String>,
}

/// Errors that can occur when using the QuickChart client.
#[derive(Error, Debug)]
pub enum QCError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

impl QuickchartClient {
    /// Create a new QuickChart client instance.
    ///
    /// # Example
    ///
    /// ```
    /// use quickchart_rs::QuickchartClient;
    ///
    /// let client = QuickchartClient::new();
    /// ```
    pub fn new() -> QuickchartClient {
        let client = Client::builder()
            .user_agent("quickchart-rs/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        QuickchartClient {
            client,
            base_url: Url::parse(BASE_URL).expect("Failed to parse base URL"),
            chart: String::new(),
            width: None,
            height: None,
            device_pixel_ratio: None,
            background_color: None,
            version: None,
            format: None,
        }
    }

    /// Set the Chart.js configuration.
    ///
    /// The chart parameter should be a JSON string containing a valid Chart.js configuration.
    /// Both valid JSON and JavaScript object notation are supported.
    ///
    /// # Arguments
    ///
    /// * `chart` - Chart.js configuration as a JSON string
    ///
    /// # Example
    ///
    /// ```
    /// use quickchart_rs::QuickchartClient;
    ///
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string());
    /// ```
    pub fn chart(mut self, chart: String) -> Self {
        self.chart = chart;
        self
    }

    /// Set the chart width in pixels.
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the chart height in pixels.
    ///
    /// # Arguments
    ///
    /// * `height` - Height in pixels
    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    /// Set the device pixel ratio for high-DPI displays.
    ///
    /// Useful for generating sharper images on retina/high-DPI displays.
    /// Common values are 1.0 (standard) or 2.0 (retina).
    ///
    /// # Arguments
    ///
    /// * `dpr` - Device pixel ratio (typically 1.0 or 2.0)
    pub fn device_pixel_ratio(mut self, dpr: f32) -> Self {
        self.device_pixel_ratio = Some(dpr);
        self
    }

    /// Set the background color of the chart.
    ///
    /// # Arguments
    ///
    /// * `color` - Background color (e.g., "transparent", "#ffffff", "white")
    pub fn background_color(mut self, color: String) -> Self {
        self.background_color = Some(color);
        self
    }

    /// Set the Chart.js version to use.
    ///
    /// # Arguments
    ///
    /// * `version` - Chart.js version (e.g., "2" or "3")
    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

    /// Set the output image format.
    ///
    /// # Arguments
    ///
    /// * `format` - Image format ("png" or "svg")
    pub fn format(mut self, format: String) -> Self {
        self.format = Some(format);
        self
    }

    fn parse_chart(chart: &str) -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(chart)
            .unwrap_or_else(|_| serde_json::Value::String(chart.to_string()))
    }

    fn build_json_body(&self) -> serde_json::Value {
        let chart_value = Self::parse_chart(&self.chart);
        let mut json_body = serde_json::json!({ "chart": chart_value });

        if let Some(w) = self.width {
            json_body["width"] = serde_json::Value::Number(w.into());
        }
        if let Some(h) = self.height {
            json_body["height"] = serde_json::Value::Number(h.into());
        }
        if let Some(dpr) = self.device_pixel_ratio {
            json_body["devicePixelRatio"] =
                serde_json::Value::Number(serde_json::Number::from_f64(dpr as f64).unwrap());
        }
        if let Some(ref bkg) = self.background_color {
            json_body["backgroundColor"] = serde_json::Value::String(bkg.clone());
        }
        if let Some(ref v) = self.version {
            json_body["version"] = serde_json::Value::String(v.clone());
        }
        if let Some(ref f) = self.format {
            json_body["format"] = serde_json::Value::String(f.clone());
        }

        json_body
    }

    fn compact_chart(chart: &str) -> String {
        if let Ok(chart_json) = serde_json::from_str::<serde_json::Value>(chart) {
            return serde_json::to_string(&chart_json).unwrap_or_else(|_| chart.to_string());
        }

        let mut result = String::with_capacity(chart.len());
        let mut prev_was_space = false;

        for ch in chart.chars() {
            if ch == '\n' || ch == '\r' {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else if ch.is_whitespace() {
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else {
                result.push(ch);
                prev_was_space = false;
            }
        }

        result.trim().to_string()
    }

    /// Generate a chart URL using the current configuration.
    ///
    /// This method creates a GET request URL that can be used to fetch the chart image.
    /// The URL includes all configured parameters as query parameters.
    ///
    /// # Returns
    ///
    /// A `Result` containing the chart URL as a `String`, or an error if URL construction fails.
    ///
    /// # Example
    ///
    /// ```
    /// use quickchart_rs::QuickchartClient;
    ///
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
    ///     .width(800)
    ///     .height(400);
    ///
    /// let url = client.get_url().unwrap();
    /// assert!(url.starts_with("https://quickchart.io/chart"));
    /// assert!(url.contains("w=800"));
    /// assert!(url.contains("h=400"));
    /// ```
    pub fn get_url(&self) -> Result<String, QCError> {
        // Compact the chart string to make the URL shorter
        let compacted_chart = Self::compact_chart(&self.chart);

        let mut url = self.base_url.join("/chart")?;

        {
            let mut query = url.query_pairs_mut();
            query.append_pair("c", &compacted_chart);

            if let Some(w) = self.width {
                query.append_pair("w", &w.to_string());
            }
            if let Some(h) = self.height {
                query.append_pair("h", &h.to_string());
            }
            if let Some(dpr) = self.device_pixel_ratio {
                query.append_pair("devicePixelRatio", &dpr.to_string());
            }
            if let Some(ref bkg) = self.background_color {
                query.append_pair("bkg", bkg);
            }
            if let Some(ref v) = self.version {
                query.append_pair("v", v);
            }
            if let Some(ref f) = self.format {
                query.append_pair("f", f);
            }
        }

        Ok(url.to_string())
    }

    /// Create a short URL for the chart via POST request.
    ///
    /// This method sends a POST request to QuickChart's `/chart/create` endpoint
    /// and returns a shortened URL that can be shared or embedded.
    ///
    /// # Returns
    ///
    /// A `Result` containing the short URL as a `String`, or an error if the request fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use quickchart_rs::QuickchartClient;
    ///
    /// # async fn example() -> Result<(), quickchart_rs::QCError> {
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string());
    ///
    /// let short_url = client.get_short_url().await?;
    /// println!("Short URL: {}", short_url);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_short_url(&self) -> Result<String, QCError> {
        let json_body = self.build_json_body();

        let response = self
            .client
            .post(self.base_url.join("/chart/create")?.to_string())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&json_body)?)
            .send()
            .await?;

        let response = response.error_for_status()?;
        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        // Extract the "url" field from the response
        let url = response_json
            .get("url")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                serde_json::from_str::<serde_json::Value>("null")
                    .map_err(|e| QCError::JsonParseError(e))
                    .unwrap_err()
            })?;

        Ok(url.trim_matches('"').trim_matches('\'').to_string())
    }

    /// Download the chart image as bytes via POST request.
    ///
    /// This method sends a POST request to QuickChart and returns the image bytes.
    /// Useful for saving charts to files or processing them in memory.
    ///
    /// # Returns
    ///
    /// A `Result` containing the image bytes as a `Vec<u8>`, or an error if the request fails.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use quickchart_rs::QuickchartClient;
    ///
    /// # async fn example() -> Result<(), quickchart_rs::QCError> {
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
    ///     .format("png".to_string());
    ///
    /// let image_bytes = client.post().await?;
    /// std::fs::write("chart.png", image_bytes)?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn post(&self) -> Result<Vec<u8>, QCError> {
        let json_body = self.build_json_body();

        let response = self
            .client
            .post(self.base_url.join("/chart")?.to_string())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(&json_body)?)
            .send()
            .await?;

        let response = response.error_for_status()?;
        Ok(response.bytes().await?.to_vec())
    }

    /// Download the chart image and save it directly to a file.
    ///
    /// This is a convenience method that combines [`post()`](QuickchartClient::post) and file writing.
    /// It downloads the chart image and saves it to the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - Path where the image file should be saved (can be `&str`, `String`, `Path`, or `PathBuf`)
    ///
    /// # Returns
    ///
    /// A `Result` that is `Ok(())` if the file was written successfully, or an error if the request or file write fails.
    ///
    /// # Example
    ///
    /// ```
    /// use quickchart_rs::QuickchartClient;
    /// use std::fs;
    ///
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string());
    ///
    /// let rt = tokio::runtime::Runtime::new().unwrap();
    /// let test_path = "test_chart_output.png";
    /// rt.block_on(client.to_file(test_path)).unwrap();
    /// assert!(fs::metadata(test_path).unwrap().len() > 0);
    /// fs::remove_file(test_path).unwrap();
    /// ```
    pub async fn to_file(&self, path: impl AsRef<Path>) -> Result<(), QCError> {
        let image_bytes = self.post().await?;
        std::fs::write(path, image_bytes)?;
        Ok(())
    }
}
