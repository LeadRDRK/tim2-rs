use std::{fs::File, path::{Path, PathBuf}};

use minifb::{Key, KeyRepeat, Scale, ScaleMode, Window, WindowOptions};
use tim2::*;

const SCALES: [Scale; 4] = [
    Scale::X1,
    Scale::X2,
    Scale::X4,
    Scale::X8
];

struct Image {
    xrgb: Vec<u32>,
    width: u16,
    height: u16,
    image_type: ImageType
}

impl Image {
    fn window_title(&self, path: &Path, picture_index: usize, picture_count: usize) -> String {
        format!("{} ({} of {}) [{}x{}, {:?}] - tim2view",
            path.file_name().unwrap().display(),
            picture_index + 1, 
            picture_count,
            self.width,
            self.height,
            self.image_type
        )
    }
}

fn alpha_blend(dest: u8, src: u8, alpha: u8) -> u32 {
    ((((src as i32 - dest as i32) * alpha as i32) >> 7) + dest as i32).clamp(0, 255) as u32
}

fn run(path: &Path, no_transparency_grid: bool) -> anyhow::Result<()> {
    let file = File::open(path)?;
    let mut decoder = Decoder::from_reader(file)?;
    let mut pictures: Vec<Image> = Vec::with_capacity(decoder.picture_count() as _);
    while let Some(picture_decoder) = decoder.next_picture()? {
        let picture = picture_decoder.decode()?;
        let info = picture.info();
        let width = info.image_width;
        let height = info.image_height;
        let image_type = info.image_type;

        let image_reader = picture.image_reader(None)?;
        let rgba_u8 = image_reader.read_to_rgba()?;

        let mut xrgb: Vec<u32> = Vec::with_capacity(image_reader.pixel_count());
        for y in 0..height {
            for x in 0..width {
                let index = y as usize * width as usize + x as usize;
                let offset = index * 4;
                let [r, g, b, a] = rgba_u8[offset..offset + 4].try_into().unwrap();

                let bg_color: u8 = if no_transparency_grid {
                    0
                }
                else {
                    if (x / 8 + y / 8) % 2 == 0 {
                        0x66
                    }
                    else {
                        0x99
                    }
                };

                let r = alpha_blend(bg_color, r, a);
                let g = alpha_blend(bg_color, g, a);
                let b = alpha_blend(bg_color, b, a);
                xrgb.push((r << 16) | (g << 8) | b);
            }
        }

        pictures.push(Image { xrgb, width, height, image_type });
    }

    let mut picture_index = 0;
    let mut scale_index = 0;
    loop {
        let mut window = {
            let Some(picture) = pictures.get(picture_index) else {
                println!("TIM2 file has no pictures, quitting.");
                return Ok(());
            };

            let mut window = Window::new(
                &picture.window_title(path, picture_index, pictures.len()),
                picture.width as _,
                picture.height as _,
                WindowOptions {
                    scale: SCALES[scale_index],
                    scale_mode: ScaleMode::AspectRatioStretch,
                    resize: true,
                    ..Default::default()
                }
            )?;
            window.set_target_fps(60);
            window.update_with_buffer(&picture.xrgb, picture.width as _, picture.height as _)?;

            window
        };

        while window.is_open() {
            let mut next_picture_index = None;
            let mut next_scale_index = None;

            if window.is_key_pressed(Key::Right, KeyRepeat::No) {
                next_picture_index = Some((picture_index + 1) % pictures.len());
            }

            if window.is_key_pressed(Key::Left, KeyRepeat::No) {
                if next_picture_index.is_none() {
                    next_picture_index = Some((if picture_index == 0 { pictures.len() } else { picture_index }) - 1);
                }
                else {
                    // cancelled out
                    next_picture_index = None;
                }
            }

            if window.is_key_pressed(Key::Up, KeyRepeat::No) {
                next_scale_index = (scale_index < SCALES.len() - 1).then(|| scale_index + 1);
            }

            if window.is_key_pressed(Key::Down, KeyRepeat::No) {
                if next_scale_index.is_none() {
                    next_scale_index = Some(scale_index.saturating_sub(1));
                }
                else {
                    // cancelled out
                    next_scale_index = None;
                }
            }

            if let Some(index) = next_scale_index && scale_index != index {
                scale_index = index;
                break;
            }

            if let Some(index) = next_picture_index && picture_index != index {
                let picture = &pictures[index];
                window.set_title(&picture.window_title(path, index, pictures.len()));
                window.update_with_buffer(&picture.xrgb, picture.width as _, picture.height as _)?;
                picture_index = index;
            }
            else {
                window.update();
            }
        }

        // allow recreating the window when the scale is changed
        if !window.is_open() {
            break;
        }
    }

    Ok(())
}

fn print_help() {
    println!("Usage: tim2view [OPTIONS] <TIM2_FILE>

Options:
    -h, --help                  Print help
    -b, --no-transparency-grid  Disable the checkerboard background for transparent images");
}

fn main() {
    let mut path_opt: Option<PathBuf> = None;
    let mut no_transparency_grid = false;
    for arg in std::env::args().skip(1) {
        match arg.as_str() {
            "-h" | "--help" => {
                print_help();
                return;
            },
            "-b" | "--no-transparency-grid" => no_transparency_grid = true,
            _ => {
                if path_opt.is_none() {
                    path_opt = Some(PathBuf::from(arg))
                }
                else {
                    println!("Invalid argument: {arg}\n");
                    print_help();
                    return;
                }
            }
        }
    }

    let Some(path) = path_opt else {
        print_help();
        return;
    };

    if let Err(e) = run(&path, no_transparency_grid) {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
