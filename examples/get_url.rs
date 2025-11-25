use quickchart_rs::QuickchartClient;

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    // Generate the chart URL
    let url = client.get_url()?;

    println!("Chart URL: {}", url);
    println!("\nYou can open this URL in your browser to view the chart!");

    Ok(())
}
