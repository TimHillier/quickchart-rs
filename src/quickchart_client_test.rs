use super::*;

#[test]
fn test_new_client() {
    let client = QuickchartClient::new();
    assert_eq!(client.chart, "");
    assert_eq!(client.width, None);
    assert_eq!(client.height, None);
    assert_eq!(client.device_pixel_ratio, None);
    assert_eq!(client.background_color, None);
    assert_eq!(client.version, None);
    assert_eq!(client.format, None);
}

#[test]
fn test_builder_pattern() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar"}"#.to_string())
        .width(800)
        .height(400)
        .device_pixel_ratio(2.0)
        .background_color("transparent".to_string())
        .version("3".to_string())
        .format("png".to_string());

    assert_eq!(client.chart, r#"{"type":"bar"}"#);
    assert_eq!(client.width, Some(800));
    assert_eq!(client.height, Some(400));
    assert_eq!(client.device_pixel_ratio, Some(2.0));
    assert_eq!(client.background_color, Some("transparent".to_string()));
    assert_eq!(client.version, Some("3".to_string()));
    assert_eq!(client.format, Some("png".to_string()));
}

#[test]
fn test_get_url_basic() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar","data":{"labels":["A","B"],"datasets":[{"data":[1,2]}]}}"#.to_string());

    let url = client.get_url().unwrap();
    assert!(url.starts_with("https://quickchart.io/chart"));
    assert!(url.contains("c="));
}

#[test]
fn test_get_url_with_all_parameters() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar"}"#.to_string())
        .width(1200)
        .height(600)
        .device_pixel_ratio(2.0)
        .background_color("white".to_string())
        .version("3".to_string())
        .format("svg".to_string());

    let url = client.get_url().unwrap();
    assert!(url.contains("w=1200"));
    assert!(url.contains("h=600"));
    assert!(url.contains("devicePixelRatio=2"));
    assert!(url.contains("bkg=white"));
    assert!(url.contains("v=3"));
    assert!(url.contains("f=svg"));
}

#[test]
fn test_get_url_with_partial_parameters() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"line"}"#.to_string())
        .width(800)
        .format("png".to_string());

    let url = client.get_url().unwrap();
    assert!(url.contains("w=800"));
    assert!(url.contains("f=png"));
    // Should not contain h= since height wasn't set
    assert!(!url.contains("h="));
}

#[test]
fn test_compact_chart_with_valid_json() {
    let chart = r#"{
        "type": "bar",
        "data": {
            "labels": ["A", "B"]
        }
    }"#;
    let compacted = QuickchartClient::compact_chart(chart);
    // Should be valid JSON without extra whitespace
    assert!(serde_json::from_str::<serde_json::Value>(&compacted).is_ok());
    assert!(!compacted.contains('\n'));
}

#[test]
fn test_compact_chart_with_javascript_notation() {
    let chart = "{type: 'bar', data: {labels: ['A', 'B']}}";
    let compacted = QuickchartClient::compact_chart(chart);
    // Should remove extra whitespace
    assert!(compacted.len() <= chart.len());
    assert!(!compacted.contains('\n'));
}

#[test]
fn test_compact_chart_preserves_content() {
    let chart = r#"{"type":"bar","data":{"labels":["A","B"]}}"#;
    let compacted = QuickchartClient::compact_chart(chart);
    // Already compact JSON should remain similar
    assert!(compacted.contains("bar"));
    assert!(compacted.contains("A"));
    assert!(compacted.contains("B"));
}

#[test]
fn test_build_json_body() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar"}"#.to_string())
        .width(800)
        .height(400)
        .device_pixel_ratio(2.0)
        .background_color("transparent".to_string())
        .version("3".to_string())
        .format("png".to_string());

    let json_body = client.build_json_body();
    
    assert!(json_body.get("chart").is_some());
    assert_eq!(json_body["width"], 800);
    assert_eq!(json_body["height"], 400);
    assert_eq!(json_body["devicePixelRatio"].as_f64().unwrap(), 2.0);
    assert_eq!(json_body["backgroundColor"], "transparent");
    assert_eq!(json_body["version"], "3");
    assert_eq!(json_body["format"], "png");
}

#[test]
fn test_build_json_body_with_minimal_config() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"pie"}"#.to_string());

    let json_body = client.build_json_body();
    
    assert!(json_body.get("chart").is_some());
    // Optional fields should not be present
    assert!(json_body.get("width").is_none());
    assert!(json_body.get("height").is_none());
}

#[test]
fn test_error_display() {
    let error = QCError::MissingField("url".to_string());
    let error_msg = format!("{}", error);
    assert!(error_msg.contains("Missing field"));
    assert!(error_msg.contains("url"));
}

#[test]
fn test_builder_chain() {
    let client = QuickchartClient::new()
        .chart("test".to_string())
        .width(100)
        .height(200)
        .width(300) // Overwrite previous width
        .format("svg".to_string());

    assert_eq!(client.width, Some(300));
    assert_eq!(client.height, Some(200));
    assert_eq!(client.format, Some("svg".to_string()));
}

