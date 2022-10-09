use std::ffi::OsStr;

use crate::model::{ImageComparison, ImageComparisonResult, ImageComparisonState, Rectangle};
use image::{ColorType, DynamicImage, GenericImageView, ImageFormat, Rgba, RgbaImage};

pub fn compare_images(expected_image: &OsStr, actual_image: &OsStr) -> ImageComparisonResult {
    if let Some(image_comparison_result) = validate_image_format(expected_image, actual_image) {
        return image_comparison_result;
    }
    let expected: DynamicImage = image::open(expected_image).unwrap();
    let actual: DynamicImage = image::open(actual_image).unwrap();
    if let Some(image_comparison_result) = validate_color_type(&expected, &actual) {
        return image_comparison_result;
    }
    if let Some(image_comparison_result) = validate_dimensions(&expected, &actual) {
        return image_comparison_result;
    }
    let mut image_comparison: ImageComparison = ImageComparison::new(expected, actual);
    let rectangles: Vec<Rectangle> = populate_rectangles(&mut image_comparison);
    println!("rectangles.len: {}", rectangles.len());
    if rectangles.len() > 0 {
        let result_image: RgbaImage =
            draw_rectangles(&image_comparison, rectangles, Rgba::<u8>([255, 0, 0, 255]));
        return ImageComparisonResult {
            image_comparison_state: ImageComparisonState::Mismatch,
            result_image: Some(result_image),
        };
    }
    return ImageComparisonResult {
        image_comparison_state: ImageComparisonState::Match,
        result_image: None,
    };
}

fn validate_image_format(
    expected_image: &OsStr,
    actual_image: &OsStr,
) -> Option<ImageComparisonResult> {
    let expected_image_format: ImageFormat = ImageFormat::from_path(expected_image).unwrap();
    let actual_image_format: ImageFormat = ImageFormat::from_path(actual_image).unwrap();
    if !(expected_image_format == ImageFormat::Png && actual_image_format == ImageFormat::Png) {
        eprintln!(
            "image format: {:?} & {:?} are not supported",
            expected_image_format, actual_image_format
        );
        return Some(ImageComparisonResult {
            image_comparison_state: ImageComparisonState::FormatNotSupported,
            result_image: None,
        });
    }
    None
}

fn validate_color_type(
    expected: &DynamicImage,
    actual: &DynamicImage,
) -> Option<ImageComparisonResult> {
    if !(expected.color() == ColorType::Rgba8 && actual.color() == ColorType::Rgba8) {
        eprintln!(
            "expected image colortype: {:?} and actual image colortype: {:?} is not supported",
            expected.color(),
            actual.color()
        );
        return Some(ImageComparisonResult {
            image_comparison_state: ImageComparisonState::ColorTypeNotSupported,
            result_image: None,
        });
    }
    None
}

fn validate_dimensions(
    expected: &DynamicImage,
    actual: &DynamicImage,
) -> Option<ImageComparisonResult> {
    if !dimensions_are_equal(&expected.dimensions(), &actual.dimensions()) {
        eprintln!(
            "expected image dimensions: {:?} and actual image dimensions: {:?} are not equal",
            expected.dimensions(),
            actual.dimensions()
        );
        return Some(ImageComparisonResult {
            image_comparison_state: ImageComparisonState::SizeMismatch,
            result_image: None,
        });
    }
    None
}

fn dimensions_are_equal(first: &(u32, u32), second: &(u32, u32)) -> bool {
    first.eq(second)
}

fn populate_rectangles(image_comparison: &mut ImageComparison) -> Vec<Rectangle> {
    let count_of_different_pixels: usize = populate_matrix(image_comparison);
    println!("count_of_different_pixels: {}", count_of_different_pixels);
    if count_of_different_pixels == 0usize
        || is_allowed_percent_of_different_pixels(image_comparison, &count_of_different_pixels)
    {
        return Vec::<Rectangle>::new();
    }
    group_regions(image_comparison);
    let mut rectangles: Vec<Rectangle> = Vec::new();
    let default_rectangle: Rectangle = Rectangle::create_default();
    while image_comparison.counter <= image_comparison.region_count {
        // println!(
        //     "{} {}",
        //     image_comparison.counter, image_comparison.region_count
        // );
        let rectangle: Rectangle = create_rectangle(image_comparison);
        // println!(
        //     "rectangle: {:?} -- {:?}",
        //     rectangle.min_point, rectangle.max_point
        // );
        if !rectangle.equals(&default_rectangle)
            && rectangle.size() >= image_comparison.minimal_rectangle_size as usize
        {
            rectangles.push(rectangle);
        }
        image_comparison.counter += 1;
    }
    //println!("rectangles.len before merge: {}", rectangles.len());
    merge_rectangles(merge_rectangles(rectangles))
}

