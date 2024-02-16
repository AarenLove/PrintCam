use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    response::{Html, IntoResponse, Response},
    routing::get,
};

use image::{ImageBuffer, ImageOutputFormat};
use serde::{Deserialize, Serialize};
use tokio::sync::watch::{self, Receiver};
use webcam::CameraBuffer;

mod webcam;

struct AppState {
    camera_buffer_rx: Receiver<Vec<CameraBuffer>>,
    config: Configuration,
}

#[derive(Debug, Serialize, Deserialize)]
struct Configuration {
    website_ms_per_frame: u32,
    seconds_per_frame: u32,
    jpg_quality: u8, // 0-100

    resolution: Option<(u32, u32)>,
}

#[tokio::main]
async fn main() {
    // Create a channel to send the image from the webcam to the webserver
    let (image_list_tx, camera_list_rx) = watch::channel(Vec::new());

    let config_str =
        std::fs::read_to_string("config.toml").expect("Could not read config.toml file");
    let config: Configuration = toml::from_str(&config_str).unwrap();

    println!("Config: {:?}", config);

    let state = Arc::new(AppState {
        camera_buffer_rx: camera_list_rx,
        config,
    });

    tokio::spawn(webcam::setup_cameras(state.clone(), image_list_tx));

    let app = axum::Router::new()
        // .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/image/:camera_index", get(image))
        .with_state(state);

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

// basic handler that responds with a static string
async fn root(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let website_ms_per_frame = state.config.website_ms_per_frame;

    let html = format!(
        r###"
        <img id="dynamic-image">
        
    <script>
        let urlsToRevoke = [];

        function removeImage(imageUrl) {{
            // console.log('Revoking URL:', imageUrl);
            URL.revokeObjectURL(imageUrl)
        }}

        // Function to update the image
        async function updateImage() {{
            try {{
                // Fetch the image from the server
                const response = await fetch('/image/0');
                const blob = await response.blob();
                
                // Create a URL for the blob
                const imageUrl = URL.createObjectURL(blob);
                
                // Update the src attribute of the image element
                document.getElementById('dynamic-image').src = imageUrl;

                // Clean up the URL after some time
                for (const url of urlsToRevoke) {{
                    removeImage(url);
                }}
                urlsToRevoke = [];
                urlsToRevoke.push(imageUrl);
                
            }} catch (error) {{
                console.error('Error fetching image:', error);
            }}
        }}

        updateImage();
        setInterval(updateImage, {website_ms_per_frame});
    </script>"###
    );

    let html = empty_html_page(&html).await;

    Html(html).into_response()
}

// Get Request for in memory image
// Image gets send to the client directly without saving it to disk
async fn image(
    State(state): State<Arc<AppState>>,
    Path(camera_index): Path<u32>,
) -> impl IntoResponse {
    let camera_buffer_list = state.camera_buffer_rx.borrow().clone();

    for camera_buffer in camera_buffer_list {
        if camera_buffer.camera_index.as_index().unwrap() == camera_index {
            let image = camera_buffer.buffer.unwrap_or(ImageBuffer::new(1, 1));
            let image_format = ImageOutputFormat::Jpeg(state.config.jpg_quality);

            let mut buffer = std::io::Cursor::new(Vec::new());
            image.write_to(&mut buffer, image_format).unwrap();

            return Response::builder()
                .header("Content-Type", "image/jpg")
                .body(Body::from(buffer.into_inner()))
                .unwrap();
        }
    }

    Response::builder()
        .status(404)
        .body(Body::from("Camera not found"))
        .unwrap()
}

async fn empty_html_page(inner_string: &str) -> String {
    format!(
        r###"<!DOCTYPE html>
    <html lang="en">
    <head>
        <meta charset="UTF-8">
        <title>Home</title>
        <link rel="stylesheet" type="text/css" href="/assets/global.css" />
    </head>
        <body>
        <div class="content">
            {inner_string}
        </div>
        </body>
    </html>"###
    )
}
