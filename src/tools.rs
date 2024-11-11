use colored::*;
use std::{fs::File, io::Write};

pub fn download_file(url: &str, file: &str) {
    use curl::easy::Easy;
    use log::error;
    let mut easy = Easy::new();
    let mut buf = File::create(file).unwrap();
    easy.useragent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/116.0.0.0 Safari/537.36").unwrap();
    easy.url(url).unwrap();
    easy.write_function(move |data| {
        buf.write_all(data).unwrap();
        Ok(data.len())
    })
    .unwrap();
    match easy.perform() {
        Ok(_) => {
            let correct_code: u32 = "200".parse().unwrap();
            if easy.response_code().unwrap() != correct_code {
                panic!("Download of  {} failed", url.red());
            }
        }
        Err(e) => {
            error!("Download failed of:{}", e);
            panic!("Download failed of:{}", e);
        }
    }
}

pub mod json {
    use crate::environment::*;
    use log::*;
    use serde_json::Value;
    use std::fs::File;
    use std::io::Read;

    pub fn parse_config(path: &str, option: &str) -> Option<String> {
        let mut file = match File::open(path) {
            Ok(file) => file,
            Err(e) => {
                error!("File open failed:{}", e);
                panic!("error: File open failed");
            }
        };
        let mut contenttext = String::new();

        if file.read_to_string(&mut contenttext).is_err() {
            error!("Read file failed");
            panic!("error: Read file failed");
        }
        let json: Value = match serde_json::from_str(&contenttext) {
            Ok(json) => json,
            Err(e) => {
                error!("JSON parse failed:{}", e);
                panic!("error: JSON parse failed");
            }
        };

        if let Some(option1) = json.get(option) {
            return Some(option1.to_string());
        }

        None
    }
    pub fn parse_rootfs(distribution: &str, verisons: &str) -> Option<String> {
        let json_path = format!("{}/rootfs.json", PROPERTIES_PATH);
        let mut file = match File::open(json_path) {
            Ok(file) => file,
            Err(e) => {
                error!("File open failed:{}", e);
                panic!("error: File open failed");
            }
        };
        let mut contenttext = String::new();
        if file.read_to_string(&mut contenttext).is_err() {
            error!("Read file failed");
            panic!("error: Read file failed");
        }
        let json: Value = match serde_json::from_str(&contenttext) {
            Ok(json) => json,
            Err(e) => {
                error!("JSON parse failed:{}", e);
                panic!("error: JSON parse failed");
            }
        };
        if let Some(ds) = json.get(distribution) {
            if let Some(vs) = ds.get(verisons) {
                if let Some(url) = vs.as_str() {
                    return Some(url.to_string());
                }
            }
        }

        None
    }
}
pub mod container {
    use crate::environment::*;
    use colored::*;
    use log::*;
    use std::{
        env::consts::ARCH,
        fs::{create_dir_all, remove_file, set_permissions, write, File, Permissions},
        os::unix::fs::PermissionsExt,
        os::unix::process::CommandExt,
        process::{exit, Command},
    };

