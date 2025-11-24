use reqwest::{Client, Url};
use thiserror::Error;

const BASE_URL: &str = "https://quickchart.io";

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

#[derive(Error, Debug)]
pub enum QCError {
    #[error("HTTP error: {0}")]
    HttpError(#[from] reqwest::Error),
    #[error("Failed to parse JSON response: {0}")]
    JsonParseError(#[from] serde_json::Error),
    #[error("Failed to parse URL: {0}")]
    UrlParseError(#[from] url::ParseError),
}

impl QuickchartClient {
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

    // Builder methods
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

    // Parse chart string as JSON, or return as string if invalid JSON
    fn parse_chart(chart: &str) -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(chart)
            .unwrap_or_else(|_| serde_json::Value::String(chart.to_string()))
    }

    // Build JSON body with chart and optional parameters
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

    // Compact the chart string by removing unnecessary whitespace
    fn compact_chart(chart: &str) -> String {
        // Try to parse as JSON first and minify it
        if let Ok(chart_json) = serde_json::from_str::<serde_json::Value>(chart) {
            // Valid JSON - return minified version
            return serde_json::to_string(&chart_json).unwrap_or_else(|_| chart.to_string());
        }

        // Not valid JSON - compact JavaScript object notation
        // Remove newlines and collapse multiple spaces to single spaces
        let mut result = String::with_capacity(chart.len());
        let mut prev_was_space = false;

        for ch in chart.chars() {
            if ch == '\n' || ch == '\r' {
                // Replace newlines with a space (if not already preceded by space)
                if !prev_was_space {
                    result.push(' ');
                    prev_was_space = true;
                }
            } else if ch.is_whitespace() {
                // Collapse multiple whitespace characters to single space
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

    /// Generate the URL using the given configuration.
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
}
