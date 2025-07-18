use log::{info, warn};
use dicom::object::open_file;
use dicom_pixeldata::{PixelDecoder, DecodedPixelData};
use std::path::Path;
use env_logger;
use image::{GrayImage, RgbaImage, ImageBuffer, Rgba, imageops, DynamicImage};
use clap::Parser;
use ndarray::Array2;
use std::fs::File;
use std::io::Read;

#[derive(Parser)]
#[command(name = "rust-dl-heatmap-processing")]
#[command(about = "A DICOM heatmap processing tool with ML model integration")]
#[command(version)]
struct Args {
    /// Input DICOM file path
    #[arg(short, long, default_value = "sample.dcm")]
    input: String,
    
    /// Output PNG file path
    #[arg(short, long, default_value = "output.png")]
    output: String,
    
    /// Heatmap data file (.npy, .json, .csv, or .bin)
    #[arg(long)]
    heatmap: Option<String>,
    
    /// Color scheme for heatmap (red, hot, jet, viridis, plasma)
    #[arg(long, default_value = "red")]
    colormap: String,
    
    /// Heatmap opacity (0.0 to 1.0)
    #[arg(long, default_value = "0.6")]
    opacity: f32,
    
    /// Normalization method (minmax, zscore, percentile)
    #[arg(long, default_value = "minmax")]
    normalization: String,
    
    /// Use demo mode with simulated data
    #[arg(short, long)]
    demo: bool,
}

#[derive(Debug, Clone)]
pub enum ColorMap {
    Red,
    Hot,
    Jet,
    Viridis,
    Plasma,
}

impl ColorMap {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "red" => Ok(ColorMap::Red),
            "hot" => Ok(ColorMap::Hot),
            "jet" => Ok(ColorMap::Jet),
            "viridis" => Ok(ColorMap::Viridis),
            "plasma" => Ok(ColorMap::Plasma),
            _ => Err(format!("Unknown colormap: {}. Available: red, hot, jet, viridis, plasma", s)),
        }
    }
}

#[derive(Debug, Clone)]
pub enum Normalization {
    MinMax,
    ZScore,
    Percentile,
}

