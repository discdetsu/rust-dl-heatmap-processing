# rust-dl-heatmap-processing

A Rust-based DICOM heatmap processing tool that efficiently processes real medical imaging data and overlays heatmaps generated from deep learning models, leveraging the performance and safety of Rust.

## Features

*   **Real DICOM Processing**: Extract and process actual pixel data from DICOM medical images
*   **Multiple Bit Depths**: Support for 8-bit and 16-bit DICOM pixel data with automatic windowing
*   **Image Format Support**: Handle both grayscale and RGB DICOM images
*   **Heatmap Overlay**: Generate semi-transparent heatmap overlays on medical images
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

# Process a specific DICOM file
cargo run -- --input path/to/your/file.dcm --output result.png

# Generate demo heatmap with simulated data
cargo run -- --demo --output demo_result.png

# Show help
cargo run -- --help
```

### Command-Line Options

- `-i, --input <FILE>`: Input DICOM file path (default: `sample.dcm`)
- `-o, --output <FILE>`: Output PNG file path (default: `output.png`)
- `-d, --demo`: Use demo mode with simulated data
- `-h, --help`: Print help information
- `-V, --version`: Print version information

### Examples

```bash
# Process a real DICOM file
cargo run -- -i medical_scan.dcm -o heatmap_result.png

# Create a demo with custom output name
cargo run -- --demo -o my_demo.png

# Using the built executable
./target/release/rust-dl-heatmap-processing -i scan.dcm -o output.png
```

## How It Works

1. **DICOM Reading**: Opens and parses DICOM files using the `dicom-rs` ecosystem
2. **Pixel Data Extraction**: Extracts real pixel data using `dicom-pixeldata` with support for:
   - 8-bit and 16-bit pixel data
   - Grayscale and RGB images
   - Automatic bit-depth conversion and windowing
3. **Base Image Creation**: Converts DICOM pixel data to RGBA format
4. **Heatmap Generation**: Creates a semi-transparent red gradient overlay
5. **Image Composition**: Overlays the heatmap onto the medical image
6. **PNG Export**: Saves the final result as a PNG file

## Technical Details

### Supported DICOM Formats
- **Pixel Data**: 8-bit and 16-bit
- **Color Space**: Grayscale and RGB
- **Transfer Syntaxes**: Various DICOM transfer syntaxes supported by dicom-rs

### Dependencies
- `dicom` v0.8.1 - Core DICOM processing
- `dicom-pixeldata` v0.8.1 - Pixel data decoding with image support
- `image` v0.25.1 - Image processing and PNG output
- `clap` v4.5.41 - Command-line argument parsing
- `log` & `env_logger` - Logging support

### Performance
- **Memory Efficient**: Processes images in-memory with minimal allocations
- **Safe Processing**: Rust's memory safety prevents common image processing errors
- **Fast Execution**: Optimized for quick processing of medical imaging data

## Error Handling

The tool gracefully handles various error conditions:
- **Missing DICOM files**: Falls back to demo mode
- **Unsupported formats**: Provides clear error messages
- **Corrupted data**: Safe error handling with detailed logging

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
*   The Rust community for excellent crates and documentation