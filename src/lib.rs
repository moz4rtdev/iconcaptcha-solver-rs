//! IconCaptcha Solver is a solution for solving IconCaptcha images.
//! It provides methods to load images from various sources and extract icons from them.

use base64::prelude::*;
use image::{DynamicImage, GenericImageView, ImageBuffer, ImageReader, Rgba};
use std::{fmt::Display, io::Cursor};

#[cfg(feature = "js")]
use neon::prelude::*;

#[derive(Debug, Clone)]
pub struct Icon {
    pub position: u32,
    pub start: u32,
    pub end: u32,
    pub center_x: u32,
    pub center_y: u32,
}

impl Display for Icon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Icon {{ position: {}, start: {}, end: {}, center_x: {}, center_y: {} }}",
            self.position, self.start, self.end, self.center_x, self.center_y
        )
    }
}

pub struct IconCaptcha {
    img: DynamicImage,
}

impl IconCaptcha {
    /// Load an image from a file path.
    /// Example:
    /// ```
    /// use iconcaptcha_solver::IconCaptcha;
    /// let captcha = IconCaptcha::load_image("path/to/image.png");
    /// ```
    pub fn load_image(path: &str) -> Self {
        let img = ImageReader::open(path).unwrap().decode().unwrap();
        Self { img }
    }

    /// Load an image from a base64 string.
    /// Example:
    /// ```
    /// use iconcaptcha_solver::IconCaptcha;
    /// let captcha = IconCaptcha::load_from_base64("base64_string");
    /// ```
    pub fn load_from_base64(base64: &str) -> Result<Self, String> {
        let base64_dec = BASE64_STANDARD.decode(base64);
        if let Err(_) = base64_dec {
            return Err("Invalid base64".to_string());
        }
        let img = ImageReader::new(Cursor::new(&base64_dec.unwrap()[..]))
            .with_guessed_format()
            .unwrap()
            .decode();
        if let Err(_) = img {
            return Err("Invalid image".to_string());
        }

        Ok(Self { img: img.unwrap() })
    }

    /// Load an image from a byte array.
    /// Example:
    /// ```
    /// use iconcaptcha_solver::IconCaptcha;
    /// let captcha = IconCaptcha::load_from_bytes(vec![0, 1, 2, 3]);
    /// ```
    pub fn load_from_bytes(bytes: Vec<u8>) -> Self {
        let img = ImageReader::new(Cursor::new(&bytes[..]))
            .with_guessed_format()
            .unwrap()
            .decode()
            .unwrap();

        Self { img }
    }

    /// Save the captcha image to a file.
    /// Example:
    /// ```
    /// use iconcaptcha_solver::IconCaptcha;
    /// let captcha = IconCaptcha::load_from_bytes(vec![0, 1, 2, 3]);
    /// captcha.save("captcha.png");
    /// ```
    pub fn save(&self, path: &str) {
        self.img.save(path).unwrap()
    }

    fn get_positions(&self) -> Vec<Icon> {
        let img = self.img.clone();
        let height = img.height();
        let width = img.width();

        // array initiate with 0
        // 0 is position initial position
        let mut delimiter = vec![0];

        for i in 0..width {
            let pixel = img.get_pixel(i, 0);
            if pixel[0] == 64 && pixel[1] == 64 && pixel[2] == 64 {
                delimiter.push(i);
            }
            if pixel[0] == 240 && pixel[1] == 240 && pixel[2] == 240 {
                delimiter.push(i);
            }
        }

        // width end position
        let _ = delimiter.push(width);

        let mut imgs_positions = vec![];

        for i in 0..delimiter.len() - 1 {
            // reverse to avoid negative result
            // start == initial position or color gray
            // end == end position or color gray
            // delimiter[end], delimiter[start]
            let (p_end, p_start) = (delimiter[i + 1], delimiter[i]);

            //calculate center
            // (p_end - 1) - (p_start + 1) == icon width
            // (p_end - 1) - (p_start + 1)) / 2) == icon center
            // (((p_end - 1) - (p_start + 1)) / 2) + delimiter[i] + 1 == icon center position
            let center = (((p_end - 1) - (p_start + 1)) / 2) + delimiter[i] + 1;
            //end - start - center
            imgs_positions.push(vec![p_end - 1, p_start + 1, center]);
        }

        let mut icons_positions: Vec<Icon> = Vec::new();
        for (index, icon) in imgs_positions.iter().enumerate() {
            let icon = Icon {
                position: index as u32 + 1,
                start: icon[1],
                end: icon[0],
                center_x: icon[2],
                center_y: height / 2,
            };
            icons_positions.push(icon);
        }
        icons_positions
    }

    fn cropped(&self, icons_positions: &Vec<Icon>) -> Vec<DynamicImage> {
        let mut icons = vec![];
        for positions in icons_positions {
            let img_rgb = self
                .img
                .crop_imm(positions.start, 0, positions.end - positions.start, 50)
                .to_rgba8();

            let (width, height) = img_rgb.dimensions();
            let mut min_x = width;
            let mut min_y = height;
            let mut max_x = 0;
            let mut max_y = 0;

            // It goes through all the pixels and identifies the corners
            // of the icon's bounding box.
            //  (min_x, min_y)
            //   \
            //    \
            //     \
            //      \
            //       (max_x, max_y)

            for (x, y, pixel) in img_rgb.enumerate_pixels() {
                if pixel.0[3] != 0 {
                    if x < min_x {
                        min_x = x;
                    }
                    if y < min_y {
                        min_y = y;
                    }
                    if x > max_x {
                        max_x = x;
                    }
                    if y > max_y {
                        max_y = y;
                    }
                }
            }

            // Calculate the dimensions of the new image
            // resulting in the area + 1 pixel to fit
            // completely the icon
            let new_width = max_x - min_x + 1;
            let new_height = max_y - min_y + 1;

            // Create a new image with the area of the icon + 1px
            let mut new_img: ImageBuffer<Rgba<u8>, Vec<u8>> =
                ImageBuffer::new(new_width, new_height);

            // Copy pixels not nulls for new image
            for (x, y, pixel) in img_rgb.enumerate_pixels() {
                if pixel.0[3] != 0 {
                    // centralize the icon
                    let new_x = x - min_x;
                    let new_y = y - min_y;
                    new_img.put_pixel(new_x, new_y, *pixel);
                }
            }
            let new_img = DynamicImage::ImageRgba8(new_img);
            icons.push(new_img);
        }
        icons
    }

