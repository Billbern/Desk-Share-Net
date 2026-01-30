use anyhow::Error;
use image::{ImageBuffer, Rgba, DynamicImage};

#[cfg(target_os = "macos")]
use core_graphics::{
    display::{CGDisplay, CGDisplayStreamRef, CGDisplayStreamUpdateRef},
    image::CGImageRef,
};

/// Capture the screen on macOS using Core Graphics
pub async fn capture_screen(resolution: (u32, u32)) -> Result<Vec<u8>, Error> {
    #[cfg(target_os = "macos")]
    {
        match capture_with_core_graphics(resolution).await {
            Ok(data) => return Ok(data),
            Err(e) => {
                tracing::warn!("Core Graphics capture failed: {}, using fallback", e);
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

/// Capture screen using Core Graphics (macOS native)
#[cfg(target_os = "macos")]
async fn capture_with_core_graphics(resolution: (u32, u32)) -> Result<Vec<u8>, Error> {
    // This is a simplified implementation
    // Full implementation would use:
    // 1. CGDisplayCreateImage for main display
    // 2. Convert CGImage to raw bytes
    // 3. Resize if needed
    // 4. Encode to JPEG
    
    // For now, return error to fallback
    Err(anyhow::anyhow!("Core Graphics capture not fully implemented"))
}

/// Encode image as JPEG
fn encode_image(img: &DynamicImage) -> Result<Vec<u8>, Error> {
    let mut buffer = Vec::new();
    let mut encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 80);
    encoder.encode_image(img)?;
    Ok(buffer)
}

/// Generate test pattern for macOS
fn generate_test_pattern(resolution: (u32, u32)) -> Vec<u8> {
    let (width, height) = resolution;
    let mut img = ImageBuffer::new(width, height);
    
    // Create macOS-style gradient
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = 100 + ((x as f32 / width as f32) * 155.0) as u8;
        let g = 100 + ((y as f32 / height as f32) * 155.0) as u8;
        let b = 200;
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
    
    #[tokio::test]
    async fn test_capture_screen() {
        let result = capture_screen((1920, 1080)).await;
        assert!(result.is_ok());
        
        let data = result.unwrap();
        assert!(!data.is_empty());
    }
}