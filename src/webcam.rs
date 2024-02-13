use std::sync::Arc;

use image::{ImageBuffer, Rgb};
use tokio::sync::{watch::Sender, Mutex};

use nokhwa::{
    pixel_format::RgbFormat,
    utils::{CameraIndex, RequestedFormat, RequestedFormatType, Resolution},
};

use crate::AppState;

#[derive(Clone)]
pub(crate) struct Camera {
    pub index: CameraIndex,
    pub resolution: Resolution,
    pub camera: Arc<Mutex<nokhwa::Camera>>,

    pub buffer: Option<ImageBuffer<Rgb<u8>, Vec<u8>>>,
}

pub async fn setup_cameras(state: Arc<AppState>, cameras_tx: Sender<Vec<Camera>>) {
    let mut camera_list = vec![];

    let connected_cameras = nokhwa::query(nokhwa::native_api_backend().unwrap()).unwrap();

    // Print the number of connected cameras and their names
    println!("Connected Cameras: {}", connected_cameras.len());
    for camera in &connected_cameras {
        println!("Camera {}: {}", camera.index(), camera.human_name());
    }

    // Open the cameras and start streaming
    for camera in connected_cameras {
        // let resolution = RequestedFormatType::HighestResolution(Resolution::new(320, 240));
        // let resolution = RequestedFormatType::HighestResolution(Resolution::new(640, 480));
        let resolution = RequestedFormatType::HighestResolution(Resolution::new(1280, 720));
        // let resolution = RequestedFormatType::HighestResolution(Resolution::new(1920, 1080));

        let requested_format = RequestedFormat::new::<RgbFormat>(resolution);

        let mut camera = nokhwa::Camera::new(camera.index().clone(), requested_format).unwrap();

        camera.open_stream().unwrap();

        camera_list.push(Camera {
            index: camera.index().clone(),
            resolution: camera.resolution(),
            camera: Arc::new(Mutex::new(camera)),
            buffer: None,
        });
    }

    // Loop through the cameras and send the images to the webserver
    loop {
        for camera_iter in &mut camera_list {
            let camera_buffer = camera_iter.camera.lock().await.frame().unwrap();
            let buffer = camera_buffer.decode_image::<RgbFormat>().unwrap();

            camera_iter.buffer = Some(buffer)
        }

        cameras_tx.send(camera_list.clone()).unwrap();
        tokio::time::sleep(tokio::time::Duration::from_millis(
            state.config.seconds_per_frame as u64,
        ))
        .await;
    }
}
