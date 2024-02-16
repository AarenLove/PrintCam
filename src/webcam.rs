use std::sync::Arc;

use image::{ImageBuffer, Rgb};
use tokio::sync::watch::Sender;

use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution},
};

use crate::AppState;

#[derive(Clone)]
pub(crate) struct CameraBuffer {
    pub camera_index: CameraIndex,
    pub buffer: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
    pub _name: String, // Added camera name field
}

pub async fn setup_cameras(state: Arc<AppState>, cameras_tx: Sender<Vec<CameraBuffer>>) {
    let mut camera_list = vec![];

    let connected_cameras = nokhwa::query(nokhwa::native_api_backend().unwrap()).unwrap();
    let res_config = state.config.resolution.unwrap_or((1280, 720));

    // Print the number of connected cameras and their names
    println!("Connected Cameras: {}", connected_cameras.len());
    for camera in &connected_cameras {
        println!(
            "Camera {}: {} at {}p",
            camera.index(),
            camera.human_name(),
            res_config.1
        );
    }

    // Open the cameras and start streaming
    for camera in connected_cameras {
        let requested_format = RequestedFormat::new::<RgbFormat>(
            RequestedFormatType::HighestResolution(Resolution::new(res_config.0, res_config.1)),
        );

        let mut camera = nokhwa::Camera::new(camera.index().clone(), requested_format).unwrap();
        camera.open_stream().unwrap();

        camera_list.push(camera);
    }

    // Loop through the cameras and send the images to the webserver
    loop {
        let mut camera_buffer_list = vec![];

        for camera_iter in &mut camera_list {
            // Check if the camera is still connected
            if !camera_iter.is_stream_open() {
                println!("Camera {} is not connected", camera_iter.index());
                continue;
            }

            let camera_buffer = camera_iter.frame().unwrap();
            let buffer = camera_buffer.decode_image::<RgbFormat>().unwrap();

            // Add the image to the list
            camera_buffer_list.push(CameraBuffer {
                camera_index: camera_iter.index().clone(),
                _name: camera_iter.info().human_name(),
                buffer: Some(buffer),
            });
        }

        // Send the image_list to the webserver
        cameras_tx.send(camera_buffer_list).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(
            state.config.seconds_per_frame as u64,
        ))
        .await;
    }
}
