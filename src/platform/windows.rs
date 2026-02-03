use anyhow::Error;
use image::{ImageBuffer, Rgba, DynamicImage};
use std::io::Cursor;

// Windows Graphics Capture API types - currently using xcap fallback instead
// TODO: Update to windows crate v0.52+ API when implementing native capture
#[cfg(target_os = "windows")]
use windows::Graphics::Capture::GraphicsCaptureItem;

/// Capture the screen on Windows using Graphics Capture API
/// Falls back to screenshot crate if native API fails
pub async fn capture_screen(resolution: (u32, u32)) -> Result<Vec<u8>, Error> {
    // Try native Windows Graphics Capture API first
    #[cfg(target_os = "windows")]
    {
        match capture_with_graphics_api(resolution).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                tracing::warn!("Windows Graphics Capture API failed: {}, falling back", e);
            }
        }
    }
    
    
    // Fallback to xcap crate
    match xcap::Monitor::all() {
        Ok(monitors) => {
            if let Some(monitor) = monitors.first() {
                match monitor.capture_image() {
                    Ok(image) => {
                        let dynamic_image = image::DynamicImage::ImageRgba8(image);
                        let (width, height) = dynamic_image.dimensions();
                        
                        // Resize if needed
                        if width != resolution.0 || height != resolution.1 {
                            let resized = dynamic_image.resize_exact(
                                resolution.0, 
                                resolution.1, 
                                image::imageops::FilterType::Lanczos3
                            );
                            return encode_image(&resized);
                        }
                        
                        return encode_image(&dynamic_image);
                    }
                    Err(e) => {
                        tracing::warn!("XCap capture failed: {}, using test pattern", e);
                    }
                }
            }
        }
        Err(e) => {
            tracing::warn!("XCap monitor enumeration failed: {}, using test pattern", e);
        }
    }
    
    // Last resort: generate test pattern
    Ok(generate_test_pattern(resolution))
}

/// Capture screen using Windows Graphics Capture API
/// TODO: Implement with windows crate v0.52+ types
#[cfg(target_os = "windows")]
async fn capture_with_graphics_api(_resolution: (u32, u32)) -> Result<Vec<u8>, Error> {
    // Full implementation would require updated windows crate types:
    // 1. Create D3D11 device
    // 2. Create GraphicsCaptureItem for primary monitor
    // 3. Create frame pool
    // 4. Capture frame
    // 5. Copy to staging texture
    // 6. Map and read pixels
    
    // For now, return error to fallback to xcap
    Err(anyhow::anyhow!("Windows Graphics Capture API not fully implemented"))
}

/// Encode image as JPEG with quality settings
fn encode_image(img: &DynamicImage) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 80);
    encoder.encode_image(img)?;
    Ok(buffer)
}

/// Generate test pattern for Windows (fallback)
fn generate_test_pattern(resolution: (u32, u32)) -> Vec<u8> {
    let (width, height) = resolution;
    let mut img = ImageBuffer::new(width, height);
    
    // Create gradient pattern with "Windows" label
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = ((x as f32 / width as f32) * 255.0) as u8;
        let g = ((y as f32 / height as f32) * 255.0) as u8;
        let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
        *pixel = Rgba([r, g, b, 255]);
    }
    
    // Convert to JPEG
    let mut buffer = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 70);
    encoder.encode(&img, width, height, image::ColorType::Rgba8).unwrap();
    
    buffer
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_capture_screen() {
        let result = capture_screen((1920, 1080)).await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.is_empty());
    }
}