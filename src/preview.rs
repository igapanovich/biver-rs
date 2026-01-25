use crate::biver_result::BiverResult;
use eframe::{CreationContext, Frame, NativeOptions};
use egui::{ColorImage, Context, Image, Key, TextureHandle, TextureOptions, ViewportBuilder, ViewportCommand};
use image::ImageFormat;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;

pub fn open_window(preview_file_path: PathBuf) -> BiverResult<()> {
    let file = File::open(&preview_file_path)?;
    let reader = BufReader::new(file);
    let image = image::load(reader, ImageFormat::Jpeg)?;
    let size = [image.width() as usize, image.height() as usize];
    let buffer = image.to_rgba8();
    let pixels = buffer.into_flat_samples();
    let egui_image = ColorImage::from_rgba_unmultiplied(size, pixels.as_slice());

    let opts = NativeOptions {
        centered: true,
        viewport: ViewportBuilder::default().with_inner_size((1024.0, 1024.0)),
        ..NativeOptions::default()
    };

    eframe::run_native("BiVer Previewer", opts, Box::new(|cc| Ok(Box::new(PreviewApp::new(cc, egui_image)))))?;

    Ok(())
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
        if ctx.input(|i| i.key_pressed(Key::Q)) {
            ctx.send_viewport_cmd(ViewportCommand::Close)
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            let ui_size = ui.available_size();

            ui.add(Image::new(&self.image_texture).fit_to_exact_size(ui_size))
        });
    }
}
