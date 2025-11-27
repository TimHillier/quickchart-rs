# quickchart-rs

A Rust client library for [QuickChart.io](https://quickchart.io), a web service that generates chart images from Chart.js configurations.

## Installation

```toml
[dependencies]
quickchart-rs = "0.1.1"
```

## Usage

### Generate Chart URL

```rust
use quickchart_rs::QuickchartClient;

let client = QuickchartClient::new()
    .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
    .width(800)
    .height(400);

let url = client.get_url()?;
```

### Download Chart Image

```rust
use quickchart_rs::QuickchartClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let image_bytes = QuickchartClient::new()
        .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
        .format("png".to_string())
        .post()
        .await?;
    
    std::fs::write("chart.png", image_bytes)?;
    Ok(())
}
```

### Save to File

```rust
use quickchart_rs::QuickchartClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    QuickchartClient::new()
        .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string())
        .to_file("chart.png")
        .await?;
    Ok(())
}
```

## Documentation

Full API documentation is available at [docs.rs/quickchart-rs](https://docs.rs/quickchart-rs) or by running `cargo doc --open`.

## Examples

See the [examples](examples/) directory for more complete examples.

## License

MIT License - see [LICENSE](LICENSE) for details.