impl Normalization {
    fn from_str(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "minmax" => Ok(Normalization::MinMax),
            "zscore" => Ok(Normalization::ZScore),
            "percentile" => Ok(Normalization::Percentile),
            _ => Err(format!("Unknown normalization: {}. Available: minmax, zscore, percentile", s)),
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();
    let dicom_path = Path::new(&args.input);
    let png_path = Path::new(&args.output);

    // Parse colormap and normalization options
    let colormap = ColorMap::from_str(&args.colormap)?;
    let normalization = Normalization::from_str(&args.normalization)?;
    
    // Validate opacity range
    if args.opacity < 0.0 || args.opacity > 1.0 {
        return Err("Opacity must be between 0.0 and 1.0".into());
    }

    // Force demo mode if requested
    if args.demo {
        info!("Demo mode requested - creating heatmap with simulated data");
        let rows = 512u32;
        let columns = 512u32;
        create_demo_heatmap(rows, columns, png_path, &colormap, args.opacity)?;
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
        create_demo_heatmap(rows, columns, png_path, &colormap, args.opacity)?;
        return Ok(());
    }

    let obj = open_file(&dicom_path)?;
    
    // Get basic image information
    let rows = obj.element_by_name("Rows")?.to_int::<u32>()?;
    let columns = obj.element_by_name("Columns")?.to_int::<u32>()?;
    
    info!("DICOM image dimensions: {}x{}", columns, rows);
    
    // Load heatmap data if provided
    let heatmap_data = if let Some(heatmap_path) = &args.heatmap {
        match load_heatmap_data(heatmap_path) {
            Ok(data) => {
                info!("Successfully loaded heatmap data: {}x{}", data.nrows(), data.ncols());
                Some(data)
            }
            Err(e) => {
                warn!("Failed to load heatmap data: {}", e);
                warn!("Proceeding without heatmap overlay");
                None
            }
        }
    } else {
        None
    };
    
    // Try to decode real DICOM pixel data
    match decode_dicom_pixel_data(&obj, rows, columns) {
        Ok(base_image) => {
            info!("Successfully decoded DICOM pixel data");
            create_heatmap_with_real_data(
                base_image, 
                png_path, 
                heatmap_data, 
                &colormap, 
                &normalization, 
                args.opacity
            )?;
        }
        Err(e) => {
            warn!("Failed to decode DICOM pixel data: {}", e);
            warn!("Falling back to simulated data");
            create_demo_heatmap(rows, columns, png_path, &colormap, args.opacity)?;
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
    heatmap_data: Option<Array2<f32>>,
    colormap: &ColorMap,
    normalization: &Normalization,
    opacity: f32,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = base_rgba_image.dimensions();
    
    info!("Creating heatmap overlay on real DICOM data ({}x{})", width, height);
    
    let heatmap_rgba = if let Some(data) = heatmap_data {
        // Use real heatmap data
        info!("Using real heatmap data with {} colormap and {} normalization", 
              format!("{:?}", colormap).to_lowercase(), 
              format!("{:?}", normalization).to_lowercase());
        
        // Resize heatmap data to match image dimensions if needed
        let resized_data = if data.nrows() != height as usize || data.ncols() != width as usize {
            warn!("Heatmap dimensions ({}x{}) don't match image dimensions ({}x{}), resizing...", 
                  data.nrows(), data.ncols(), height, width);
            resize_heatmap(&data, width as usize, height as usize)
        } else {
            data
        };
        
        // Normalize the data
        let normalized_data = normalize_heatmap(&resized_data, normalization);
        
        // Apply colormap
        apply_colormap(&normalized_data, colormap, opacity)
    } else {
        // Generate default gradient heatmap
        info!("No heatmap data provided, generating default gradient with {} colormap", 
              format!("{:?}", colormap).to_lowercase());
        generate_default_heatmap(width, height, colormap, opacity)
    };

    // Overlay the heatmap onto the base RGBA image
    imageops::overlay(&mut base_rgba_image, &heatmap_rgba, 0, 0);

    // Save the resulting image
    base_rgba_image.save_with_format(png_path, image::ImageFormat::Png)?;

    info!("Successfully created PNG with heatmap overlay on real DICOM data: {}", png_path.display());
    
    Ok(())
}

/// Resize heatmap data to match target dimensions using nearest neighbor interpolation
fn resize_heatmap(data: &Array2<f32>, target_width: usize, target_height: usize) -> Array2<f32> {
    let (src_height, src_width) = data.dim();
    let mut resized = Array2::zeros((target_height, target_width));
    
    for row in 0..target_height {
        for col in 0..target_width {
            let src_row = ((row as f32 / target_height as f32) * src_height as f32) as usize;
            let src_col = ((col as f32 / target_width as f32) * src_width as f32) as usize;
            
            let src_row = src_row.min(src_height - 1);
            let src_col = src_col.min(src_width - 1);
            
            resized[[row, col]] = data[[src_row, src_col]];
        }
    }
    
    resized
}

/// Generate default gradient heatmap when no real data is provided
fn generate_default_heatmap(width: u32, height: u32, colormap: &ColorMap, opacity: f32) -> RgbaImage {
    let mut heatmap_rgba = RgbaImage::new(width, height);
    
    for y in 0..height {
        for x in 0..width {
            // Simple gradient: intensity increases with x and y
            let value = ((x as f32 / width as f32) + (y as f32 / height as f32)) / 2.0;
            let color = get_color_from_value(value, colormap);
            let alpha = (opacity * 255.0) as u8;
            
            heatmap_rgba.put_pixel(x, y, Rgba([color.0, color.1, color.2, alpha]));
        }
    }
    
    heatmap_rgba
}

/// Load heatmap data from various file formats
fn load_heatmap_data(file_path: &str) -> Result<Array2<f32>, Box<dyn std::error::Error>> {
    let path = Path::new(file_path);
    let extension = path.extension()
        .and_then(|ext| ext.to_str())
        .ok_or("Could not determine file extension")?
        .to_lowercase();
    
    info!("Loading heatmap data from: {} (format: {})", file_path, extension);
    
    match extension.as_str() {
        "npy" => Err("NPY format support coming soon! Please use .json, .csv, or .bin format for now.".into()),
        "json" => load_json_heatmap(file_path),
        "csv" => load_csv_heatmap(file_path),
        "bin" => load_binary_heatmap(file_path),
        _ => Err(format!("Unsupported heatmap file format: {}. Supported: .json, .csv, .bin", extension).into()),
    }
}

/// Load heatmap from .json file
/// Expected format: {"data": [[1.0, 2.0], [3.0, 4.0]], "shape": [2, 2]}
fn load_json_heatmap(file_path: &str) -> Result<Array2<f32>, Box<dyn std::error::Error>> {
    let mut file = File::open(file_path)?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    
    let parsed: serde_json::Value = serde_json::from_str(&contents)?;
    
    // Try to extract data as nested arrays
    if let Some(data_array) = parsed.get("data").and_then(|v| v.as_array()) {
        let mut flat_data = Vec::new();
        let rows = data_array.len();
        let mut cols = 0;
        
        for (i, row) in data_array.iter().enumerate() {
            if let Some(row_array) = row.as_array() {
                if i == 0 {
                    cols = row_array.len();
                }
                for val in row_array {
                    if let Some(num) = val.as_f64() {
                        flat_data.push(num as f32);
                    } else {
                        return Err("JSON data must contain numeric values".into());
                    }
                }
            } else {
                return Err("JSON data must be array of arrays".into());
            }
        }
        
        Array2::from_shape_vec((rows, cols), flat_data)
            .map_err(|e| e.into())
    } else {
        Err("JSON must contain 'data' field with array of arrays".into())
    }
}

/// Load heatmap from .csv file
fn load_csv_heatmap(file_path: &str) -> Result<Array2<f32>, Box<dyn std::error::Error>> {
    let mut reader = csv::Reader::from_path(file_path)?;
    let mut data = Vec::new();
    let mut rows = 0;
    let mut cols = 0;
    
    for result in reader.records() {
        let record = result?;
        if rows == 0 {
            cols = record.len();
        }
        
        for field in &record {
            let value: f32 = field.parse()
                .map_err(|_| format!("Could not parse '{}' as number", field))?;
            data.push(value);
        }
        rows += 1;
    }
    
    if data.is_empty() {
        return Err("CSV file is empty".into());
    }
    
    Array2::from_shape_vec((rows, cols), data)
        .map_err(|e| e.into())
}

/// Normalize heatmap data using different methods
fn normalize_heatmap(data: &Array2<f32>, method: &Normalization) -> Array2<f32> {
    match method {
        Normalization::MinMax => {
            let min_val = data.iter().fold(f32::INFINITY, |a, &b| a.min(b));
            let max_val = data.iter().fold(f32::NEG_INFINITY, |a, &b| a.max(b));
            let range = max_val - min_val;
            
            if range == 0.0 {
                data.clone()
            } else {
                data.mapv(|x| (x - min_val) / range)
            }
        }
        Normalization::ZScore => {
            let mean = data.mean().unwrap_or(0.0);
            let variance = data.mapv(|x| (x - mean).powi(2)).mean().unwrap_or(1.0);
            let std_dev = variance.sqrt();
            
            if std_dev == 0.0 {
                data.clone()
            } else {
                data.mapv(|x| (x - mean) / std_dev)
            }
        }
        Normalization::Percentile => {
            let mut sorted_values: Vec<f32> = data.iter().cloned().collect();
            sorted_values.sort_by(|a, b| a.partial_cmp(b).unwrap());
            
            let len = sorted_values.len();
            let p5_idx = (0.05 * len as f32) as usize;
            let p95_idx = (0.95 * len as f32) as usize;
            
            let p5_val = sorted_values[p5_idx];
            let p95_val = sorted_values[p95_idx];
            let range = p95_val - p5_val;
            
            if range == 0.0 {
                data.clone()
            } else {
                data.mapv(|x| ((x - p5_val) / range).max(0.0).min(1.0))
            }
        }
    }
}

/// Apply colormap to normalized heatmap data
fn apply_colormap(normalized_data: &Array2<f32>, colormap: &ColorMap, opacity: f32) -> RgbaImage {
    let (rows, cols) = normalized_data.dim();
    let mut heatmap_rgba = RgbaImage::new(cols as u32, rows as u32);
    
    for row in 0..rows {
        for col in 0..cols {
            let value = normalized_data[[row, col]];
            let color = get_color_from_value(value, colormap);
            let alpha = (opacity * 255.0) as u8;
            
            heatmap_rgba.put_pixel(
                col as u32,
                row as u32,
                Rgba([color.0, color.1, color.2, alpha]),
            );
        }
    }
    
    heatmap_rgba
}

/// Get RGB color from normalized value [0,1] using specified colormap
fn get_color_from_value(value: f32, colormap: &ColorMap) -> (u8, u8, u8) {
    let value = value.max(0.0).min(1.0); // Clamp to [0,1]
    
    match colormap {
        ColorMap::Red => {
            // Simple red gradient
            let intensity = (value * 255.0) as u8;
            (intensity, 0, 0)
        }
        ColorMap::Hot => {
            // Hot colormap: black -> red -> yellow -> white
            if value < 0.33 {
                let t = value / 0.33;
                ((t * 255.0) as u8, 0, 0)
            } else if value < 0.66 {
                let t = (value - 0.33) / 0.33;
                (255, (t * 255.0) as u8, 0)
            } else {
                let t = (value - 0.66) / 0.34;
                (255, 255, (t * 255.0) as u8)
            }
        }
        ColorMap::Jet => {
            // Jet colormap: blue -> cyan -> yellow -> red
            if value < 0.25 {
                let t = value / 0.25;
                (0, (t * 255.0) as u8, 255)
            } else if value < 0.5 {
                let t = (value - 0.25) / 0.25;
                (0, 255, (255.0 * (1.0 - t)) as u8)
            } else if value < 0.75 {
                let t = (value - 0.5) / 0.25;
                ((t * 255.0) as u8, 255, 0)
            } else {
                let t = (value - 0.75) / 0.25;
                (255, (255.0 * (1.0 - t)) as u8, 0)
            }
        }
        ColorMap::Viridis => {
            // Simplified viridis: purple -> blue -> green -> yellow
            if value < 0.33 {
                let t = value / 0.33;
                ((68.0 + t * (59.0 - 68.0)) as u8, (1.0 + t * (82.0 - 1.0)) as u8, (84.0 + t * (139.0 - 84.0)) as u8)
            } else if value < 0.66 {
                let t = (value - 0.33) / 0.33;
                ((59.0 + t * (33.0 - 59.0)) as u8, (82.0 + t * (144.0 - 82.0)) as u8, (139.0 + t * (140.0 - 139.0)) as u8)
            } else {
                let t = (value - 0.66) / 0.34;
                ((33.0 + t * (253.0 - 33.0)) as u8, (144.0 + t * (231.0 - 144.0)) as u8, (140.0 + t * (37.0 - 140.0)) as u8)
            }
        }
        ColorMap::Plasma => {
            // Simplified plasma: purple -> pink -> yellow
            if value < 0.5 {
                let t = value / 0.5;
                ((13.0 + t * (190.0 - 13.0)) as u8, (8.0 + t * (84.0 - 8.0)) as u8, (135.0 + t * (160.0 - 135.0)) as u8)
            } else {
                let t = (value - 0.5) / 0.5;
                ((190.0 + t * (240.0 - 190.0)) as u8, (84.0 + t * (249.0 - 84.0)) as u8, (160.0 + t * (33.0 - 160.0)) as u8)
            }
        }
    }
}

/// Load heatmap from binary file (assumes f32 values in row-major order)
/// File should start with 8 bytes: 4 bytes for rows (u32), 4 bytes for cols (u32)
fn load_binary_heatmap(file_path: &str) -> Result<Array2<f32>, Box<dyn std::error::Error>> {
    use byteorder::{LittleEndian, ReadBytesExt};
    
    let mut file = File::open(file_path)?;
    
    // Read dimensions
    let rows = file.read_u32::<LittleEndian>()? as usize;
    let cols = file.read_u32::<LittleEndian>()? as usize;
    
    info!("Binary heatmap dimensions: {}x{}", rows, cols);
    
    // Read data
    let mut data = Vec::with_capacity(rows * cols);
    for _ in 0..(rows * cols) {
        data.push(file.read_f32::<LittleEndian>()?);
    }
    
    Array2::from_shape_vec((rows, cols), data)
        .map_err(|e| e.into())
}

fn create_demo_heatmap(rows: u32, columns: u32, png_path: &Path, colormap: &ColorMap, opacity: f32) -> Result<(), Box<dyn std::error::Error>> {
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

    // Generate demo heatmap with specified colormap
    let heatmap_rgba = generate_default_heatmap(columns, rows, colormap, opacity);

    // Overlay the heatmap onto the base RGBA image
    imageops::overlay(&mut base_rgba_image, &heatmap_rgba, 0, 0);

    // Save the resulting image
    base_rgba_image.save_with_format(png_path, image::ImageFormat::Png)?;

    info!("Successfully created demo PNG with {} heatmap overlay: {}", 
          format!("{:?}", colormap).to_lowercase(), png_path.display());
    info!("Note: Using simulated base image. Place a real DICOM file as 'sample.dcm' to process real medical data.");
    
    Ok(())
}
