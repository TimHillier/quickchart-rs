# quickchart-rs

A Rust client library for [QuickChart.io](https://quickchart.io), a web service that generates chart images from
Chart.js configurations.

## Features

- Generate chart URLs for embedding in HTML, email, or other formats
- Download chart images as PNG/SVG bytes via POST requests
- Create short URLs for sharing charts
- Builder pattern API for easy configuration
- Support for all QuickChart parameters (width, height, format, version, etc.)

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
quickchart-rs = "0.1.0"
```

## Usage

### Basic Example - Generate Chart URL

```rust
use quickchart_rs::QuickchartClient;

let client = QuickchartClient::new()
.chart(r#"{
        "type": "bar",
        "data": {
            "labels": ["January", "February", "March"],
            "datasets": [{
                "label": "Sales",
                "data": [50, 60, 70]
            }]
        }
    }"#.to_string())
.width(800)
.height(400);

let url = client.get_url() ?;
println!("Chart URL: {}", url);
```

### Download Chart Image

```rust
use quickchart_rs::QuickchartClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = QuickchartClient::new()
        .chart(r#"{
            "type": "line",
            "data": {
                "labels": ["Q1", "Q2", "Q3", "Q4"],
                "datasets": [{
                    "label": "Revenue",
                    "data": [100, 120, 115, 134]
                }]
            }
        }"#.to_string())
        .width(800)
        .height(400)
        .format("png".to_string());

    let image_bytes = client.post().await?;

    // Save to file
    std::fs::write("chart.png", image_bytes)?;

    Ok(())
}
```

### Create Short URL

```rust
use quickchart_rs::QuickchartClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = QuickchartClient::new()
        .chart(r#"{
            "type": "pie",
            "data": {
                "labels": ["Red", "Blue", "Yellow"],
                "datasets": [{
                    "data": [300, 50, 100]
                }]
            }
        }"#.to_string());

    let short_url = client.get_short_url().await?;
    println!("Short URL: {}", short_url);

    Ok(())
}
```

### Advanced Configuration

```rust
use quickchart_rs::QuickchartClient;

let client = QuickchartClient::new()
.chart(chart_config_json.to_string())
.width(1200)
.height(600)
.device_pixel_ratio(2.0)
.background_color("transparent".to_string())
.version("2".to_string())
.format("svg".to_string());
```

## API Reference

### `QuickchartClient`

The main client struct for interacting with QuickChart.io.

#### Builder Methods

- `new() -> QuickchartClient` - Create a new client instance
- `chart(chart: String) -> Self` - Set the Chart.js configuration (JSON string)
- `width(width: usize) -> Self` - Set chart width in pixels
- `height(height: usize) -> Self` - Set chart height in pixels
- `device_pixel_ratio(dpr: f32) -> Self` - Set device pixel ratio (for high-DPI displays)
- `background_color(color: String) -> Self` - Set background color (e.g., "transparent", "#ffffff")
- `version(version: String) -> Self` - Set Chart.js version (e.g., "2", "3")
- `format(format: String) -> Self` - Set output format ("png" or "svg")

#### Methods

- `get_url(&self) -> Result<String, QCError>` - Generate a chart URL (synchronous)
- `post(&self) -> Result<Vec<u8>, QCError>` - Download chart image as bytes (async)
- `get_short_url(&self) -> Result<String, QCError>` - Create a short URL for the chart (async)

### Error Handling

The library uses the `QCError` enum for error handling:

```rust
pub enum QCError {
    HttpError(reqwest::Error),
    JsonParseError(serde_json::Error),
    UrlParseError(url::ParseError),
}
```

All errors implement `std::error::Error` and can be converted using `?`.

## Chart Configuration

The `chart` method accepts a Chart.js configuration as a JSON string. You can use either:

- **Valid JSON**: `{"type": "bar", "data": {...}}`
- **JavaScript object notation**: `{type: 'bar', data: {...}}`

The library will automatically handle both formats.

## Examples

See the [examples](examples/) directory for more complete examples.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Links

- [QuickChart.io Documentation](https://quickchart.io/documentation/)
- [Chart.js Documentation](https://www.chartjs.org/docs/)
