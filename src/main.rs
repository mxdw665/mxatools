mod cli;
mod compression;
mod configuration;
mod environment;
mod logger;
mod tools;

use crate::tools::download_file;
use clap::Parser;
use colored::*;
use std::{
    fs::{create_dir_all, metadata, remove_file},
    process::Command,
};

fn check_file() {
    //设置函数
    let proc_path = "/data/data/com.termux/files/usr/share/at/proc/".to_string();
    let proc_zip = "/data/data/com.termux/files/usr/share/at/proc/proc.tar.gz".to_string();
    let rootfs_json_path = "/data/data/com.termux/files/usr/share/at/json/rootfs.json".to_string();
    let sources_json_path =
        "/data/data/com.termux/files/usr/share/at/json/sources.json".to_string();
    let rootfs_json_url =
        "https://gitee.com/Tidal-team/at-resources/raw/master/json/rootfs.json".to_string();
    let sources_json_url =
        "https://gitee.com/Tidal-team/at-resources/raw/master/json/sources.json".to_string();
    let proc_url = "https://gitee.com/Tidal-team/at-resources/raw/master/proc.tar.gz".to_string();
    //判断是否存在
    if metadata(rootfs_json_path.clone()).is_err() {
        println!("{}", "No rootfs file detected, downloading".yellow());
        let _ = create_dir_all("/data/data/com.termux/files/usr/share/at/json");
        download_file(&rootfs_json_url, &rootfs_json_path);
    }
    if metadata(proc_path.clone()).is_err() {
        println!("{}", "No sources file detected, downloading".yellow());
        let _ = create_dir_all(&proc_path);
        download_file(&proc_url, &proc_zip);
        println!("{}", "Processing File".yellow());
        Command::new("tar")
            .arg("-xf")
            .arg(proc_zip.clone())
            .arg("-C")
            .arg(proc_path)
            .output()
            .expect("Failed to extract films");
        let _ = remove_file(&proc_zip);
    }
    if metadata(sources_json_path.clone()).is_err() {
        println!("{}", "No sources file detected, downloading".yellow());
        let _ = create_dir_all("/data/data/com.termux/files/usr/share/at/json");
        download_file(&sources_json_url, &sources_json_path);
    }
}

fn main() {
    println!("QQ communication group: 687235389");
    check_file();
    cli::Main::parse().run();
}
