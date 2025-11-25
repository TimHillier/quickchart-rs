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

    QuickchartClient::new()
        .chart(chart_config.to_string())
        .width(800)
        .height(400)
        .to_file("output.png")
        .await
        .expect("Error writing chart to file");

    Ok(())
}
