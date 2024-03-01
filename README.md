# PrintCam

## Description
This application will setup an IP camera and stream the frames from your Web Cam to a web server. The web server will display the iamges in a web page. This application is intended to be used as a tool for monitoring 3D printers. The user will be able to access the web page from any device connected to the same network as the server. The default port for the web server is 3000.


Just plugin your web cam and run the application. The web server will be available at `localhost:3000` or the internal IP of the Server .



## Dependencies
- [Rust](https://www.rust-lang.org/)

## Build
To build the application, run the following command:
```bash 
cargo run --release
```
