# rust-dl-heatmap-processing

A Rust-based DICOM heatmap processing tool that efficiently processes real medical imaging data and overlays heatmaps generated from deep learning models, leveraging the performance and safety of Rust.

## Features

*   **Real DICOM Processing**: Extract and process actual pixel data from DICOM medical images
*   **Multiple Bit Depths**: Support for 8-bit and 16-bit DICOM pixel data with automatic windowing
*   **Image Format Support**: Handle both grayscale and RGB DICOM images
*   **🔥 ML Heatmap Integration**: Load and visualize actual ML model outputs
*   **Multiple Data Formats**: Support for JSON, CSV, and binary heatmap data files
*   **Advanced Color Schemes**: 5 scientific colormaps (Red, Hot, Jet, Viridis, Plasma)
*   **Smart Normalization**: MinMax, Z-Score, and Percentile normalization methods
*   **Configurable Opacity**: Adjustable heatmap transparency (0.0-1.0)
*   **Automatic Resizing**: Smart resizing when heatmap dimensions don't match image
*   **Command-Line Interface**: Easy-to-use CLI with flexible input/output options
*   **Demo Mode**: Create sample heatmaps with simulated data for testing

## Installation

1.  **Prerequisites:**
    *   Ensure you have Rust installed. You can get it from [rustup.rs](https://rustup.rs/).

2.  **Clone the repository:**
    ```bash
    git clone https://github.com/your-username/rust-dl-heatmap-processing.git
    cd rust-dl-heatmap-processing
    ```

3.  **Build the project:**
    ```bash
    cargo build --release
    ```

## Usage

### Basic Commands

```bash
# Process a DICOM file (with fallback to demo if file not found)
cargo run

# Process a specific DICOM file with ML heatmap overlay
cargo run -- --input scan.dcm --heatmap model_output.json --output result.png

# Generate demo with different colormaps
cargo run -- --demo --colormap viridis --opacity 0.8 --output demo.png
```

### Command-Line Options

- `-i, --input <FILE>`: Input DICOM file path (default: `sample.dcm`)
- `-o, --output <FILE>`: Output PNG file path (default: `output.png`)
- `--heatmap <FILE>`: Heatmap data file (.json, .csv, .bin) *[NEW!]*
- `--colormap <SCHEME>`: Color scheme (red, hot, jet, viridis, plasma) *[NEW!]*
- `--opacity <VALUE>`: Heatmap opacity 0.0-1.0 (default: 0.6) *[NEW!]*
- `--normalization <METHOD>`: Normalization (minmax, zscore, percentile) *[NEW!]*
- `-d, --demo`: Use demo mode with simulated data
- `-h, --help`: Print help information
- `-V, --version`: Print version information

### 🔥 ML Heatmap Integration Examples

#### JSON Format
```bash
# Create heatmap data file
echo '{
  "data": [
    [0.1, 0.3, 0.8, 0.6],
    [0.4, 0.9, 0.7, 0.2],
    [0.6, 0.5, 0.3, 0.8],
    [0.2, 0.7, 0.9, 0.4]
  ]
}' > ml_heatmap.json

# Apply to DICOM with Viridis colormap
cargo run -- -i medical_scan.dcm --heatmap ml_heatmap.json --colormap viridis --output result.png
```

#### CSV Format
```bash
# Create CSV heatmap data
echo "0.1,0.3,0.8,0.6
0.4,0.9,0.7,0.2
0.6,0.5,0.3,0.8
0.2,0.7,0.9,0.4" > ml_heatmap.csv

# Apply with Jet colormap and custom opacity
cargo run -- -i scan.dcm --heatmap ml_heatmap.csv --colormap jet --opacity 0.8
```

#### Advanced Usage
```bash
# High-contrast visualization with percentile normalization
cargo run -- -i brain_scan.dcm --heatmap attention_map.json \
  --colormap plasma --opacity 0.7 --normalization percentile

# Hot colormap for anomaly detection
cargo run -- -i chest_xray.dcm --heatmap anomaly_scores.csv \
  --colormap hot --opacity 0.9 --normalization zscore
```

## How It Works

1. **DICOM Reading**: Opens and parses DICOM files using the `dicom-rs` ecosystem
2. **Pixel Data Extraction**: Extracts real pixel data using `dicom-pixeldata` with support for:
   - 8-bit and 16-bit pixel data
   - Grayscale and RGB images
   - Automatic bit-depth conversion and windowing
3. **🔥 ML Data Loading**: Loads heatmap data from various formats:
   - **JSON**: Nested arrays with numeric values
   - **CSV**: Comma-separated numeric values
   - **Binary**: Raw f32 data with dimension headers
4. **Smart Processing**: 
   - Automatic resizing when dimensions don't match
   - Multiple normalization methods for optimal visualization
   - Scientific colormaps for accurate representation
5. **Base Image Creation**: Converts DICOM pixel data to RGBA format
6. **Heatmap Generation**: Creates colored, semi-transparent overlays from real data
7. **Image Composition**: Overlays the heatmap onto the medical image
8. **PNG Export**: Saves the final result as a PNG file

## Technical Details

### Supported Heatmap Formats
- **JSON**: `{"data": [[1.0, 2.0], [3.0, 4.0]]}`
- **CSV**: Comma-separated values in row-major order
- **Binary**: f32 values with 8-byte header (rows, cols as u32)
- **NPY**: *Coming soon!*

### Scientific Colormaps
- **Red**: Simple red intensity gradient
- **Hot**: Black → Red → Yellow → White (temperature-like)
- **Jet**: Blue → Cyan → Yellow → Red (classic scientific)
- **Viridis**: Purple → Blue → Green → Yellow (perceptually uniform)
- **Plasma**: Purple → Pink → Yellow (high contrast)

### Normalization Methods
- **MinMax**: Scale to [0,1] using data range
- **Z-Score**: Standard score normalization
- **Percentile**: 5th-95th percentile clipping

### Dependencies
- `dicom` v0.8.1 - Core DICOM processing
- `dicom-pixeldata` v0.8.1 - Pixel data decoding with image support
- `ndarray` v0.16.1 - Array operations for heatmap processing
- `image` v0.25.1 - Image processing and PNG output
- `csv` v1.3.1 - CSV file parsing
- `npyz` v0.8.4 - NPY file support (coming soon)
- `clap` v4.5.41 - Command-line argument parsing
- `log` & `env_logger` - Logging support

### Performance
- **Memory Efficient**: Processes images in-memory with minimal allocations
- **Safe Processing**: Rust's memory safety prevents common image processing errors
- **Fast Execution**: Optimized for quick processing of medical imaging data
- **Smart Resizing**: Efficient nearest-neighbor interpolation for dimension matching

## Error Handling

The tool gracefully handles various error conditions:
- **Missing DICOM files**: Falls back to demo mode
- **Missing heatmap files**: Continues with default gradient generation
- **Unsupported formats**: Provides clear error messages with supported alternatives
- **Dimension mismatches**: Automatic resizing with warning logs
- **Corrupted data**: Safe error handling with detailed logging

## Examples Gallery

Generated test outputs with different colormaps:
- `viridis_demo.png` - JSON data with Viridis colormap
- `jet_demo.png` - CSV data with Jet colormap and percentile normalization
- `plasma_demo.png` - Default gradient with Plasma colormap

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1.  Fork the Project
2.  Create your Feature Branch (`git checkout -b feature/AmazingFeature`)
3.  Commit your Changes (`git commit -m 'Add some AmazingFeature'`)
4.  Push to the Branch (`git push origin feature/AmazingFeature`)
5.  Open a Pull Request

## License

This project is licensed under the MIT License - see the `LICENSE` file for details.

## Acknowledgements

*   [DICOM-rs](https://github.com/Enet4/dicom-rs) - Rust implementation of the DICOM standard
*   [image-rs](https://github.com/image-rs/image) - Rust image processing library
*   [ndarray](https://github.com/rust-ndarray/ndarray) - Rust N-dimensional array library
*   The Rust community for excellent crates and documentation