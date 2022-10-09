#![allow(dead_code)]
use image::{DynamicImage, GenericImageView, RgbaImage};
use ndarray::Array2;
use std::cmp::{max, min};

pub(crate) struct ImageComparison {
    pub(crate) threshold: u32,
    pub(crate) allowing_percent_of_different_pixels: f64,
    pub(crate) counter: u32,
    pub(crate) region_count: u32,
    pub(crate) minimal_rectangle_size: u32,
    pub(crate) expected: DynamicImage,
    pub(crate) actual: DynamicImage,
    pub(crate) image_width: u32,
    pub(crate) image_height: u32,
    pub(crate) matrix: Array2<usize>,
}

impl ImageComparison {
    pub(crate) fn new(expected: DynamicImage, actual: DynamicImage) -> Self {
        Self {
            threshold: 5,
            counter: 2,
            region_count: 2,
            minimal_rectangle_size: 1,
            image_width: expected.dimensions().0,
            image_height: expected.dimensions().1,
            matrix: Array2::<usize>::zeros((
                expected.dimensions().1 as usize,
                expected.dimensions().0 as usize,
            )),
            allowing_percent_of_different_pixels: 0f64,
            expected,
            actual,
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum ImageComparisonState {
    FormatNotSupported,
    ColorTypeNotSupported,
    SizeMismatch,
    Mismatch,
    Match,
}
#[derive(Debug)]
pub struct ImageComparisonResult {
    pub image_comparison_state: ImageComparisonState,
    pub result_image: Option<RgbaImage>,
}
#[derive(Debug, Copy, Clone)]
pub(crate) struct Rectangle {
    pub(crate) min_point: Point,
    pub(crate) max_point: Point,
}

impl Rectangle {
    pub(crate) fn create_default() -> Self {
        Self {
            min_point: Point::new(usize::MAX, usize::MAX),
            max_point: Point::new(usize::MIN, usize::MIN),
        }
    }
    pub(crate) fn create_zero() -> Self {
        Self {
            min_point: Point::new(0, 0),
            max_point: Point::new(0, 0),
        }
    }
    pub(crate) fn create_with_points(min_point: Point, max_point: Point) -> Self {
        Self {
            min_point,
            max_point,
        }
    }
    pub(crate) fn create_with_coordinates(
        min_x: usize,
        min_y: usize,
        max_x: usize,
        max_y: usize,
    ) -> Self {
        Self {
            min_point: Point::new(min_x, min_y),
            max_point: Point::new(max_x, max_y),
        }
    }
    pub(crate) fn merge(&self, that: &Rectangle) -> Self {
        Self::create_with_coordinates(
            min(self.min_point.x, that.min_point.x),
            min(self.min_point.y, that.min_point.y),
            max(self.max_point.x, that.max_point.x),
            max(self.max_point.y, that.max_point.y),
        )
    }
    pub(crate) fn is_overlapping(&self, that: &Rectangle) -> bool {
        if self.max_point.y < that.min_point.y || that.max_point.y < self.min_point.y {
            return false;
        }
        self.max_point.x >= that.min_point.x && that.max_point.x >= self.min_point.x
    }
    pub(crate) fn get_width(&self) -> usize {
        self.max_point.x - self.min_point.x + 1
    }
    pub(crate) fn get_height(&self) -> usize {
        self.max_point.y - self.min_point.y + 1
    }
    pub(crate) fn size(&self) -> usize {
        self.get_width() * self.get_height()
    }
    pub(crate) fn contains_point(&self, point: &Point) -> bool {
        point.x >= self.min_point.x
            && point.x <= self.max_point.x
            && point.y >= self.min_point.y
            && point.y <= self.max_point.y
    }
    pub(crate) fn equals(&self, that: &Rectangle) -> bool {
        self.min_point.equals(&that.min_point) && self.max_point.equals(&that.max_point)
    }
    pub(crate) fn out_of_bounds(&self, image_comparison: &ImageComparison) -> bool {
        if self.min_point.x >= image_comparison.image_width as usize
            || self.max_point.x >= image_comparison.image_width as usize
            || self.min_point.y >= image_comparison.image_height as usize
            || self.max_point.y >= image_comparison.image_height as usize
        {
            return true;
        }
        false
    }
}
#[derive(Debug, Copy, Clone)]
pub(crate) struct Point {
    pub(crate) x: usize,
    pub(crate) y: usize,
}

impl Point {
    pub(crate) fn new(x: usize, y: usize) -> Self {
        Self { x, y }
    }
    pub(crate) fn equals(&self, that: &Point) -> bool {
        return self.x == that.x && self.y == that.y;
    }
    pub(crate) fn increment(&mut self) {
        if let (Some(result_x), Some(result_y)) = (self.x.checked_add(1), self.y.checked_add(1)) {
            self.x = result_x;
            self.y = result_y;
        }
    }
    pub(crate) fn decrement(&mut self) {
        if let (Some(result_x), Some(result_y)) = (self.x.checked_sub(1), self.y.checked_sub(1)) {
            self.x = result_x;
            self.y = result_y;
        }
    }
}