    fn arch_get() -> &'static str {
        (match ARCH {
            "aarch64" => "arm64",
            "arm" => "armhf",
            _ => todo!(),
        }) as _
    }

    pub fn download_rootfs(url: &str, distribution: &str, verisons: &str) {
        let arch = arch_get();
        let new_url = format!("{url}{arch}/default/");
        let save_path = ROOTFS_PATH;
        let dl = format!("aria2c {new_url}\"$(curl {new_url} | grep href | tail -n 2 | cut -d '\"' -f 4 | head -n 1)\"rootfs.tar.xz -d {save_path} -o {distribution}-{verisons}.tar.xz");
        println!("Downlading rootfs");
        let output = Command::new("sh")
            .arg("-c")
            .arg(dl)
            .output()
            .expect("Failed to execute aria2c");
        if output.status.success() {
            info!("Download rootfs successful");
            println!("{}", "Download successful".green());
        } else {
            error!(
                "Download rootfs failed:{}",
                String::from_utf8_lossy(&output.stderr)
            );
            println!("{}", "Download failed".red());
            exit(415);
        }
    }

    pub fn work(distribution: &str, verisons: &str, dns: &str) {
        use std::io::Write;
        info!("delete resolv");
        println!("In the process of dealing with the aftermath");
        let resolv = format!(
            "{}/{}-{}/etc/resolv.conf",
            CONTAINER_PATH, distribution, verisons
        );
        let _ = remove_file(&resolv);
        info!("get dns");
        let dns_url = format!(
            "https://gitee.com/Tidal-team/at-resources/raw/master/dns/{}",
            dns
        );
        let dns_content = Command::new("curl")
            .arg(dns_url)
            .output()
            .expect("DNS acquisition failed");
        if String::from_utf8_lossy(&dns_content.stdout).contains("Repository or file not found") {
            panic!("Unknown dns");
        }
        Command::new("rm")
            .arg("-rf")
            .arg(&resolv)
            .output()
            .expect("Deletion of DNS file failed");
        Command::new("touch")
            .arg(&resolv)
            .output()
            .expect("Failed to create new DNS file");
        let mut file = File::create(&resolv).expect("Failed to create resolv.conf");
        file.write_all(&dns_content.stdout)
            .expect("Failed to write to resolv.conf");
        info!("copy proc");
        println!("Copy proc films");
        let proc_path = format!("{}/proc", CONTAINER_PATH);
        let proc = format!("{}/*", PROC_PATH);
        Command::new("cp")
            .arg("-r")
            .arg(&proc)
            .arg(proc_path)
            .output()
            .expect("Copy failed");
        let container = format!("{}/{}-{}", CONTAINER_PATH, distribution, verisons);
        let gl_repo = format!("{}/usr/share/at/", container);
        let _ = create_dir_all(gl_repo.clone());
        if std::fs::metadata(format!("{}/usr/share/at/config.json", container)).is_err() {
            crate::tools::download_file(
                "https://gitee.com/Tidal-team/at-resources/raw/master/json/config.json",
                &format!("{}/usr/share/at/config.json", container),
            );
        }
        let script = format!("{}/aftermath_script.sh", gl_repo);
        let mut _package_install;
        let mut _package_update;
        match distribution {
            "arch" => {
                _package_install = "/bin/pacman";
                _package_update = "pacman -Syy";
            }
            "fedora" | "centos" => {
                _package_install = "/bin/yum";
                _package_update = "yum update&&yum upgrade";
            }
            "debian" | "ubuntu" => {
                _package_install = "/bin/apt";
                _package_update = "apt update&&apt upgrade";
            }
            _ => {
                exit(400);
            }
        }
        //创建文件
        let _ = File::create(script.clone());
        //写入文件
        let content = format!(
            r#"#!/bin/bash
RED=$(printf '\033[31m')
GREEN=$(printf '\033[32m')
YELLOW=$(printf '\033[33m')
BLUE=$(printf '\033[34m')
PURPLE=$(printf '\033[35m')
CYAN=$(printf '\033[36m')
RESET=$(printf '\033[m')
BOLD=$(printf '\033[1m')
if [ ! -f {_package_install} ]; then
	echo -e "{_package_install} not find"
	exit
fi
printf "%s\n" "${{YELLOW}}Updateing${{RESET}}"
{_package_update}
if ! grep -q "localhost" /etc/hosts 2>/dev/null; then
	cat >>/etc/hosts <<-EOF
		127.0.0.1       localhost
		::1     localhost ip6-localhost ip6-loopback
		ff02::1 ip6-allnodes
		ff02::2 ip6-allrouters
		EOF
	fi
	if [[ -s "/tmp/hostname" ]]; then
		NEW_HOST_NAME=$(head -n 1 /tmp/hostname)
		cp -f /tmp/hostname /etc
		if ! grep -q "$NEW_HOST_NAME" /etc/hosts 2>/dev/null; then
			printf "%s\n" \
			"127.0.0.1 $NEW_HOST_NAME" \
			"::1 $NEW_HOST_NAME" \
			>>/etc/hosts
		fi
	fi
	if [[ $(command -v hostname) ]]; then
		if ! grep -q "$(hostname)" /etc/hosts 2>/dev/null; then
			printf "%s\n" \
			"127.0.0.1 $(hostname) $(hostname -f)" \
			"::1 $(hostname) $(hostname -f)" \
			>>/etc/hosts
		fi
	fi
	[[ ! -e /etc/hostname ]] || cat /etc/hostname
	[[ ! -e /etc/hosts ]] || cat /etc/hosts
	if [[  $(command -v groupadd) ]];then
		groupadd aid_system -g 1000 || groupadd aid_system -g 1074
		groupadd aid_radio -g 1001
		groupadd aid_bluetooth -g 1002
		groupadd aid_graphics -g 1003
		groupadd aid_input -g 1004
		groupadd aid_audio -g 1005
		groupadd aid_camera -g 1006
		groupadd aid_log -g 1007
		groupadd aid_compass -g 1008
		groupadd aid_mount -g 1009
		groupadd aid_wifi -g 1010
		groupadd aid_adb -g 1011
		groupadd aid_install -g 1012
		groupadd aid_media -g 1013
		groupadd aid_dhcp -g 1014
		groupadd aid_sdcard_rw -g 1015
		groupadd aid_vpn -g 1016
		groupadd aid_keystore -g 1017
		groupadd aid_usb -g 1018
		groupadd aid_drm -g 1019
		groupadd aid_mdnsr -g 1020
		groupadd aid_gps -g 1021
		groupadd aid_media_rw -g 1023
		groupadd aid_mtp -g 1024
		groupadd aid_drmrpc -g 1026
		groupadd aid_nfc -g 1027
		groupadd aid_sdcard_r -g 1028
		groupadd aid_clat -g 1029
		groupadd aid_loop_radio -g 1030
		groupadd aid_media_drm -g 1031
		groupadd aid_package_info -g 1032
		groupadd aid_sdcard_pics -g 1033
		groupadd aid_sdcard_av -g 1034
		groupadd aid_sdcard_all -g 1035
		groupadd aid_logd -g 1036
		groupadd aid_shared_relro -g 1037
		groupadd aid_dbus -g 1038
		groupadd aid_tlsdate -g 1039
		groupadd aid_media_ex -g 1040
		groupadd aid_audioserver -g 1041
		groupadd aid_metrics_coll -g 1042
		groupadd aid_metricsd -g 1043
		groupadd aid_webserv -g 1044
		groupadd aid_debuggerd -g 1045
		groupadd aid_media_codec -g 1046
		groupadd aid_cameraserver -g 1047
		groupadd aid_firewall -g 1048
		groupadd aid_trunks -g 1049
		groupadd aid_nvram -g 1050
		groupadd aid_dns -g 1051
		groupadd aid_dns_tether -g 1052
		groupadd aid_webview_zygote -g 1053
		groupadd aid_vehicle_network -g 1054
		groupadd aid_media_audio -g 1055
		groupadd aid_media_video -g 1056
		groupadd aid_media_image -g 1057
		groupadd aid_tombstoned -g 1058
		groupadd aid_media_obb -g 1059
		groupadd aid_ese -g 1060
		groupadd aid_ota_update -g 1061
		groupadd aid_automotive_evs -g 1062
		groupadd aid_lowpan -g 1063
		groupadd aid_hsm -g 1064
		groupadd aid_reserved_disk -g 1065
		groupadd aid_statsd -g 1066
		groupadd aid_incidentd -g 1067
		groupadd aid_secure_element -g 1068
		groupadd aid_lmkd -g 1069
		groupadd aid_llkd -g 1070
		groupadd aid_iorapd -g 1071
		groupadd aid_gpu_service -g 1072
		groupadd aid_network_stack -g 1073
		groupadd aid_shell -g 2000
		groupadd aid_cache -g 2001
		groupadd aid_diag -g 2002
		groupadd aid_oem_reserved_start -g 2900
		groupadd aid_oem_reserved_end -g 2999
		groupadd aid_net_bt_admin -g 3001
		groupadd aid_net_bt -g 3002
		groupadd aid_inet -g 3003
		groupadd aid_net_raw -g 3004
		groupadd aid_net_admin -g 3005
		groupadd aid_net_bw_stats -g 3006
		groupadd aid_net_bw_acct -g 3007
		groupadd aid_readproc -g 3009
		groupadd aid_wakelock -g 3010
		groupadd aid_uhid -g 3011
		groupadd aid_everybody -g 9997
		groupadd aid_misc -g 9998
		groupadd aid_nobody -g 9999
		groupadd aid_app_start -g 10000
		groupadd aid_app_end -g 19999
		groupadd aid_cache_gid_start -g 20000
		groupadd aid_cache_gid_end -g 29999
		groupadd aid_ext_gid_start -g 30000
		groupadd aid_ext_gid_end -g 39999
		groupadd aid_ext_cache_gid_start -g 40000
		groupadd aid_ext_cache_gid_end -g 49999
		groupadd aid_shared_gid_start -g 50000
		groupadd aid_shared_gid_end -g 59999
		groupadd aid_overflowuid -g 65534 2>/dev/null || groupadd aid_overflowuid -g 65535
		#添加65534 group将导致opensuse的system-user-nobody配置失败。
		groupadd aid_isolated_start -g 99000
		groupadd aid_isolated_end -g 99999
		groupadd aid_user_offset -g 100000
		#usermod -a -G aid_bt,aid_bt_net,aid_inet,aid_net_raw,aid_admin root
		usermod -a -G aid_system,aid_radio,aid_bluetooth,aid_graphics,aid_input,aid_audio,aid_camera,aid_log,aid_compass,aid_mount,aid_wifi,aid_adb,aid_install,aid_media,aid_dhcp,aid_sdcard_rw,aid_vpn,aid_keystore,aid_usb,aid_drm,aid_mdnsr,aid_gps,aid_media_rw,aid_mtp,aid_drmrpc,aid_nfc,aid_sdcard_r,aid_clat,aid_loop_radio,aid_media_drm,aid_package_info,aid_sdcard_pics,aid_sdcard_av,aid_sdcard_all,aid_logd,aid_shared_relro,aid_dbus,aid_tlsdate,aid_media_ex,aid_audioserver,aid_metrics_coll,aid_metricsd,aid_webserv,aid_debuggerd,aid_media_codec,aid_cameraserver,aid_firewall,aid_trunks,aid_nvram,aid_dns,aid_dns_tether,aid_webview_zygote,aid_vehicle_network,aid_media_audio,aid_media_video,aid_media_image,aid_tombstoned,aid_media_obb,aid_ese,aid_ota_update,aid_automotive_evs,aid_lowpan,aid_hsm,aid_reserved_disk,aid_statsd,aid_incidentd,aid_secure_element,aid_lmkd,aid_llkd,aid_iorapd,aid_gpu_service,aid_network_stack,aid_shell,aid_cache,aid_diag,aid_oem_reserved_start,aid_oem_reserved_end,aid_net_bt_admin,aid_net_bt,aid_inet,aid_net_raw,aid_net_admin,aid_net_bw_stats,aid_net_bw_acct,aid_readproc,aid_wakelock,aid_uhid,aid_everybody,aid_misc,aid_nobody,aid_app_start,aid_app_end,aid_cache_gid_start,aid_cache_gid_end,aid_ext_gid_start,aid_ext_gid_end,aid_ext_cache_gid_start,aid_ext_cache_gid_end,aid_shared_gid_start,aid_shared_gid_end,aid_isolated_start,aid_isolated_end,aid_user_offset root
		usermod -g aid_inet _apt 2>/dev/null
		usermod -a -G aid_inet,aid_net_raw portage 2>/dev/null
	fi
	printf "%s\n" "${{YELLOW}}Installing necessary software package${{RESET}}"
	if [[ $(command -v apt) ]]; then
		if [[ ! $(command -v eatmydata) ]]; then
			printf "%s\n" "${{GREEN}}apt ${{YELLOW}}install -y ${{BLUE}}eatmydata${{RESET}}"
			apt update 2>/dev/null || apt update 2>/dev/null
			apt install -y eatmydata || apt install -y -f eatmydata
		else
			printf "%s\n" "${{GREEN}}eatmydata apt ${{BLUE}}update${{RESET}}"
			eatmydata apt update || apt update
		fi
		if [ ! $(command -v locale-gen) ]; then
			printf "%s\n" "${{GREEN}}eatmydata apt ${{YELLOW}}install -y ${{BLUE}}locales${{RESET}}"
			eatmydata apt install -y locales 2>/dev/null || apt install -y locales 2>/dev/null
		fi
	fi
	if [[ $(command -v apt) ]]; then
		if [[ ! $(command -v eatmydata) ]]; then
			printf "%s\n" "${{GREEN}}apt ${{YELLOW}}install -y ${{BLUE}}eatmydata${{RESET}}"
			apt update 2>/dev/null || apt update 2>/dev/null
			apt install -y eatmydata || apt install -y -f eatmydata
		else
			printf "%s\n" "${{GREEN}}eatmydata apt ${{BLUE}}update${{RESET}}"
			eatmydata apt update || apt update
		fi
		if [[ -n $(command -v perl) ]]; then
			printf "%s\n" "${{GREEN}}eatmydata apt ${{YELLOW}}reinstall -y ${{BLUE}}perl-base${{RESET}}"
			eatmydata apt-get install --reinstall -y perl-base 2>/dev/null || apt-get install --reinstall -y perl-base 2>/dev/null
		fi
		if [ ! $(command -v locale-gen) ]; then
			printf "%s\n" "${{GREEN}}eatmydata apt ${{YELLOW}}install -y ${{BLUE}}locales${{RESET}}"
			eatmydata apt install -y locales 2>/dev/null || apt install -y locales 2>/dev/null
		fi
	fi
	printf "%s\n" "${{GREEN}}apt ${{YELLOW}}reinstall -y ${{BLUE}}perl-base${{RESET}}"
	apt-get install --reinstall -y perl-base 2>/dev/null
	for i in perlbug unzip pigz; do
		if [[ -L /usr/bin/$i ]]; then
			case $i in
			perlbug) DEP="perl" ;;
			*) DEP=$i ;;
			esac
			printf "%s\n" "${{GREEN}}apt ${{YELLOW}}reinstall -y ${{BLUE}}${{DEP}}$RESET"
			apt-get install --reinstall -y $DEP 2>/dev/null
		fi
	done
	for i in 6 7 8 9 10; do
		if [[ -L "/usr/bin/python3.${{i}}m" ]]; then
			printf "%s\n" "${{GREEN}}apt ${{YELLOW}}reinstall -y ${{BLUE}}python3.$i-minimal${{RESET}}"
			apt-get install --reinstall -y python3.$i-minimal 2>/dev/null
			dpkg --configure -a
			break
		fi
	done
"#
        );
        let permissions = Permissions::from_mode(0o755);
        let _ = set_permissions(script.clone(), permissions);
        let content_u8 = content.as_bytes();
        let _ = write(script, content_u8);
        Command::new("proot").env_remove("LD_PRELOAD").arg("-S").arg(container).arg("-b").arg("/proc").arg("-b").arg("/dev").arg("-b").arg("/sys").arg("/usr/bin/env").arg("-i").arg("HOME=/root").arg("USER=root").arg("TERM=xterm-256color").arg("TMPDIR=/tmp").arg("LANG=C.UTF-8").arg("PATH=/usr/local/sbin:/usr/local/bin:/bin:/usr/bin:/sbin:/usr/sbin:/usr/games:/usr/local/games").arg("/usr/share/at/aftermath_script.sh").exec();
    }
}
