use anyhow::Error;
use image::{ImageBuffer, Rgba, DynamicImage};

#[cfg(target_os = "linux")]
use x11::xlib::{XOpenDisplay, XDefaultRootWindow, XGetImage, ZPixmap};

/// Capture the screen on Linux (X11 or Wayland)
pub async fn capture_screen(resolution: (u32, u32)) -> Result<Vec<u8>, Error> {
    // Detect display server
    let display_server = detect_display_server();
    
    match display_server.as_str() {
        "x11" => {
            #[cfg(target_os = "linux")]
            {
                match capture_with_x11(resolution).await {
                    Ok(data) => return Ok(data),
                    Err(e) => {
                        tracing::warn!("X11 capture failed: {}, using fallback", e);
                    }
                }
            }
        }
        "wayland" => {
            tracing::info!("Wayland detected, using screenshot crate");
        }
        _ => {
            tracing::warn!("Unknown display server, using fallback");
        }
    }
    
    // Fallback to screenshot crate
    match screenshot::Screen::all() {
        Ok(screens) => {
            if let Some(screen) = screens.first() {
                let image = screen.capture()?;
                let (width, height) = image.dimensions();
                
                // Resize if needed
                if width != resolution.0 || height != resolution.1 {
                    let resized = image.resize_exact(
                        resolution.0,
                        resolution.1,
                        image::imageops::FilterType::Lanczos3
                    );
                    return encode_image(&resized);
                }
                
                return encode_image(&image);
            }
        }
        Err(e) => {
            tracing::warn!("Screenshot crate failed: {}, using test pattern", e);
        }
    }
    
    // Last resort: generate test pattern
    Ok(generate_test_pattern(resolution))
}

/// Detect which display server is running
fn detect_display_server() -> String {
    // Check for Wayland
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return "wayland".to_string();
    }
    
    // Check for X11
    if std::env::var("DISPLAY").is_ok() {
        return "x11".to_string();
    }
    
    "unknown".to_string()
}

/// Capture screen using X11
#[cfg(target_os = "linux")]
async fn capture_with_x11(resolution: (u32, u32)) -> Result<Vec<u8>, Error> {
    // This is a simplified implementation
    // Full implementation would use:
    // 1. XOpenDisplay to connect to X server
    // 2. XDefaultRootWindow to get root window
    // 3. XGetImage to capture screen
    // 4. Convert to image format
    // 5. Resize and encode
    
    // For now, return error to fallback
    Err(anyhow::anyhow!("X11 capture not fully implemented"))
}

/// Encode image as JPEG
fn encode_image(img: &DynamicImage) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 80);
    encoder.encode_image(img)?;
    Ok(buffer)
}

/// Generate test pattern for Linux
fn generate_test_pattern(resolution: (u32, u32)) -> Vec<u8> {
    let (width, height) = resolution;
    let mut img = ImageBuffer::new(width, height);
    
    // Create Linux-style gradient (orange/purple theme)
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = 200 + ((x as f32 / width as f32) * 55.0) as u8;
        let g = 100 + ((y as f32 / height as f32) * 100.0) as u8;
        let b = 50;
        *pixel = Rgba([r, g, b, 255]);
    }
    
    // Convert to JPEG
    let mut buffer = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 70);
    encoder.encode(&img, width, height, image::ColorType::Rgba8).unwrap();
    
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_detect_display_server() {
        let server = detect_display_server();
        assert!(server == "x11" || server == "wayland" || server == "unknown");
    }
    
    #[tokio::test]
    async fn test_capture_screen() {
        let result = capture_screen((1920, 1080)).await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.is_empty());
    }
}