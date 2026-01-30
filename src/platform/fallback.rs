use anyhow::Error;
use image::{ImageBuffer, Rgba};

/// Fallback screen capture for unsupported platforms
pub async fn capture_screen(resolution: (u32, u32)) -> Result<Vec<u8>, Error> {
    Ok(generate_test_pattern(resolution))
}

/// Generate test pattern
fn generate_test_pattern(resolution: (u32, u32)) -> Vec<u8> {
    let (width, height) = resolution;
    let mut img = ImageBuffer::new(width, height);
    
    for (x, y, pixel) in img.enumerate_pixels_mut() {
        let r = ((x as f32 / width as f32) * 255.0) as u8;
        let g = ((y as f32 / height as f32) * 255.0) as u8;
        let b = (((x + y) as f32 / (width + height) as f32) * 255.0) as u8;
        *pixel = Rgba([r, g, b, 255]);
    }
    
    // Convert to JPEG
    let mut buffer = Vec::new();
    let encoder = image::codecs::jpeg::JpegEncoder::new_with_quality(&mut buffer, 70);
    encoder.encode(&img, width, height, image::ColorType::Rgba8).unwrap();
    
    buffer
}