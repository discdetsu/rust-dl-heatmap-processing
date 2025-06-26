use log::info;
use dicom::object::open_file;
use std::path::Path;
use env_logger;
use image::{GrayImage, RgbaImage, ImageBuffer, Rgba, imageops, DynamicImage};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let dicom_path = Path::new("sample.dcm"); // TODO: Replace with actual DICOM file path
    let png_path = Path::new("output.png");

    // Check if DICOM file exists
    if !dicom_path.exists() {
        println!("DICOM file not found at: {}", dicom_path.display());
        println!("Please provide a valid DICOM file path in the code or as a command line argument.");
        println!("For now, creating a demo heatmap with simulated data...");
        
        // Use simulated dimensions for demo
        let rows = 512u32;
        let columns = 512u32;
        
        create_demo_heatmap(rows, columns, png_path)?;
        return Ok(());
    }

    let obj = open_file(&dicom_path)?;
    
    // Get basic image information
    let rows = obj.element_by_name("Rows")?.to_int::<u32>()?;
    let columns = obj.element_by_name("Columns")?.to_int::<u32>()?;
    
    info!("DICOM image dimensions: {}x{}", columns, rows);
    
    create_demo_heatmap(rows, columns, png_path)?;
    
    Ok(())
}

fn create_demo_heatmap(rows: u32, columns: u32, png_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Create a simple gradient as a base image (simulating DICOM data)
    let mut image_data_u8: Vec<u8> = Vec::with_capacity((rows * columns) as usize);
    for y in 0..rows {
        for x in 0..columns {
            // Create a simple gradient pattern
            let intensity = ((x + y) as f32 / (columns + rows) as f32 * 255.0) as u8;
            image_data_u8.push(intensity);
        }
    }

    // Create a grayscale image from the simulated data
    let gray_image: GrayImage = ImageBuffer::from_raw(columns, rows, image_data_u8)
        .ok_or_else(|| "Failed to create GrayImage from simulated data")?;

    // Convert grayscale to RGBA to allow for color overlay
    let mut base_rgba_image: RgbaImage = DynamicImage::ImageLuma8(gray_image).to_rgba8();

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

    info!("Successfully created PNG with heatmap overlay: {}", png_path.display());
    info!("Note: Using simulated base image. To use actual DICOM pixel data, additional DICOM processing is needed.");
    
    Ok(())
}
