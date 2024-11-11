use crate::environment::*;
use clap::Parser;
use colored::*;
use log::*;

#[derive(Debug, Parser, Clone)]
pub struct Main {
    #[arg(long, short = 'i', default_value = "")]
    install: String,
    #[arg(long, short = 'r', default_value = "")]
    remove: String,
    #[arg(long, short = 'd', default_value = "")]
    download: String,
    #[arg(long, short = 'l', default_value = "")]
    login: String,
    #[arg(long, short = 'b', default_value = "")]
    bakup: String,
    #[arg(long, short = 'r', default_value = "")]
    restore: String,
    #[arg(long, short = 'L', default_value = "")]
    list: String,
    #[arg(long, short = 'c', default_value = "")]
    configuration: String,
    #[arg(long, default_value = "xz")]
    compress_type: String,
    #[arg(long, default_value = "")]
    compress_package: String,
    #[arg(long, default_value = "xz")]
    decompression_type: String,
    #[arg(long, default_value = "")]
    dns: String,
    #[arg(long, default_value = "")]
    configuration_key: String,
    #[arg(long, default_value = "")]
    configuration_value: String,
    #[arg(long, default_value = "")]
    log_file: String,
    #[arg(long, default_value = "true")]
    log_to_stdout: bool,
    #[arg(long, default_value = "true")]
    log_to_stderr: bool,
}

fn spilt_string(s: &str) -> (&str, String) {
    let mut strings = s.split('/');
    let f_strings = strings.next().unwrap_or_default();
    let r_strings = strings.collect::<Vec<&str>>().join("/");
    (f_strings, r_strings)
}

