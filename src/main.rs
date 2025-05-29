use log::{info, error};
use dicom::object::open_file;
use dicom::pixeldata::PixelData;
use png::Encoder;
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;
use env_logger;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Specify the input DICOM file and output PNG file paths
    let dicom_path = Path::new("input.dcm"); // TODO: Replace with actual DICOM file path
    let png_path = Path::new("output.png"); // TODO: Replace with desired output PNG file path

    // Open the DICOM file
    let obj = open_file(&dicom_path)?;

    // Get the pixel data
    let pixel_data = obj.pixel_data()?;

    // Get the image dimensions
    let rows = obj.element_by_name("Rows")?.to_int::<u32>()?;
    let columns = obj.element_by_name("Columns")?.to_int::<u32>()?;

    // Create the output PNG file
    let file = File::create(png_path)?;
    let w = BufWriter::new(file);

    // Create a PNG encoder
    let mut encoder = Encoder::new(w, columns, rows);
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight); // Assuming 8-bit grayscale, adjust if necessary

    let mut writer = encoder.write_header()?;

    // Write the pixel data to the PNG file
    // This assumes the pixel data is 8-bit grayscale.
    // You may need to adjust this based on the actual DICOM pixel data format.
    match pixel_data {
        PixelData::U8(data) => {
            writer.write_image_data(&data)?;
        }
        PixelData::U16(data) => {
            // Convert U16 to U8. This is a simple scaling, more sophisticated methods might be needed.
            let mut u8_data: Vec<u8> = Vec::with_capacity(data.len());
            for &val in data.iter() {
                // Example: Scale 0-65535 to 0-255.
                // This might not be the best way to handle all cases.
                // Windowing/leveling might be necessary for proper visualization.
                u8_data.push((val / 256) as u8);
            }
            writer.write_image_data(&u8_data)?;
        }
        _ => {
            error!("Unsupported pixel data format");
            // You might want to return an error here or handle other formats.
            return Err("Unsupported pixel data format".into());
        }
    }

    info!("Successfully converted DICOM to PNG: {}", png_path.display());

    Ok(())
}
