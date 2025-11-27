use reqwest::{Client, Url};
use std::path::Path;
use thiserror::Error;

#[cfg(test)]
#[path = "quickchart_client_test.rs"]
mod tests;

const BASE_URL: &str = "https://quickchart.io";
const USER_AGENT: &str = concat!("quickchart-rs/", env!("CARGO_PKG_VERSION"));
const CHART_ENDPOINT: &str = "/chart";
const CREATE_ENDPOINT: &str = "/chart/create";

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
    #[error("Missing field in response: {0}")]
    MissingField(String),
}

impl Default for QuickchartClient {
    fn default() -> Self {
        Self::new()
    }
}

impl QuickchartClient {
    /// Create a new QuickChart client instance.
    pub fn new() -> Self {
        let client = Client::builder()
            .user_agent(USER_AGENT)
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

    /// Set the Chart.js configuration as a JSON string. Both valid JSON and JavaScript object notation are supported.
    pub fn chart(mut self, chart: String) -> Self {
        self.chart = chart;
        self
    }

    pub fn width(mut self, width: usize) -> Self {
        self.width = Some(width);
        self
    }

    pub fn height(mut self, height: usize) -> Self {
        self.height = Some(height);
        self
    }

    pub fn device_pixel_ratio(mut self, dpr: f32) -> Self {
        self.device_pixel_ratio = Some(dpr);
        self
    }

    /// Set the background color. Supports named colors ("transparent", "white"), HEX ("#ffffff"),
    /// RGB ("rgb(255, 0, 0)"), and HSL ("hsl(0, 100%, 50%)") formats.
    pub fn background_color(mut self, color: String) -> Self {
        self.background_color = Some(color);
        self
    }

    pub fn version(mut self, version: String) -> Self {
        self.version = Some(version);
        self
    }

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
        // Try to parse as JSON
        if let Ok(chart_json) = serde_json::from_str::<serde_json::Value>(chart) {
            return serde_json::to_string(&chart_json).unwrap_or_else(|_| chart.to_string());
        }

        // For non-JSON
        chart
            .chars()
            .fold((String::with_capacity(chart.len()), false), |(mut acc, prev_space), ch| {
                let is_whitespace = ch.is_whitespace();
                if is_whitespace && !prev_space {
                    acc.push(' ');
                } else if !is_whitespace {
                    acc.push(ch);
                }
                (acc, is_whitespace)
            })
            .0
            .trim()
            .to_string()
    }

    /// Generate a chart URL with all configured parameters as query parameters.
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
        let compacted_chart = Self::compact_chart(&self.chart);
        let mut url = self.base_url.join(CHART_ENDPOINT)?;

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

    /// Create a short URL for the chart via POST request to `/chart/create`.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use quickchart_rs::QuickchartClient;
    ///
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
    ///     .get_short_url().await?;
    /// ```
    pub async fn get_short_url(&self) -> Result<String, QCError> {
        let json_body = self.build_json_body();
        let response = self
            .send_post_request(CREATE_ENDPOINT, &json_body)
            .await?;

        let response_text = response.text().await?;
        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;

        response_json
            .get("url")
            .and_then(|v| v.as_str())
            .map(|url| url.trim_matches('"').trim_matches('\'').to_string())
            .ok_or_else(|| QCError::MissingField("url".to_string()))
    }

    async fn send_post_request(
        &self,
        endpoint: &str,
        json_body: &serde_json::Value,
    ) -> Result<reqwest::Response, QCError> {
        let response = self
            .client
            .post(self.base_url.join(endpoint)?.to_string())
            .header("Content-Type", "application/json")
            .body(serde_json::to_string(json_body)?)
            .send()
            .await?;

        response.error_for_status().map_err(Into::into)
    }

    /// Download the chart image as bytes via POST request.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
    ///     .post().await?;
    ///
    /// ```
    pub async fn post(&self) -> Result<Vec<u8>, QCError> {
        let json_body = self.build_json_body();
        let response = self.send_post_request(CHART_ENDPOINT, &json_body).await?;
        Ok(response.bytes().await?.to_vec())
    }

    /// Download the chart image and save it directly to a file. Convenience method that combines
    /// [`post()`](QuickchartClient::post) and file writing.
    ///
    /// # Example
    ///
    /// ```no_run
    /// let client = QuickchartClient::new()
    ///     .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
    ///     .to_file("output.png")
    ///     .await?;
    /// ```
    pub async fn to_file(&self, path: impl AsRef<Path>) -> Result<(), QCError> {
        let image_bytes = self.post().await?;
        std::fs::write(path, image_bytes)?;
        Ok(())
    }
}

