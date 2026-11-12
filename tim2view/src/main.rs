#![windows_subsystem = "windows"]

use std::{fs::File, io::{BufReader, IsTerminal}, path::{Path, PathBuf}};

use minifb::{Key, KeyRepeat, Menu, ScaleMode, Window, WindowOptions};
use rfd::{FileDialog, MessageButtons, MessageDialog, MessageLevel};
use tim2::*;

const IDM_FILE_EXIT: usize = 1;
const IDM_GO_NEXT: usize = 2;
const IDM_GO_PREVIOUS: usize = 3;
const IDM_GO_FIRST: usize = 4;
const IDM_GO_LAST: usize = 5;
const IDM_HELP_ABOUT: usize = 6;

struct Image {
    xrgb: Vec<u32>,
    width: u16,
    height: u16,
    image_type: ImageType
}

impl Image {
    fn window_title(&self, path: &Path, picture_index: usize, picture_count: usize) -> String {
        format!("{} ({} of {}) [{}x{}, {:?}] - {}",
            path.file_name().unwrap().display(),
            picture_index + 1, 
            picture_count,
            self.width,
            self.height,
            self.image_type,
            env!("CARGO_PKG_NAME")
        )
    }
}

fn alpha_blend(dest: u8, src: u8, alpha: u8) -> u32 {
    ((((src as i32 - dest as i32) * alpha as i32) >> 7) + dest as i32).clamp(0, 255) as u32
}

fn run(path: &Path, no_transparency_grid: bool) -> anyhow::Result<()> {
    let pictures: Vec<Image> = {
        let file = File::open(path)?;
        let mut decoder = Decoder::from_reader(BufReader::new(file))?;
        let mut pictures = Vec::with_capacity(decoder.picture_count() as _);

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

        pictures
    };

    let mut picture_index = 0;
    let mut window = {
        let Some(picture) = pictures.get(picture_index) else {
            create_dialog("TIM2 file has no pictures.", MessageLevel::Info).show();
            return Ok(());
        };

        let mut window = Window::new(
            &picture.window_title(path, picture_index, pictures.len()),
            picture.width as _,
            picture.height as _,
            WindowOptions {
                scale_mode: ScaleMode::AspectRatioStretch,
                resize: true,
                ..Default::default()
            }
        )?;
        window.set_target_fps(60);

        let mut menu = Menu::new("File")?;
        menu.add_item("Exit", IDM_FILE_EXIT)
            .shortcut(Key::Q, minifb::MENU_KEY_CTRL)
            .build();
        window.add_menu(&menu);

        let mut menu = Menu::new("Go")?;
        menu.add_item("Next", IDM_GO_NEXT)
            .shortcut(Key::Right, 0)
            .build();
        menu.add_item("Previous", IDM_GO_PREVIOUS)
            .shortcut(Key::Left, 0)
            .build();
        menu.add_item("First", IDM_GO_FIRST)
            .build();
        menu.add_item("Last", IDM_GO_LAST)
            .build();
        window.add_menu(&menu);

        let mut menu = Menu::new("Help")?;
        menu.add_item("About", IDM_HELP_ABOUT).build();
        window.add_menu(&menu);

        window.update_with_buffer(&picture.xrgb, picture.width as _, picture.height as _)?;

        window
    };

    let mut prev_window_size = window.get_size();
    while window.is_open() {
        let menu_pressed = {
            #[cfg(not(any(target_os = "macos", target_os = "redox", windows, target_arch = "wasm32")))]
            { None }

            #[cfg(any(target_os = "macos", target_os = "redox", windows, target_arch = "wasm32"))]
            { window.is_menu_pressed() }
        };
        match menu_pressed {
            Some(IDM_FILE_EXIT) => break,
            Some(IDM_HELP_ABOUT) => show_about_dialog(&window),
            _ => ()
        }

        let mut next_picture_index = None;

        if window.is_key_pressed(Key::Right, KeyRepeat::No)
            || menu_pressed.is_some_and(|id| id == IDM_GO_NEXT)
        {
            next_picture_index = Some((picture_index + 1) % pictures.len());
        }

        if window.is_key_pressed(Key::Left, KeyRepeat::No)
            || menu_pressed.is_some_and(|id| id == IDM_GO_PREVIOUS)
        {
            if next_picture_index.is_none() {
                next_picture_index = Some((if picture_index == 0 { pictures.len() } else { picture_index }) - 1);
            }
            else {
                // cancelled out
                next_picture_index = None;
            }
        }

        if let Some(id) = menu_pressed {
            match id {
                IDM_GO_FIRST => next_picture_index = Some(0),
                IDM_GO_LAST => next_picture_index = Some(pictures.len() - 1),
                _ => ()
            }
        }

        let window_size = window.get_size();
        if let Some(index) = next_picture_index && picture_index != index {
            let picture = &pictures[index];
            window.set_title(&picture.window_title(path, index, pictures.len()));
            window.update_with_buffer(&picture.xrgb, picture.width as _, picture.height as _)?;
            picture_index = index;
        }
        else {
            if prev_window_size == window_size {
                window.update();
            }
            else {
                // wayland and x11 freaks out if the buffer isn't handed to the window again
                let picture = &pictures[picture_index];
                window.update_with_buffer(&picture.xrgb, picture.width as _, picture.height as _)?;
            }
        }
        prev_window_size = window_size;
    }

    Ok(())
}

#[must_use]
fn create_dialog(message: impl Into<String>, level: MessageLevel) -> MessageDialog {
    MessageDialog::new()
        .set_title(env!("CARGO_PKG_NAME"))
        .set_description(message)
        .set_level(level)
        .set_buttons(MessageButtons::Ok)
}

fn show_about_dialog(parent: &Window) {
    let message = format!(
        "{} v{}\n{}\n\n{}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_PKG_REPOSITORY")
    );

    create_dialog(message, MessageLevel::Info)
        .set_parent(parent)
        .show();
}

fn print_help() {
    println!("{} v{}
{}

Usage: {} [OPTIONS] <TIM2_FILE>

Options:
    -h, --help                  Print help
    -b, --no-transparency-grid  Disable the checkerboard background for transparent images",
    
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_DESCRIPTION"),
        env!("CARGO_BIN_NAME")
    );
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

    if !std::io::stdout().is_terminal() {
        // open a file picker if program wasn't launched from the terminal and there is no file argument
        path_opt = path_opt
            .or_else(|| {
                FileDialog::new()
                    .set_title(format!("Open File - {}", env!("CARGO_PKG_NAME")))
                    .add_filter("TIM2 texture", &["tm2"])
                    .pick_file()
            });
    }

    let Some(path) = path_opt else {
        print_help();
        return;
    };

    if let Err(e) = run(&path, no_transparency_grid) {
        create_dialog(e.to_string(), MessageLevel::Error).show();
        std::process::exit(1);
    }
}