    fn reflect_image(&self, imgs: Vec<DynamicImage>) -> Vec<DynamicImage> {
        let mut reflected_image = vec![];
        for img in imgs {
            let img = img.to_rgba8();
            let (width, height) = img.dimensions();
            let mut new_img = ImageBuffer::new(width, height);
            for y in 0..height {
                for x in 0..width {
                    let pixel = img.get_pixel(x, y);
                    new_img.put_pixel(width - 1 - x, y, *pixel);
                }
            }

            let _ = reflected_image.push(DynamicImage::ImageRgba8(new_img));
        }
        reflected_image
    }

    fn rotate(&self, image: &DynamicImage) -> Vec<DynamicImage> {
        let mut img_rotate = vec![
            image.clone(),
            image.rotate90(),
            image.rotate180(),
            image.rotate270(),
        ];
        let img_reflected = self.reflect_image(img_rotate.clone());
        img_rotate.extend_from_slice(&img_reflected[..]);
        img_rotate
    }

    /// Solve the captcha image.
    /// This method returns the struct Icon with the some informations about the captcha image.
    ///
    /// Example:
    /// ```
    /// use iconcaptcha_solver::IconCaptcha;
    /// let captcha = IconCaptcha::load_from_bytes(vec![0, 1, 2, 3]);
    /// let icon = captcha.solve();
    /// ```
    pub fn solve(self) -> Icon {
        let icons_positions = self.get_positions();
        let icons_cropped = self.cropped(&icons_positions);
        let mut icons_repeat: Vec<i32> = vec![0; icons_positions.len()];
        for (i, img) in icons_cropped.iter().enumerate() {
            for (j, img2) in icons_cropped.iter().enumerate() {
                if i == j {
                    continue;
                }
                let imgs_rotate = self.rotate(&img2);
                let mut diff = 0;
                'rotation: for ic in imgs_rotate {
                    for (p1, p2) in img.pixels().zip(ic.pixels()) {
                        if p1.2[3] != p2.2[3] {
                            diff += 1;
                        }
                    }
                    if diff == 0 {
                        icons_repeat[i] = icons_repeat[i] + 1;
                        break 'rotation;
                    }
                    diff = 0;
                }
            }
        }

        let mut index_position_final = 0;
        let mut index_position = icons_repeat.len() as i32;
        for (i, n) in icons_repeat.iter().enumerate() {
            if n < &index_position {
                index_position = *n;
                index_position_final = i;
            }
        }
        icons_positions[index_position_final].clone()
    }
}

#[cfg(feature = "js")]
fn solve(mut cx: FunctionContext) -> JsResult<JsObject> {
    let bs64_img = cx.argument::<JsString>(0)?.value(&mut cx);
    let cap = IconCaptcha::load_from_base64(&bs64_img);
    if let Err(_) = cap {
        let obj = cx.empty_object();
        let msg = cx.string("invalid image");
        let status = cx.boolean(false);
        obj.set(&mut cx, "message", msg)?;
        obj.set(&mut cx, "success", status)?;
        return Ok(obj);
    }
    let icon = cap.unwrap().solve();
    let obj = cx.empty_object();
    let position = cx.number(icon.position);
    obj.set(&mut cx, "position", position)?;
    let start = cx.number(icon.start);
    obj.set(&mut cx, "start", start)?;
    let end = cx.number(icon.end);
    obj.set(&mut cx, "end", end)?;
    let center_x = cx.number(icon.center_x);
    obj.set(&mut cx, "center_x", center_x)?;
    let center_y = cx.number(icon.center_y);
    obj.set(&mut cx, "center_y", center_y)?;
    let status = cx.boolean(true);
    obj.set(&mut cx, "success", status)?;
    Ok(obj)
}

#[cfg(feature = "js")]
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
    cx.export_function("solve", solve)?;
    Ok(())
}

#[cfg(test)]
mod test {
    use walkdir::WalkDir;

    use super::*;

    #[test]
    fn solving() {
        let paths = WalkDir::new("captchas").sort_by_file_name().into_iter();
        let mut imgs = Vec::new();
        for path in paths {
            let pat = path.unwrap().path().to_str().unwrap().to_string();
            if pat.contains("png") {
                imgs.push(pat);
            }
        }
        let result = vec![
            3, 4, 2, 2, 3, 3, 3, 2, 2, 1, 3, 3, 1, 5, 1, 5, 1, 4, 2, 6, 1, 8, 1,
        ];

        let mut result_cap = vec![];
        for img in imgs {
            let img = IconCaptcha::load_image(&img);
            let icon = img.solve();
            //break;
            result_cap.push(icon.position);
        }
        assert_eq!(result, result_cap);
    }
}
