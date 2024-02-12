use std::sync::Arc;

use axum::{
    body::Body,
    extract::State,
    response::{Html, IntoResponse, Response},
    routing::get,
};

use image::{ImageBuffer, ImageOutputFormat};
use serde::{Deserialize, Serialize};
use tokio::sync::watch::{self, Receiver};
// use tower_http::services::{ServeDir, ServeFile};

mod webcam;

struct AppState {
    image: Receiver<ImageBuffer<image::Rgb<u8>, Vec<u8>>>,
    config: Configuration,
}

#[derive(Debug, Serialize, Deserialize)]
struct Configuration {
    camera_frame_rate: u32,
    website_ms_per_frame: u32,
    seconds_per_frame: u32,
    jpg_quality: u8, // 0-100
}

#[tokio::main]
async fn main() {
    // Create a channel to send the image from the webcam to the webserver
    let (image_tx, image_rx) = watch::channel(ImageBuffer::new(1, 1));

    let config_str =
        std::fs::read_to_string("config.toml").expect("Could not read config.toml file");
    let config: Configuration = toml::from_str(&config_str).unwrap();

    println!("Config: {:#?}", config);

    let state = Arc::new(AppState {
        image: image_rx,
        config,
    });

    tokio::spawn(webcam::take_picture(state.clone(), image_tx));

    let app = axum::Router::new()
        // Add asset route
        // .nest_service("/assets", ServeDir::new("assets"))
        .route("/", get(root))
        .route("/image", get(image))
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

        function getTextAfterLastSlash(str) {{
            const lastSlashIndex = str.lastIndexOf("/");
            if (lastSlashIndex !== -1) {{
                return str.substring(lastSlashIndex + 1);
            }} else {{
                return str; // If there is no "/" in the string, return the original string
            }}
        }}

        function removeImage(imageUrl) {{
            // console.log('Revoking URL:', imageUrl);
            URL.revokeObjectURL(imageUrl)

            // let uuid = getTextAfterLastSlash(imageUrl);
            // console.log('Revoking uuid:', uuid);
            // URL.revokeObjectURL(imageUrl)
        }}

        // Function to update the image
        async function updateImage() {{
            try {{
                // Fetch the image from the server
                const response = await fetch('/image');
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

    let html = empty_html_page(state, &html).await;

    Html(html).into_response()
}

// Get Request for in memory image
// Image gets send to the client directly without saving it to disk
async fn image(State(state): State<Arc<AppState>>) -> impl IntoResponse {
    let mut buffer = std::io::Cursor::new(Vec::new());

    let image = state.image.borrow();
    image
        .write_to(
            &mut buffer,
            ImageOutputFormat::Jpeg(state.config.jpg_quality),
        )
        .unwrap();

    Response::builder()
        .header("Content-Type", "image/jpg")
        .body(Body::from(buffer.into_inner()))
        .unwrap()
}

async fn empty_html_page(state: Arc<AppState>, inner_string: &str) -> String {
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

// <script>
//     // list of URLs to revoke
//     let urlsToRevoke = [];

//     function removeImage(imageUrl) {{
//         // console.log('Revoking URL:', imageUrl);
//         URL.revokeObjectURL(imageUrl)
//     }}

//     // Function to update the image
//     async function updateImage() {{
//         try {{
//             // Fetch the image from the server
//             const response = await fetch('/image');
//             const blob = await response.blob();

//             // Create a URL for the blob
//             const imageUrl = URL.createObjectURL(blob);

//             // Update the src attribute of the image element
//             document.getElementById('dynamic-image').src = imageUrl;

//             // Clean up the URL after some time
//             for (const url of urlsToRevoke) {{
//                 removeImage(url);
//             }}
//             urlsToRevoke = [];

//             urlsToRevoke.push(imageUrl);

//         }} catch (error) {{
//             console.error('Error fetching image:', error);
//         }}
//     }}

//     updateImage();
//     setInterval(updateImage, {website_ms_per_frame});

//     </script>
