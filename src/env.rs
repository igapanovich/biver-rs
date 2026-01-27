use crate::image_magick::ImageMagickEnv;
use crate::xdelta3::XDelta3Env;
use std::path::{Path, PathBuf};

pub struct Env {
    pub xdelta3_path: Option<PathBuf>,
    pub image_magick_path: Option<PathBuf>,
}

impl ImageMagickEnv for Env {
    fn image_magick_path(&self) -> Option<&Path> {
        self.image_magick_path.as_deref()
    }
}

impl XDelta3Env for Env {
    fn xdelta3_path(&self) -> Option<&Path> {
        self.xdelta3_path.as_deref()
    }
}