fn populate_matrix(image_comparison: &mut ImageComparison) -> usize {
    let mut count_of_different_pixels: usize = 0;
    let e_imgbuf: &RgbaImage = image_comparison.expected.as_rgba8().expect("msg");
    let a_imgbuf: &RgbaImage = image_comparison.actual.as_rgba8().expect("msg");
    for (x, y, expected_pixel) in e_imgbuf.enumerate_pixels() {
        let actual_pixel: &Rgba<u8> = a_imgbuf.get_pixel(x, y);
        let e: [u8; 4] = expected_pixel.0;
        let a: [u8; 4] = actual_pixel.0;
        if e.ne(&a) {
            count_of_different_pixels += 1;
            //println!("x: {} -- y: {}", x, y);
            image_comparison.matrix[[y as usize, x as usize]] = 1;
        }
    }
    count_of_different_pixels
}

fn is_allowed_percent_of_different_pixels(
    image_comparison: &ImageComparison,
    count_of_different_pixels: &usize,
) -> bool {
    let total_pixel_count: usize =
        image_comparison.matrix.nrows() * image_comparison.matrix.ncols();
    let actual_percent_of_different_pixels: f64 =
        (*count_of_different_pixels as f64 / total_pixel_count as f64) * 100f64;
    actual_percent_of_different_pixels <= image_comparison.allowing_percent_of_different_pixels
}

fn group_regions(image_comparison: &mut ImageComparison) {
    for y in 0..image_comparison.matrix.nrows() {
        for x in 0..image_comparison.matrix.ncols() {
            if image_comparison.matrix[[y, x]] == 1 {
                //println!("x: {} -- y: {}", x, y);
                join_to_region(image_comparison, x, y);
                image_comparison.region_count += 1;
            }
        }
    }
}

fn join_to_region(image_comparison: &mut ImageComparison, x: usize, y: usize) {
    if is_jump_rejected(image_comparison, x, y) {
        //println!("JumpRejected x: {} -- y: {}", x, y);
        return;
    }
    image_comparison.matrix[[y, x]] = image_comparison.region_count as usize;
    // println!(
    //     "jtr x: {} -- y: {} = {}",
    //     x, y, image_comparison.region_count
    // );
    for i in 0..image_comparison.threshold {
        join_to_region(image_comparison, x + 1 + i as usize, y);
        join_to_region(image_comparison, x, y + 1 + i as usize);
        if no_sub_overflow(y, 1, i as usize) {
            join_to_region(image_comparison, x + 1 + i as usize, y - 1 - i as usize);
        } else {
            //println!("JumpRejected -ov1- x: {} -- y: {}", x, y);
        }
        if no_sub_overflow(x, 1, i as usize) {
            join_to_region(image_comparison, x - 1 - i as usize, y + 1 + i as usize);
        } else {
            //println!("JumpRejected -ov2- x: {} -- y: {}", x, y);
        }
        join_to_region(image_comparison, x + 1 + i as usize, y + 1 + i as usize);
    }
}

fn is_jump_rejected(image_comparison: &ImageComparison, x: usize, y: usize) -> bool {
    y >= image_comparison.matrix.nrows()
        || x >= image_comparison.matrix.ncols()
        || image_comparison.matrix[[y, x]] != 1
}

fn no_sub_overflow(value: usize, operand_1: usize, operand_2: usize) -> bool {
    if let Some(result_1) = value.checked_sub(operand_1) {
        if let Some(_result_2) = result_1.checked_sub(operand_2) {
            return true;
        }
    }
    false
}

