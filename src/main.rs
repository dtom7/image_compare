use image::ImageFormat;
use image_compare::{compare_images, ImageComparisonResult, ImageComparisonState};
use std::{
    fs::remove_file,
    path::{Path, PathBuf},
};
fn main() -> std::io::Result<()> {
    let mut expected: PathBuf = PathBuf::new();
    expected.push("C:\\Temp\\expected.png");
    let mut actual: PathBuf = PathBuf::new();
    actual.push("C:\\Temp\\actual.png");
    let result: ImageComparisonResult = compare_images(expected.as_os_str(), actual.as_os_str());
    match result.image_comparison_state {
        ImageComparisonState::FormatNotSupported => (),
        ImageComparisonState::ColorTypeNotSupported => (),
        ImageComparisonState::SizeMismatch => (),
        ImageComparisonState::Mismatch => {
            println!("expected and actual images are not matching");
            let mut result_image_path: PathBuf = PathBuf::new();
            result_image_path.push("C:\\Temp\\result.png");
            if Path::new(result_image_path.as_os_str()).try_exists()? {
                remove_file(&result_image_path)?;
            }
            if let Some(result_image) = result.result_image {
                result_image
                    .save_with_format(result_image_path, ImageFormat::Png)
                    .unwrap();
            }
        }
        ImageComparisonState::Match => println!("expected and actual images are matching"),
    }
    Ok(())
}
