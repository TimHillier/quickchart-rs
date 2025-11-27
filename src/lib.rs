//! # quickchart-rs
//! 
//! A Rust client library for [QuickChart.io](https://quickchart.io), a web service that generates chart images from
//! Chart.js configurations.
//! 
//! ## Features
//! 
//! - Generate chart URLs for embedding in HTML, email, or other formats
//! - Download chart images as PNG/SVG bytes via POST requests
//! - Create short URLs for sharing charts
//! - Builder pattern API for easy configuration

mod quickchart_client;
pub use quickchart_client::{QuickchartClient, QCError};