use super::{ConvertError, Format};
use std::path::Path;

pub fn convert(input: &Path, target: Format, output: &Path) -> Result<(), ConvertError> {
    // image::open() auto-detects format from file header — like PIL in Python
    let img = image::open(input).map_err(|e| {
        ConvertError::ImageError(format!("Failed to open image: {}", e))
    })?;

    // Map our Format to image crate's ImageFormat enum
    let img_format = match target {
        Format::Jpg => image::ImageFormat::Jpeg,
        Format::Png => image::ImageFormat::Png,
        Format::WebP => image::ImageFormat::WebP,
        Format::Bmp => image::ImageFormat::Bmp,
        Format::Tiff => image::ImageFormat::Tiff,
        _ => {
            return Err(ConvertError::ImageError(format!(
                "Image crate cannot write to {} format",
                target.extension()
            )));
        }
    };

    img.save_with_format(output, img_format).map_err(|e| {
        ConvertError::ImageError(format!("Failed to save image: {}", e))
    })?;

    Ok(())
}