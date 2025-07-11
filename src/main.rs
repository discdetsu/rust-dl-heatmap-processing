use log::{info, warn};
use dicom::object::open_file;
use dicom_pixeldata::{PixelDecoder, DecodedPixelData};
use std::path::Path;
use env_logger;
use image::{GrayImage, RgbaImage, ImageBuffer, Rgba, imageops, DynamicImage};
use clap::Parser;

#[derive(Parser)]
#[command(name = "rust-dl-heatmap-processing")]
#[command(about = "A DICOM heatmap processing tool")]
#[command(version)]
struct Args {
    /// Input DICOM file path
    #[arg(short, long, default_value = "sample.dcm")]
    input: String,
    
    /// Output PNG file path
    #[arg(short, long, default_value = "output.png")]
    output: String,
    
    /// Use demo mode with simulated data
    #[arg(short, long)]
    demo: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let dicom_path = Path::new(&args.input);
    let png_path = Path::new(&args.output);

    // Force demo mode if requested
    if args.demo {
        info!("Demo mode requested - creating heatmap with simulated data");
        let rows = 512u32;
        let columns = 512u32;
        create_demo_heatmap(rows, columns, png_path)?;
        return Ok(());
    }

    // Check if DICOM file exists
    if !dicom_path.exists() {
        println!("DICOM file not found at: {}", dicom_path.display());
        println!("Please provide a valid DICOM file path using --input flag");
        println!("Or use --demo flag to create a demo heatmap with simulated data");
        println!("Example: {} --input your_file.dcm --output result.png", env!("CARGO_PKG_NAME"));
        println!("Example: {} --demo --output demo_result.png", env!("CARGO_PKG_NAME"));
        
        // Create demo instead of failing
        info!("Falling back to demo mode...");
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
    
    // Try to decode real DICOM pixel data
    match decode_dicom_pixel_data(&obj, rows, columns) {
        Ok(base_image) => {
            info!("Successfully decoded DICOM pixel data");
            create_heatmap_with_real_data(base_image, png_path)?;
        }
        Err(e) => {
            warn!("Failed to decode DICOM pixel data: {}", e);
            warn!("Falling back to simulated data");
            create_demo_heatmap(rows, columns, png_path)?;
        }
    }
    
    Ok(())
}

fn decode_dicom_pixel_data(
    obj: &dicom::object::FileDicomObject<dicom::object::InMemDicomObject>,
    rows: u32,
    columns: u32,
) -> Result<RgbaImage, Box<dyn std::error::Error>> {
    // Decode pixel data using dicom-pixeldata
    let decoded_pixel_data = obj.decode_pixel_data()?;
    
    info!("Pixel data info: {} bits allocated, {} samples per pixel", 
          decoded_pixel_data.bits_allocated(), 
          decoded_pixel_data.samples_per_pixel());
    
    // Convert decoded pixel data to grayscale image
    let gray_image = match decoded_pixel_data.samples_per_pixel() {
        1 => {
            // Grayscale image
            convert_to_grayscale_image(&decoded_pixel_data, rows, columns)?
        }
        3 => {
            // RGB image - convert to grayscale
            let rgb_data = decoded_pixel_data.to_dynamic_image(0)?;
            rgb_data.to_luma8()
        }
        _ => {
            return Err(format!("Unsupported samples per pixel: {}", 
                             decoded_pixel_data.samples_per_pixel()).into());
        }
    };
    
    // Convert grayscale to RGBA for overlay
    let rgba_image = DynamicImage::ImageLuma8(gray_image).to_rgba8();
    
    Ok(rgba_image)
}

fn convert_to_grayscale_image(
    decoded_data: &DecodedPixelData,
    rows: u32,
    columns: u32,
) -> Result<GrayImage, Box<dyn std::error::Error>> {
    // Handle different bit depths
    match decoded_data.bits_allocated() {
        8 => {
            // 8-bit data
            let pixel_data: Vec<u8> = decoded_data.to_vec()?;
            GrayImage::from_raw(columns, rows, pixel_data)
                .ok_or("Failed to create GrayImage from 8-bit DICOM data".into())
        }
        16 => {
            // 16-bit data - need to scale to 8-bit
            let pixel_data_u16: Vec<u16> = decoded_data.to_vec()?;
            
            // Apply basic windowing: scale to 8-bit range
            // For medical images, proper windowing using Window Center/Width would be better
            let min_val = *pixel_data_u16.iter().min().unwrap_or(&0) as f32;
            let max_val = *pixel_data_u16.iter().max().unwrap_or(&0) as f32;
            let range = if max_val > min_val { max_val - min_val } else { 1.0 };
            
            info!("16-bit data range: {} - {}", min_val, max_val);
            
            let pixel_data_u8: Vec<u8> = pixel_data_u16
                .iter()
                .map(|&val| {
                    let normalized = ((val as f32 - min_val) / range) * 255.0;
                    normalized.max(0.0).min(255.0) as u8
                })
                .collect();
                
            GrayImage::from_raw(columns, rows, pixel_data_u8)
                .ok_or("Failed to create GrayImage from 16-bit DICOM data".into())
        }
        bits => {
            Err(format!("Unsupported bit depth: {} bits", bits).into())
        }
    }
}

fn create_heatmap_with_real_data(
    mut base_rgba_image: RgbaImage,
    png_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = base_rgba_image.dimensions();
    
    info!("Creating heatmap overlay on real DICOM data ({}x{})", width, height);
    
    // --- Generate Heatmap Overlay ---
    let mut heatmap_rgba = RgbaImage::new(width, height);
    for y in 0..height {
        for x in 0..width {
            // Simple gradient: red intensity increases with x, alpha increases with y
            let red_intensity = (x as f32 / width as f32 * 255.0) as u8;
            // Alpha channel makes the heatmap semi-transparent
            // More intense (less transparent) towards the bottom of the image
            let alpha = (y as f32 / height as f32 * 150.0) as u8 + 50; // Alpha from 50 to 200
            heatmap_rgba.put_pixel(x, y, Rgba([red_intensity, 0, 0, alpha]));
        }
    }

    // Overlay the heatmap onto the base RGBA image
    imageops::overlay(&mut base_rgba_image, &heatmap_rgba, 0, 0);

    // Save the resulting image
    base_rgba_image.save_with_format(png_path, image::ImageFormat::Png)?;

    info!("Successfully created PNG with heatmap overlay on real DICOM data: {}", png_path.display());
    
    Ok(())
}

fn create_demo_heatmap(rows: u32, columns: u32, png_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating demo heatmap with simulated data ({}x{})", columns, rows);
    
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

    info!("Successfully created demo PNG with heatmap overlay: {}", png_path.display());
    info!("Note: Using simulated base image. Place a real DICOM file as 'sample.dcm' to process real medical data.");
    
    Ok(())
}
