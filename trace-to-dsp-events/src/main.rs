mod metrics;
mod processing;

use anyhow::Result;
use charts::{ScaleLinear, LineSeriesView, MarkerType, PointLabelPosition, Chart, Color};
use clap::Parser;
use common::Intensity;
use kagiyama::{AlwaysReady, Watcher};
use rdkafka::{
    config::ClientConfig,
    consumer::{stream_consumer::StreamConsumer, CommitMode, Consumer},
    message::Message,
    producer::{FutureProducer, FutureRecord},
};
use std::{net::SocketAddr, time::Duration};
use streaming_types::dat1_digitizer_analog_trace_v1_generated::{
    digitizer_analog_trace_message_buffer_has_identifier, root_as_digitizer_analog_trace_message,
};

mod dsp;
use dsp::*;

use rand::random;

#[tokio::main]
async fn main() -> Result<()> {
    // Define chart related sizes.
    let width = 1600;
    let height = 1200;
    let (top, right, bottom, left) = (90, 40, 50, 60);

    // Create a band scale that will interpolate values in [0, 200] to values in the
    // [0, availableWidth] range (the width of the chart without the margins).
    let x = ScaleLinear::new()
        .set_domain(vec![0_f32, 200_f32])
        .set_range(vec![0, width - left - right]);

    // Create a linear scale that will interpolate values in [0, 100] range to corresponding
    // values in [availableHeight, 0] range (the height of the chart without the margins).
    // The [availableHeight, 0] range is inverted because SVGs coordinate system's origin is
    // in top left corner, while chart's origin is in bottom left corner, hence we need to invert
    // the range on Y axis for the chart to display as though its origin is at bottom left.
    let y = ScaleLinear::new()
        .set_domain(vec![0_f32, 200_f32])
        .set_range(vec![height - top - bottom, 0]);

    // You can use your own iterable as data as long as its items implement the `PointDatum` trait.
    let data = (0..200).map(|_|random::<Intensity>() % 128).collect::<Vec<Intensity>>();
    let old_range = std(&data);

    // Create Line series view that is going to represent the data.
    let line_view1 = LineSeriesView::new()
        .set_x_scale(&x)
        .set_y_scale(&y)
        .set_marker_type(MarkerType::Circle)
        .set_label_position(PointLabelPosition::N)
        .set_label_visibility(false)
        .set_colors(Color::from_vec_of_hex_strings(vec!["#bbbbbb"]))
        .load_data(&vector_to_point_data(&data)).unwrap();

        // You can use your own iterable as data as long as its items implement the `PointDatum` trait.
        let data = smooth(data,3);
    
        // Create Line series view that is going to represent the data.
        let line_view2 = LineSeriesView::new()
            .set_x_scale(&x)
            .set_y_scale(&y)
            .set_marker_type(MarkerType::Circle)
            .set_label_position(PointLabelPosition::N)
            .set_label_visibility(false)
            .set_colors(Color::from_vec_of_hex_strings(vec!["#880000"]))
            .load_data(&vector_to_point_data(&data)).unwrap();
    

    // You can use your own iterable as data as long as its items implement the `PointDatum` trait.
    let new_range = std(&data);
    let data = scale(data,old_range/new_range);

    // Create Line series view that is going to represent the data.
    let line_view3 = LineSeriesView::new()
        .set_x_scale(&x)
        .set_y_scale(&y)
        .set_marker_type(MarkerType::Circle)
        .set_label_position(PointLabelPosition::N)
        .set_label_visibility(false)
        .set_colors(Color::from_vec_of_hex_strings(vec!["#000000"]))
        .load_data(&vector_to_point_data(&data)).unwrap();


    // Generate and save the chart.
    Chart::new()
        .set_width(width)
        .set_height(height)
        .set_margins(top, right, bottom, left)
        .add_title(String::from("Line Chart"))
        .add_view(&line_view1)
        .add_view(&line_view2)
        .add_view(&line_view3)
        .add_axis_bottom(&x)
        .add_axis_left(&y)
        .add_left_axis_label("Custom Y Axis Label")
        .add_bottom_axis_label("Custom X Axis Label")
        .save("line-chart.svg").unwrap();
    Ok(())
}
