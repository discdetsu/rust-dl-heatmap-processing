use log::{info, error};
use dicom::object::open_file;
use dicom::pixeldata::PixelData;
use std::fs::File;
use std::path::Path;
use env_logger;
use image::{GrayImage, RgbaImage, ImageBuffer, Rgba, imageops};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let dicom_path = Path::new("input.dcm"); // TODO: Replace with actual DICOM file path
    let png_path = Path::new("output.png"); // TODO: Replace with desired output PNG file path

    let obj = open_file(&dicom_path)?;
    let pixel_data_element = obj.pixel_data()?;
    let rows = obj.element_by_name("Rows")?.to_int::<u32>()?;
    let columns = obj.element_by_name("Columns")?.to_int::<u32>()?;

    let mut image_data_u8: Vec<u8> = Vec::new();

    match pixel_data_element {
        PixelData::U8(data) => {
            image_data_u8 = data.to_vec();
        }
        PixelData::U16(data) => {
            image_data_u8 = Vec::with_capacity(data.len());
            // Basic windowing/leveling: find min/max and scale to 0-255
            // This is a simplified approach. For medical images, more precise
            // window center and width values from DICOM tags should be used if available.
            let min_val = *data.iter().min().unwrap_or(&0) as f32;
            let max_val = *data.iter().max().unwrap_or(&0) as f32;
            let range = if max_val > min_val { max_val - min_val } else { 1.0 };

            for &val in data.iter() {
                let normalized = ((val as f32 - min_val) / range) * 255.0;
                image_data_u8.push(normalized.max(0.0).min(255.0) as u8);
            }
        }
        _ => {
            error!("Unsupported pixel data format for heatmap generation.");
            return Err("Unsupported pixel data format".into());
        }
    }

    // Create a grayscale image from the DICOM data
    let gray_image: GrayImage = ImageBuffer::from_raw(columns, rows, image_data_u8)
        .ok_or_else(|| "Failed to create GrayImage from DICOM pixel data")?;

    // Convert grayscale to RGBA to allow for color overlay
    let mut base_rgba_image: RgbaImage = imageops::colorops::grayscale_to_rgba(&gray_image);

    // --- Generate Mockup Heatmap ---
    let mut heatmap_rgba = RgbaImage::new(columns, rows);
    for y in 0..rows {
        for x in 0..columns {
            // Simple gradient: red intensity increases with x, alpha increases with y
            let red_intensity = (x as f32 / columns as f32 * 255.0) as u8;
            // Alpha channel makes the heatmap semi-transparent
            // More intense (less transparent) towards the bottom of the image
            let alpha = (y as f32 / rows as f32 * 150.0) as u8 + 50; // Alpha from 50 to 200
            heatmap_rgba.put_pixel(x, y, Rgba([red_intensity, 0, 0, alpha]));
        }
    }

    // Overlay the heatmap onto the base RGBA image
    imageops::overlay(&mut base_rgba_image, &heatmap_rgba, 0, 0);

    // Save the resulting image
    base_rgba_image.save_with_format(png_path, image::ImageFormat::Png)?;

    info!("Successfully converted DICOM to PNG with heatmap overlay: {}", png_path.display());

    Ok(())
}