impl Main {
    fn install_distribution(self, d: &str, v: &str) {
        use crate::tools::{
            container::{download_rootfs, work},
            json::parse_rootfs,
        };
        use std::{
            fs::{create_dir_all, metadata},
            process::Command,
        };

        let path = format!("{}/{}-{}.tar.xz", ROOTFS_PATH, d, v);
        match d {
            "ubuntu" => {
                for i in [
                    "jammy", "focal", "bionic", "xenial", "22.10", "22.04", "20.04", "18.04",
                    "16.04",
                ] {
                    if v == i && metadata(&path).is_err() {
                        info!("Get URL");
                        let temp_url = parse_rootfs("ubuntu", v);
                        download_rootfs(temp_url.unwrap().as_str(), "ubuntu", v);
                    }
                }
            }
            "debian" => {
                for i in ["sid", "bookworm", "bullseye", "buster"] {
                    if v == i && metadata(&path).is_err() {
                        info!("Get URL");
                        let temp_url = parse_rootfs("debian", v);
                        download_rootfs(temp_url.unwrap().as_str(), "debian", v);
                    }
                }
            }
            "alpine" => {
                for i in [
                    "edge", "3.19", "3.18", "3.17", "3.16", "3.15", "3.14", "3.13", "3.12", "3.11",
                    "3.9", "3.8", "3.7",
                ] {
                    if v == i && metadata(&path).is_err() {
                        info!("Get URL");
                        let temp_url = parse_rootfs("alpine", v);
                        download_rootfs(temp_url.unwrap().as_str(), "alpine", v);
                    }
                }
            }
            _ => panic!("Unknown {}", d),
        }
        println!("Unzip the rootfs");
        let unzip_path = format!("{}/{}-{}", CONTAINER_PATH, d, v);
        // 创建文件目录
        let _ = create_dir_all(&unzip_path);
        // 解压文件到指定目录
        let output = Command::new("tar")
            .arg("-xf")
            .arg(&path)
            .arg("-C")
            .arg(&unzip_path)
            .output()
            .expect("Failed to execute command");

        if output.status.success() {
            println!("{}", "Decompression complete".green());
        } else {
            panic!("Decompression failed");
        }
        // 配置容器
        println!("Configuration Container");
        println!("If there are additional options, please configure them in config");
        work(&d, &v, &self.dns);
    }
    // 登录Linux发行版
    fn login_distribution(self, d: &str, v: &str) {
        use crate::tools::json::parse_config;
        use std::{
            fs::{copy, metadata, set_permissions, File, Permissions},
            io::prelude::*,
            os::unix::{fs::PermissionsExt, process::CommandExt},
            process::Command,
        };

        let distribution_path = format!("{}/{}-{}", CONTAINER_PATH, d, v);
        let configuration_path = format!("{}/usr/share/gl/config.json", distribution_path);
        let configuration_temp_path =
            format!("{}/usr/share/gl/config.json.temp", distribution_path);
        if metadata(configuration_temp_path.clone()).is_err() {
            let _ = copy(configuration_path.clone(), configuration_temp_path.clone());
        }
        let login_command_file = format!("{distribution_path}/usr/share/gl/login");
        let mut open_configuration_temp = File::open(configuration_temp_path.clone()).unwrap();
        let mut open_configuration = File::open(configuration_path.clone()).unwrap();
        let mut configuration_contenttext = String::new();
        let mut configuration_temp_contenttext = String::new();
        open_configuration
            .read_to_string(&mut configuration_contenttext)
            .unwrap();
        open_configuration_temp
            .read_to_string(&mut configuration_temp_contenttext)
            .unwrap();
        // 如果登录命令文件存在且配置文件未更改，则执行登录
        if metadata(login_command_file.clone()).is_ok()
            && configuration_contenttext.as_str() == configuration_temp_contenttext.as_str()
        {
            info!("login linux distribution");
            Command::new("sh")
                .arg("-c")
                .arg(&login_command_file)
                .env_remove("LD_PRELOAD")
                .exec();
        } else {
            // 如果配置文件已更改或登录命令文件不存在，则重新配置
            println!("Startup file not detected or configuration file changed");
            info!("parsing configuration");
            let user = parse_config(&configuration_path, "user")
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "user not found in config")
                })
                .map(|s| s.trim_matches('"').to_string())
                .expect("Failed to process language configuration");
            let proot_dir = parse_config(&configuration_path, "proot_dir")
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "proot_dir not found in config",
                    )
                })
                .map(|s| s.trim_matches('"').to_string())
                .expect("Failed to process language configuration");
            let login_bash = parse_config(&configuration_path, "login_bash")
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "login_bash not found in config",
                    )
                })
                .map(|s| s.trim_matches('"').to_string())
                .expect("Failed to process language configuration");
            let mount_sd = parse_config(&configuration_path, "mount_sd")
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "mount_sd not found in config",
                    )
                })
                .map(|s| s.trim_matches('"').to_string())
                .expect("Failed to process language configuration");
            let add_option = parse_config(&configuration_path, "add_option")
                .ok_or_else(|| {
                    std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "add_option not found in config",
                    )
                })
                .map(|s| s.trim_matches('"').to_string())
                .expect("Failed to process language configuration");
            let lang = parse_config(&configuration_path, "lang")
                .ok_or_else(|| {
                    std::io::Error::new(std::io::ErrorKind::NotFound, "lang not found in config")
                })
                .map(|s| s.trim_matches('"').to_string())
                .expect("Failed to process language configuration");
            // 获取用户 UID 和 GID
            let uid_output = Command::new("sh")
                .arg("-c")
                .arg(&format!(
                    "grep \"{}\" {}/etc/passwd | awk -F ':' '{{print $4}}'",
                    user, CONTAINER_PATH
                ))
                .output()
                .expect("Failed to get uid");

            let uid_stdout_str = String::from_utf8_lossy(&uid_output.stdout);
            let proot_uid = uid_stdout_str.trim().to_string();
            let gid_output = Command::new("sh")
                .arg("-c")
                .arg(&format!(
                    "grep \"{}\" {}/etc/passwd | awk -F ':' '{{print $4}}'",
                    user, CONTAINER_PATH
                ))
                .output()
                .expect("Failed to get gid");

            let gid_stdout_str = String::from_utf8_lossy(&gid_output.stdout);
            let proot_gid = gid_stdout_str.trim().to_string();

            // 实现启动
            // 启动音频
            Command::new("pulseaudio")
                .arg("--start")
                .output()
                .expect("Pulseaudio did not start successfully");

            let mut command_login = format!(
            "{proot_dir} --kill-on-exit -L -H -p --link2symlink --rootfs={} --mount=/dev --mount=/proc --mount=/system --mount={}/tmp:/dev/shm --mount={}/proc --mount=/sys --mount=/proc/self/fd/2:/dev/stderr --mount=/proc/self/fd/2:/dec/stdout --mount=/proc/self/fd/0:/dev/stdin --mount=/proc/self/fd:/dev/fd --mount=/dev/urandom:/dev/random",
            distribution_path, distribution_path, distribution_path
        );

            if mount_sd != "false" {
                command_login.push_str(" --mount=/sdcard:/media/sd");
            }

            if add_option != "null" {
                command_login.push_str(&format!(" {}", add_option));
            }

            if user != "root" {
                command_login.push_str(
                &format!(" --cwd=/home/{} --change-id={}:{} /usr/bin/env -i HOME=/home/{} USER={} TERM=xterm-256color TMPDIR=/tmp LANG={} PATH=/usr/local/sbin:/usr/local/bin:/bin:/usr/bin:/sbin:/usr/sbin:/usr/games:/usr/local/games {}",
                user, proot_uid, proot_gid, user, user, lang, login_bash));
            } else {
                command_login.push_str(
            &format!(" --cwd=/root --root-id /usr/bin/env -i HOME=/root USER=root TERM=xterm-256color TMPDIR=/tmp LANG={} PATH=/usr/local/sbin:/usr/local/bin:/bin:/usr/bin:/sbin:/usr/sbin:/usr/games:/usr/local/games {}",
                lang, login_bash));
            };

            let mut open_login_command_file = File::create(login_command_file.clone()).unwrap();
            open_login_command_file
                .write_all(command_login.as_bytes())
                .unwrap();
            let permissions = Permissions::from_mode(0o755);
            let _ = set_permissions(login_command_file.clone(), permissions);
            // 重新复制configuration.json
            info!("writing {}", configuration_path);
            let _ = copy(configuration_path.clone(), configuration_temp_path.clone());
            self.login_distribution(d, v);
        }
    }
    // 显示帮助
    // fn help(self) {
    // println!("Usage:");
    // println!("  install -- Install Linux distribution");
    // println!("  clean -- Clean cache");
    // println!("  bakup -- Bakup Linux distribution");
    // println!("  restore -- Restore Linux distribution");
    // println!("  remove -- Remove Linux distribution");
    // println!("  login -- Login Linux distribution");
    // println!("  list-installed -- List Linux distribution(installed)");
    // }
    // 例出以安装的Linux发行版
    fn list_distribution(self) {
        use std::fs::read_dir;
        let entries = read_dir(CONTAINER_PATH).expect("Failed to read the container directory");

        let directories: Vec<String> = entries
            .filter_map(|entry| {
                // 检查条目是否成功，并且是否为目录
                let entry = entry.ok()?;
                let path = entry.path();
                if path.is_dir() {
                    // 获取目录名称并转换为 String
                    path.file_name()
                        .and_then(|name| name.to_str())
                        .map(|s| s.to_owned())
                } else {
                    None
                }
            })
            .collect();

        if directories.is_empty() {
            panic!("Not installed Linux distribution");
        }

        for directory in directories.iter() {
            println!("{}", directory);
        }
    }
    fn remove_distribution(self, d: &str, v: &str) {
        use colored::*;
        use log::{error, info};
        use std::{
            fs::{remove_dir_all, set_permissions, Permissions},
            os::unix::fs::PermissionsExt,
        };
        let container_path = format!("{}/{}-{}", CONTAINER_PATH, d, v);
        // 递归改变文件夹权限为 777
        println!("setting {} permission", container_path);
        let permissions = Permissions::from_mode(0o777);
        let _ = set_permissions(container_path.clone(), permissions);

        // 删除文件夹
        match remove_dir_all(container_path.clone()) {
            Ok(()) => {
                info!("Deleted: {}", container_path);
                println!("{} Deleted: {}", "Successfully".green(), container_path);
            }
            Err(e) => {
                error!("Failed to delete {}: {}", container_path, e);
                println!("{} Deleted: {}", "Failed".red(), container_path);
                println!("Error: {}", e);
            }
        }
    }
    /*fn spilt_string(self, s:&str) -> (&str, String){
        let mut strings = s.split('/');
        let f_strings = strings.next().unwrap_or_default();
        let r_strings = strings.collect::<Vec<&str>>().join("/");
        (f_strings, r_strings)
    }*/
    fn logger_configuration(&self) {
        crate::logger::Logger {
            log_file: std::path::PathBuf::from(self.log_file.as_str()),
            log_to_stdout: self.log_to_stdout,
            log_to_stderr: self.log_to_stderr,
        }
        .logger_parse();
    }
    fn bakup_distribution(self, d: &str, v: &str) {
        crate::compression::Compress {
            compress_type: self.compress_type,
            distribution: d.to_string(),
            verisons: v.to_string(),
        }
        .compress_type();
    }
    fn restore_distribution(self, d: &str, v: &str) {
        crate::compression::Decompression {
            decompression_type: self.decompression_type,
            compress_package: self.compress_package,
            decompression_distribution: d.to_string(),
            decompression_verisons: v.to_string(),
        }
        .decompression_type();
    }
    fn configuration_distribution(self, d: &str, v: &str) {
        crate::configuration::Configuration {
            distribution: d.to_string(),
            verisons: v.to_string(),
            key: self.configuration_key,
            value: self.configuration_value,
        }
        .parse();
    }
    pub fn run(&self) {
        self.logger_configuration();
        if !self.install.is_empty() {
            let (d, v) = spilt_string(&self.install.as_str());
            self.clone().install_distribution(d, v.as_str());
        }
        if !self.remove.is_empty() {
            let (d, v) = spilt_string(&self.remove.as_str());
            self.clone().remove_distribution(d, v.as_str());
        }
        if !self.download.is_empty() {
            let (d, v) = spilt_string(&self.install.as_str());
            self.clone().install_distribution(d, v.as_str());
        }
        if !self.login.is_empty() {
            let (d, v) = spilt_string(&self.login.as_str());
            self.clone().login_distribution(d, v.as_str());
        }
        if !self.restore.is_empty() {
            let (d, v) = spilt_string(&self.restore.as_str());
            self.clone().restore_distribution(d, &v);
        }
        if !self.bakup.is_empty() {
            let (d, v) = spilt_string(&self.bakup.as_str());
            self.clone().bakup_distribution(d, &v);
        }
        if !self.list.is_empty() {
            self.clone().list_distribution();
        }
        if !self.configuration.is_empty() {
            let (d, v) = spilt_string(&self.configuration.as_str());
            self.clone().configuration_distribution(d, v.as_str());
        }
    }
}
