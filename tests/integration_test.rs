use crate::common::get_tests_resources_directory;
use image::{open, RgbaImage};
use image_compare::{compare_images, ImageComparisonResult, ImageComparisonState};
use std::path::PathBuf;

mod common;

#[test]
fn validate_image_format_fail() {
    match get_tests_resources_directory() {
        Ok(tests_resources_directory) => {
            let mut expected: PathBuf = tests_resources_directory.clone();
            expected.push("expected.jpg");
            let mut actual: PathBuf = tests_resources_directory.clone();
            actual.push("actual.jpg");
            let result: ImageComparisonResult =
                compare_images(expected.as_os_str(), actual.as_os_str());
            assert_eq!(
                result.image_comparison_state,
                ImageComparisonState::FormatNotSupported
            );
        }
        Err(error) => eprintln!("{}", error),
    }
}

#[test]
fn validate_color_type_fail() {
    match get_tests_resources_directory() {
        Ok(tests_resources_directory) => {
            let mut expected: PathBuf = tests_resources_directory.clone();
            expected.push("expected-rgb.png");
            let mut actual: PathBuf = tests_resources_directory.clone();
            actual.push("actual-rgb.png");
            let result: ImageComparisonResult =
                compare_images(expected.as_os_str(), actual.as_os_str());
            assert_eq!(
                result.image_comparison_state,
                ImageComparisonState::ColorTypeNotSupported
            );
        }
        Err(error) => eprintln!("{}", error),
    }
}

#[test]
fn validate_dimensions_fail() {
    match get_tests_resources_directory() {
        Ok(tests_resources_directory) => {
            let mut expected: PathBuf = tests_resources_directory.clone();
            expected.push("expected-1.png");
            let mut actual: PathBuf = tests_resources_directory.clone();
            actual.push("actual-2.png");
            let result: ImageComparisonResult =
                compare_images(expected.as_os_str(), actual.as_os_str());
            assert_eq!(
                result.image_comparison_state,
                ImageComparisonState::SizeMismatch
            );
        }
        Err(error) => eprintln!("{}", error),
    }
}

#[test]
fn compare_images_mismatch_1() {
    test_compare_images_mismatch("expected-1.png", "actual-1.png", "result-1.png");
}

#[test]
fn compare_images_mismatch_2() {
    test_compare_images_mismatch("expected-2.png", "actual-2.png", "result-2.png");
}

#[test]
fn compare_images_mismatch_3() {
    test_compare_images_mismatch("expected-3.png", "actual-3.png", "result-3.png");
}

#[test]
fn compare_images_mismatch_4() {
    test_compare_images_mismatch("expected-4.png", "actual-4.png", "result-4.png");
}

#[test]
fn compare_images_mismatch_5() {
    test_compare_images_mismatch("expected-5.png", "actual-5.png", "result-5.png");
}

#[test]
fn compare_images_mismatch_6() {
    test_compare_images_mismatch("expected-6.png", "actual-6.png", "result-6.png");
}

#[test]
fn compare_images_match() {
    test_compare_images_match("expected_same.png", "actual_same.png");
}

fn test_compare_images_match(expected_image: &str, actual_image: &str) {
    match get_tests_resources_directory() {
        Ok(tests_resources_directory) => {
            let mut expected: PathBuf = tests_resources_directory.clone();
            expected.push(expected_image);
            let mut actual: PathBuf = tests_resources_directory.clone();
            actual.push(actual_image);
            let image_comparison_result: ImageComparisonResult =
                compare_images(expected.as_os_str(), actual.as_os_str());
            assert_eq!(
                image_comparison_result.image_comparison_state,
                ImageComparisonState::Match
            );
            assert_eq!(None, image_comparison_result.result_image);
        }
        Err(error) => eprintln!("{}", error),
    }
}

fn test_compare_images_mismatch(expected_image: &str, actual_image: &str, result_image: &str) {
    match get_tests_resources_directory() {
        Ok(tests_resources_directory) => {
            let mut expected: PathBuf = tests_resources_directory.clone();
            expected.push(expected_image);
            let mut actual: PathBuf = tests_resources_directory.clone();
            actual.push(actual_image);
            let image_comparison_result: ImageComparisonResult =
                compare_images(expected.as_os_str(), actual.as_os_str());
            assert_eq!(
                image_comparison_result.image_comparison_state,
                ImageComparisonState::Mismatch
            );
            assert_ne!(None, image_comparison_result.result_image);
            let mut result: PathBuf = tests_resources_directory.clone();
            result.push(result_image);
            let expected_result: RgbaImage = open(result).unwrap().into_rgba8();
            let actual_result: RgbaImage = image_comparison_result
                .result_image
                .expect("result_image is missing from image_comparison_result");
            assert_eq!(true, expected_result.eq(&actual_result));
        }
        Err(error) => eprintln!("{}", error),
    }
}
