use quickchart_rs::QuickchartClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create a client with a simple bar chart configuration
    let chart_config = "{
        type: 'bar',
        data: {
            labels: ['January', 'February', 'March', 'April'],
            datasets: [{
                label: 'Sales',
                data: [50, 60, 70, 80]
            }]
        }
    }";

    let client = QuickchartClient::new()
        .chart(chart_config.to_string())
        .width(800)
        .height(400);

    // Generate a short URL for the chart
    let short_url = client.get_short_url().await?;

    println!("Short URL: {}", short_url);
    println!("\nYou can share this URL or open it in your browser to view the chart!");

    Ok(())
}