fn create_rectangle(image_comparison: &ImageComparison) -> Rectangle {
    let mut rectange = Rectangle::create_default();
    for y in 0..image_comparison.matrix.nrows() {
        for x in 0..image_comparison.matrix.ncols() {
            if image_comparison.matrix[[y, x]] == image_comparison.counter as usize {
                //println!("cr -> x: {} -- y: {}", x, y);
                update_rectangle_creation(&mut rectange, x, y);
            }
        }
    }
    rectange
}

fn update_rectangle_creation(rectangle: &mut Rectangle, x: usize, y: usize) {
    if x < rectangle.min_point.x {
        rectangle.min_point.x = x;
    }
    if x > rectangle.max_point.x {
        rectangle.max_point.x = x;
    }
    if y < rectangle.min_point.y {
        rectangle.min_point.y = y;
    }
    if y > rectangle.max_point.y {
        rectangle.max_point.y = y;
    }
}

fn merge_rectangles(mut rectangles: Vec<Rectangle>) -> Vec<Rectangle> {
    let mut position: usize = 0;
    let zero_rectangle: Rectangle = Rectangle::create_zero();
    while position < rectangles.len() {
        if rectangles[position].equals(&zero_rectangle) {
            position += 1;
        }
        for index in 1 + position..rectangles.len() {
            //println!("index: {} -- position: {}", index, position);
            let rectangle_at_position: Rectangle = rectangles[position];
            let rectangle_at_index: Rectangle = rectangles[index];
            if rectangle_at_index.equals(&zero_rectangle) {
                continue;
            }
            if rectangle_at_position.is_overlapping(&rectangle_at_index) {
                rectangles[position] = rectangle_at_position.merge(&rectangle_at_index);
                rectangles[index] = Rectangle::create_zero();
                if position != 0 {
                    position -= 1;
                }
            }
        }
        position += 1;
    }
    rectangles
        .into_iter()
        .filter(|rectangle| !rectangle.equals(&zero_rectangle))
        .collect()
}

fn draw_rectangles(
    image_comparison: &ImageComparison,
    mut rectangles: Vec<Rectangle>,
    color: Rgba<u8>,
) -> RgbaImage {
    let mut result: RgbaImage = image_comparison.actual.to_rgba8();
    let thickness: u32 = 2;
    for rectangle in rectangles.iter_mut() {
        for i in 0..=thickness {
            if i > 0 {
                rectangle.min_point.decrement();
                rectangle.max_point.increment();
            }
            // println!(
            //     "min_point_x: {:?} -- min_point_y:{:?} -- max_point_x: {:?} -- max_point_y:{:?}",
            //     rectangle.min_point.x,
            //     rectangle.min_point.y,
            //     rectangle.max_point.x,
            //     rectangle.max_point.y
            // );
            if !rectangle.out_of_bounds(image_comparison) {
                draw_rectangle(&mut result, rectangle, color);
            }
        }
    }
    result
}

fn draw_rectangle(image: &mut RgbaImage, rectangle: &Rectangle, color: Rgba<u8>) {
    let min_point_x: u32 = rectangle.min_point.x.try_into().unwrap();
    let min_point_y: u32 = rectangle.min_point.y.try_into().unwrap();
    let max_point_x: u32 = rectangle.max_point.x.try_into().unwrap();
    let max_point_y: u32 = rectangle.max_point.y.try_into().unwrap();
    draw_line_segment(
        image,
        (min_point_x, min_point_y),
        (max_point_x, min_point_y),
        color,
    );
    draw_line_segment(
        image,
        (min_point_x, max_point_y),
        (max_point_x, max_point_y),
        color,
    );
    draw_line_segment(
        image,
        (min_point_x, min_point_y),
        (min_point_x, max_point_y),
        color,
    );
    draw_line_segment(
        image,
        (max_point_x, min_point_y),
        (max_point_x, max_point_y),
        color,
    );
}

