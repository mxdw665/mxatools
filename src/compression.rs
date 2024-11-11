use crate::environment::*;
use log::*;
use std::{
    fs::{create_dir_all, metadata},
    process::Command,
};

#[derive(Debug)]
pub struct Compress {
    pub compress_type: String,
    pub distribution: String,
    pub verisons: String,
}
#[derive(Debug)]
pub struct Decompression {
    pub decompression_type: String,
    pub compress_package: String,
    pub decompression_distribution: String,
    pub decompression_verisons: String,
}

fn find_command(command: &str) -> bool {
    Command::new(command)
        .arg("-h")
        .status()
        .map(|status| status.success())
        .unwrap_or(false)
}

impl Decompression {
    fn decompression_zip(self) {
        info!("finding unzip command");
        if find_command("zip") {
            info!(
                "recovering {} to {} {}",
                self.compress_package, self.decompression_distribution, self.decompression_verisons
            );
            Command::new("unzip")
                .current_dir(&format!(
                    "{}/{}-{}",
                    CONTAINER_PATH, self.decompression_distribution, self.decompression_verisons
                ))
                .arg("-n")
                .arg(&self.compress_package)
                .output()
                .expect(
                    "recover failed
          ",
                );
        } else {
            error!("cannot find unzip command");
            panic!("cannot find unzip command");
        }
    }
    fn decompression_xz(self) {
        info!("finding xz command");
        if find_command("xz") {
            info!(
                "recovering {} to {} {}",
                self.compress_package, self.decompression_distribution, self.decompression_verisons
            );
            Command::new("tar")
                .current_dir(&format!(
                    "{}/{}-{}",
                    CONTAINER_PATH, self.decompression_distribution, self.decompression_verisons
                ))
                .arg("-PpJxvf")
                .arg(&self.compress_package)
                .output()
                .expect(
                    "recover failed
          ",
                );
        } else {
            error!("cannot find xz command");
            panic!("cannot find xz command");
        }
    }
    fn decompression_gzip(self) {
        info!("finding gzip command");
        if find_command("gzip") {
            info!(
                "recovering {} to {} {}",
                self.compress_package, self.decompression_distribution, self.decompression_verisons
            );
            Command::new("tar")
                .current_dir(&format!(
                    "{}/{}-{}",
                    CONTAINER_PATH, self.decompression_distribution, self.decompression_verisons
                ))
                .arg("-Ppzxvf")
                .arg(&self.compress_package)
                .output()
                .expect(
                    "recover failed
          ",
                );
        } else {
            error!("cannot find gzip command");
            panic!("cannot find gzip command");
        }
    }
    pub fn decompression_type(self) {
        if metadata(format!(
            "{}/{}-{}",
            CONTAINER_PATH, self.decompression_distribution, self.decompression_verisons
        ))
        .is_err()
        {
            let _ = create_dir_all(format!(
                "{}/{}-{}",
                CONTAINER_PATH, self.decompression_distribution, self.decompression_verisons
            ));
        }
        match self.decompression_type.as_str() {
            //            "zstd" => self.decompression_zstd(),
            "zip" => self.decompression_zip(),
            "xz" => self.decompression_xz(),
            "gzip" => self.decompression_gzip(),
            _ => self.decompression_xz(),
        }
    }
}
impl Compress {
    fn compress_zip(self) {
        info!("finding zip command");
        if find_command("zip") {
            info!("bakuping {} {}", self.distribution, self.verisons);
            Command::new("zip")
                .current_dir(&format!(
                    "{}/{}-{}",
                    CONTAINER_PATH, self.distribution, self.verisons
                ))
                .arg("-q")
                .arg("-r")
                .arg(&format!(
                    "/sdcard/Download/bakup/{}-{}.zip",
                    self.distribution, self.verisons
                ))
                .arg(".")
                .output()
                .expect(
                    "bakup failed
          ",
                );
        } else {
            error!("cannot find zip command");
            panic!("cannot find zip command");
        }
    }
    fn compress_xz(self) {
        info!("finding xz command");
        if find_command("xz") {
            info!("bakuping {} {}", self.distribution, self.verisons);
            Command::new("tar")
                .current_dir(&format!(
                    "{}/{}-{}",
                    CONTAINER_PATH, self.distribution, self.verisons
                ))
                .arg("-Jcvf")
                .arg(&format!(
                    "/sdcard/Download/bakup/{}-{}.tar.xz",
                    self.distribution, self.verisons
                ))
                .arg(".")
                .output()
                .expect(
                    "bakup failed
          ",
                );
        } else {
            error!("cannot find xz command");
            panic!("cannot find xz command");
        }
    }
    fn compress_gzip(self) {
        info!("finding gzip command");
        if find_command("gzip") {
            info!("bakuping {} {}", self.distribution, self.verisons);
            Command::new("tar")
                .current_dir(&format!(
                    "{}/{}-{}",
                    CONTAINER_PATH, self.distribution, self.verisons
                ))
                .arg("-zcvf")
                .arg(&format!(
                    "/sdcard/Download/bakup/{}-{}.tar.gz",
                    self.distribution, self.verisons
                ))
                .arg(".")
                .output()
                .expect(
                    "bakup failed
          ",
                );
        } else {
            error!("cannot find gzip command");
            panic!("cannot find gzip command");
        }
    }
    pub fn compress_type(self) {
        if metadata("/sdcard/Download/bakup").is_err() {
            let _ = create_dir_all("/sdcard/Download/bakup");
        }
        match self.compress_type.as_str() {
            //            "zstd" => self.compress_zstd(),
            "zip" => self.compress_zip(),
            "xz" => self.compress_xz(),
            "gzip" => self.compress_gzip(),
            _ => self.compress_xz(),
        }
    }
}
