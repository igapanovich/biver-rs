use crate::biver_result::BiverResult;
use eframe::{CreationContext, Frame, NativeOptions};
use egui::{ColorImage, Context, Image, Key, TextureHandle, TextureOptions, ViewportBuilder, ViewportCommand};
use image::ImageFormat;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub fn show_preview(image_path: PathBuf) -> BiverResult<()> {
    let image = egui_image_from_file(&image_path)?;

    eframe::run_native("BiVer Previewer", egui_options(), Box::new(|cc| Ok(Box::new(PreviewApp::new(cc, image)))))?;

    Ok(())
}

pub fn show_comparison(image_path1: PathBuf, description1: &str, image_path2: PathBuf, description2: &str) -> BiverResult<()> {
    let image1 = egui_image_from_file(&image_path1)?;
    let image2 = egui_image_from_file(&image_path2)?;

    eframe::run_native(
        &description1,
        egui_options(),
        Box::new(|cc| Ok(Box::new(ComparerApp::new(cc, image1, &description1, image2, description2)))),
    )?;

    Ok(())
}

fn egui_image_from_file(path: &PathBuf) -> BiverResult<ColorImage> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    let image = image::load(reader, ImageFormat::Jpeg)?;
    let size = [image.width() as usize, image.height() as usize];
    let buffer = image.to_rgba8();
    let pixels = buffer.into_flat_samples();
    Ok(ColorImage::from_rgba_unmultiplied(size, pixels.as_slice()))
}

fn egui_options() -> NativeOptions {
    NativeOptions {
        centered: true,
        viewport: ViewportBuilder::default().with_inner_size((1024.0, 1024.0)),
        ..NativeOptions::default()
    }
}

struct PreviewApp {
    image_texture: TextureHandle,
}

impl PreviewApp {
    fn new(cc: &CreationContext, image: ColorImage) -> Self {
        Self {
            image_texture: cc.egui_ctx.load_texture("image", image, TextureOptions::default()),
        }
    }
}

impl eframe::App for PreviewApp {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let q_pressed = ctx.input(|i| i.key_pressed(Key::Q));

        if q_pressed {
            ctx.send_viewport_cmd(ViewportCommand::Close)
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let ui_size = ui.available_size();

            ui.add(Image::new(&self.image_texture).fit_to_exact_size(ui_size))
        });
    }
}

enum SelectedImage {
    Image1,
    Image2,
}

struct ComparerApp<'a> {
    image1_texture: TextureHandle,
    image2_texture: TextureHandle,
    description1: &'a str,
    description2: &'a str,
    selected_image: SelectedImage,
}

impl<'a> ComparerApp<'a> {
    fn new(cc: &CreationContext, image1: ColorImage, description1: &'a str, image2: ColorImage, description2: &'a str) -> Self {
        Self {
            image1_texture: cc.egui_ctx.load_texture("image1", image1, TextureOptions::default()),
            image2_texture: cc.egui_ctx.load_texture("image2", image2, TextureOptions::default()),
            description1,
            description2,
            selected_image: SelectedImage::Image1,
        }
    }
}

impl<'a> eframe::App for ComparerApp<'a> {
    fn update(&mut self, ctx: &Context, _frame: &mut Frame) {
        let (q_pressed, k_pressed, j_pressed, space_pressed) = ctx.input(|i| (i.key_pressed(Key::Q), i.key_pressed(Key::K), i.key_pressed(Key::J), i.key_pressed(Key::Space)));

        if q_pressed {
            ctx.send_viewport_cmd(ViewportCommand::Close)
        }

        let update_title = |selected_image| {
            let description = match selected_image {
                SelectedImage::Image1 => self.description1,
                SelectedImage::Image2 => self.description2,
            };

            ctx.send_viewport_cmd(ViewportCommand::Title(description.to_string()));
        };

        if k_pressed {
            self.selected_image = SelectedImage::Image1;
            update_title(SelectedImage::Image1);
        }

        if j_pressed {
            self.selected_image = SelectedImage::Image2;
            update_title(SelectedImage::Image2);
        }

        if space_pressed {
            match self.selected_image {
                SelectedImage::Image1 => {
                    self.selected_image = SelectedImage::Image2;
                    update_title(SelectedImage::Image2)
                }
                SelectedImage::Image2 => {
                    self.selected_image = SelectedImage::Image1;
                    update_title(SelectedImage::Image1)
                }
            };
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let ui_size = ui.available_size();

            let image_texture = match self.selected_image {
                SelectedImage::Image1 => &self.image1_texture,
                SelectedImage::Image2 => &self.image2_texture,
            };

            ui.add(Image::new(image_texture).fit_to_exact_size(ui_size));
        });
    }
}