fn draw_line_segment(image: &mut RgbaImage, start: (u32, u32), end: (u32, u32), color: Rgba<u8>) {
    // vertical line segment
    if start.0 == end.0 {
        for y in start.1..=end.1 {
            image.put_pixel(start.0, y, color);
        }
    }
    // horizontal line segment
    if start.1 == end.1 {
        for x in start.0..=end.0 {
            image.put_pixel(x, start.1, color);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::merge_rectangles;
    use crate::model::Rectangle;
    #[test]
    fn merge_rectangles_same_coordinates() {
        let rectangle1: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 3);
        let rectangle2: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 3);
        let rectangle3: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 3);
        let rectangle4: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 3);
        let rectangles: Vec<Rectangle> = vec![rectangle1, rectangle2, rectangle3, rectangle4];
        //println!("len b: {}", rectangles.len());
        let rectangles: Vec<Rectangle> = merge_rectangles(rectangles);
        //println!("len a: {}", rectangles.len());
        assert_eq!(rectangles.len(), 1);
        let rectangle_expected: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 3);
        let rectangle_actual: Rectangle = rectangles[0];
        assert!(rectangle_expected.equals(&rectangle_actual));
    }
    #[test]
    fn merge_rectangles_same_x() {
        let rectangle1: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 3);
        let rectangle2: Rectangle = Rectangle::create_with_coordinates(1, 2, 3, 4);
        let rectangle3: Rectangle = Rectangle::create_with_coordinates(1, 3, 3, 5);
        let rectangles: Vec<Rectangle> = vec![rectangle1, rectangle2, rectangle3];
        //println!("len b: {}", rectangles.len());
        let rectangles: Vec<Rectangle> = merge_rectangles(rectangles);
        //println!("len a: {}", rectangles.len());
        assert_eq!(rectangles.len(), 1);
        let rectangle_expected: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 5);
        let rectangle_actual: Rectangle = rectangles[0];
        assert!(rectangle_expected.equals(&rectangle_actual));
    }
    #[test]
    fn merge_rectangles_same_y() {
        let rectangle1: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 2);
        let rectangle2: Rectangle = Rectangle::create_with_coordinates(2, 1, 4, 2);
        let rectangle3: Rectangle = Rectangle::create_with_coordinates(3, 1, 5, 2);
        let rectangles: Vec<Rectangle> = vec![rectangle1, rectangle2, rectangle3];
        //println!("len b: {}", rectangles.len());
        let rectangles: Vec<Rectangle> = merge_rectangles(rectangles);
        //println!("len a: {}", rectangles.len());
        assert_eq!(rectangles.len(), 1);
        let rectangle_expected: Rectangle = Rectangle::create_with_coordinates(1, 1, 5, 2);
        let rectangle_actual: Rectangle = rectangles[0];
        assert!(rectangle_expected.equals(&rectangle_actual));
    }
    #[test]
    fn merge_rectangles_diagonal_overlap() {
        let rectangle1: Rectangle = Rectangle::create_with_coordinates(1, 1, 3, 3);
        let rectangle2: Rectangle = Rectangle::create_with_coordinates(3, 3, 5, 5);
        let rectangle3: Rectangle = Rectangle::create_with_coordinates(5, 5, 7, 7);
        let rectangles: Vec<Rectangle> = vec![rectangle1, rectangle2, rectangle3];
        //println!("len b: {}", rectangles.len());
        let rectangles: Vec<Rectangle> = merge_rectangles(rectangles);
        //println!("len a: {}", rectangles.len());
        assert_eq!(rectangles.len(), 1);
        let rectangle_expected: Rectangle = Rectangle::create_with_coordinates(1, 1, 7, 7);
        let rectangle_actual: Rectangle = rectangles[0];
        assert!(rectangle_expected.equals(&rectangle_actual));
    }
    #[test]
    fn merge_rectangles_diagonal_no_overlap() {
        let rectangle1: Rectangle = Rectangle::create_with_coordinates(1, 1, 2, 2);
        let rectangle2: Rectangle = Rectangle::create_with_coordinates(3, 3, 4, 4);
        let rectangle3: Rectangle = Rectangle::create_with_coordinates(5, 5, 6, 6);
        let rectangles: Vec<Rectangle> = vec![rectangle1, rectangle2, rectangle3];
        //println!("len b: {}", rectangles.len());
        let rectangles: Vec<Rectangle> = merge_rectangles(rectangles);
        //println!("len a: {}", rectangles.len());
        assert_eq!(rectangles.len(), 3);
        assert!(rectangles[0].equals(&rectangle1));
        assert!(rectangles[1].equals(&rectangle2));
        assert!(rectangles[2].equals(&rectangle3));
    }
}
