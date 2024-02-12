use std::sync::Arc;

use image::{ImageBuffer, Rgb}; 
use tokio::sync::watch::Sender;

use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType},
};

use crate::AppState;

pub async fn take_picture(state: Arc<AppState>, image_tx: Sender<ImageBuffer<Rgb<u8>, Vec<u8>>>) {
    let index = CameraIndex::Index(0);
    let resolution = RequestedFormatType::HighestFrameRate(state.config.camera_frame_rate);
    let requested_format = RequestedFormat::new::<RgbFormat>(resolution);

    let mut camera = nokhwa::Camera::new(index, requested_format).unwrap();
    // camera.set_frame_rate(30).unwrap(); Doesnt seem to work. Maybe dependent on RequestedFormatType
    camera.open_stream().unwrap();

    loop {
        let frame_buffer = camera.frame().unwrap();
        let image = frame_buffer.decode_image::<RgbFormat>().unwrap();

        image_tx.send(image).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(
            state.config.seconds_per_frame as u64,
        ))
        .await;
    }
}
