# 配置

在 `/data/data/com.termux/files/usr/share/at/json`
## `rootfs.json`

```json
{
    "system" : {
        "code-name" : "mirror_url"
    }
}
```

- `system` 为容器名，如 ubuntu ，注意为小写。
- `code-name` 为版本号，如 debian 的 bookworm。
```json
{
    "debian" : {
        "bookworm" : "https://mirrors.bfsu.edu.cn/lxc-images/images/debian/bookworm/"
    }
}
```
- `mirror_url` 为镜像地址，默认推荐bfsu.edu.cn。

## `sources.json`
```json
{
    "mirrors" : {
        "system" : {
           "code-name" : [
        "xxx",
        "deb http://mirrors.bfsu.edu.cn/debian/ bookworm-updates main contrib non-free non-free-firmware",
        "deb http://mirrors.bfsu.edu.cn/debian/ bookworm-backports main contrib non-free non-free-firmware",
        "deb http://security.debian.org/debian-security bookworm-security main contrib non-free non-free-firmware"
            ],
        }
    }
}
```
- `mirrors`为镜像站，分别有bfsu、ustc、tuna。
- `system` 为容器名，如 ubuntu ，注意为小写。
- `code-name` 为版本号，如 debian 的 bookworm。
```json
{
    "bfsu" : {
        "debian" : {
           "bookworm" : [
        "deb http://mirrors.bfsu.edu.cn/debian/ bookworm main contrib non-free non-free-firmware",
        "deb http://mirrors.bfsu.edu.cn/debian/ bookworm-updates main contrib non-free non-free-firmware",
        "deb http://mirrors.bfsu.edu.cn/debian/ bookworm-backports main contrib non-free non-free-firmware",
        "deb http://security.debian.org/debian-security bookworm-security main contrib non-free non-free-firmware"
      ],
    }
}
```
- `mirror_url` 为镜像地址，默认推荐bfsu.edu.cn。
 

## `config.json`

```json
{
  "user":"root",
  "proot_dir":"path",
  "login_bash":"shell-bin",
  "mount_sd":"bool",
  "lang":"C.UTF-8",
  "add_option":"proot-opt"
}
```

- `user` 为登录用户，默认为root，也可以设置为其他用户。 
- `proot_dir` 为proot可执行文件路径。 
- `login_bash` 为登录shell，默认为 bash ，可自定义为其他 shell ，如 zsh 。 
- `mount_sd` 是一个布尔值，用于判断是否将SD卡挂载到容器中。默认为 "false" ，如果设置为 "true" ，则 sdcard 会被挂载。 
- `lang` 为设置语言环境， 默认为 "C.UTF-8" 。如果需要中文环境可以设置为 "zh_CN.UTF-8"。 
- `add_option` 为附加proot选项。默认为 `null`。 

-------

注意: 所有的路径（包括proot_dir和login_bash等）都必须是绝对路径，不能使用相对路径。