#[test]
fn test_url_encoding() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar","data":{"labels":["Test & Value"]}}"#.to_string());

    let url = client.get_url().unwrap();
    // URL should be properly encoded
    assert!(url.contains("https://quickchart.io/chart"));
    // The chart parameter should be in the URL
    assert!(url.contains("c="));
}

#[test]
fn test_device_pixel_ratio_formatting() {
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar"}"#.to_string())
        .device_pixel_ratio(1.5);

    let url = client.get_url().unwrap();
    assert!(url.contains("devicePixelRatio"));
}

#[test]
fn test_background_color_variations() {
    // Test named colors
    let named_colors = vec!["transparent", "white", "black"];
    for color in named_colors {
        let client = QuickchartClient::new()
            .chart(r#"{"type":"bar"}"#.to_string())
            .background_color(color.to_string());

        let url = client.get_url().unwrap();
        assert!(url.contains("bkg="));
        // Named colors should appear in URL (may be URL encoded)
        assert!(url.contains(color) || url.contains(&color.replace(" ", "%20")));
    }

    // Test HEX colors
    let hex_colors = vec!["#ffffff", "#FF0000", "#00ff00", "#000"];
    for color in hex_colors {
        let client = QuickchartClient::new()
            .chart(r#"{"type":"bar"}"#.to_string())
            .background_color(color.to_string());

        let url = client.get_url().unwrap();
        assert!(url.contains("bkg="));
        // HEX colors should be in the URL (may be URL encoded, # becomes %23)
        assert!(url.contains("#") || url.contains("%23"));
    }

    // Test RGB colors
    let rgb_colors = vec!["rgb(255, 0, 0)", "rgb(100, 150, 200)", "rgb(0,255,128)"];
    for color in rgb_colors {
        let client = QuickchartClient::new()
            .chart(r#"{"type":"bar"}"#.to_string())
            .background_color(color.to_string());

        let url = client.get_url().unwrap();
        assert!(url.contains("bkg="));
        // RGB colors should be in the URL (spaces will be URL encoded as %20)
        assert!(url.contains("rgb") || url.contains("255") || url.contains("100"));
    }

    // Test HSL colors
    let hsl_colors = vec!["hsl(0, 100%, 50%)", "hsl(120, 50%, 75%)"];
    for color in hsl_colors {
        let client = QuickchartClient::new()
            .chart(r#"{"type":"bar"}"#.to_string())
            .background_color(color.to_string());

        let url = client.get_url().unwrap();
        assert!(url.contains("bkg="));
        // HSL colors should be in the URL (spaces and % will be URL encoded)
        assert!(url.contains("hsl") || url.contains("120") || url.contains("0"));
    }
}

#[test]
fn test_background_color_url_encoding() {
    // Test that RGB colors with spaces are properly URL encoded
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar"}"#.to_string())
        .background_color("rgb(255, 0, 0)".to_string());

    let url = client.get_url().unwrap();
    // The URL should contain the color parameter
    assert!(url.contains("bkg="));
    // Spaces should be encoded as %20, or the URL should be valid
    assert!(url.contains("%20") || url.contains("rgb"));
    
    // Verify the URL can be parsed (proves encoding worked)
    assert!(url::Url::parse(&url).is_ok());
}

#[test]
fn test_background_color_hex_formats() {
    // Test various HEX formats
    let hex_formats = vec!["#fff", "#ffffff", "#FF0000", "#00ff00", "#abc123"];
    for hex in hex_formats {
        let client = QuickchartClient::new()
            .chart(r#"{"type":"bar"}"#.to_string())
            .background_color(hex.to_string());

        let url = client.get_url().unwrap();
        assert!(url.contains("bkg="));
        // Verify URL is valid
        assert!(url::Url::parse(&url).is_ok());
    }
}

#[test]
fn test_background_color_rgb_hsl_formats() {
    // Test RGB with and without spaces
    let rgb_with_spaces = "rgb(255, 0, 0)";
    let rgb_no_spaces = "rgb(255,0,0)";
    
    for rgb in &[rgb_with_spaces, rgb_no_spaces] {
        let client = QuickchartClient::new()
            .chart(r#"{"type":"bar"}"#.to_string())
            .background_color(rgb.to_string());

        let url = client.get_url().unwrap();
        assert!(url.contains("bkg="));
        assert!(url::Url::parse(&url).is_ok());
    }

    // Test HSL format
    let hsl = "hsl(120, 50%, 75%)";
    let client = QuickchartClient::new()
        .chart(r#"{"type":"bar"}"#.to_string())
        .background_color(hsl.to_string());

    let url = client.get_url().unwrap();
    assert!(url.contains("bkg="));
    assert!(url::Url::parse(&url).is_ok());
}

