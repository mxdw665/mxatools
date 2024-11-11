# MXATOOLS(MX-Android-Tools)【已停更】

一个参考TMOE的专为termux和linux制作的工具箱。现处于预发布阶段，现已完成对 Android 平台的支持。

![](https://img.shields.io/badge/release-v4.0.4-green?style=for-the-badge&logo=rust&logoColor=orange)

使用 Rust 语言编写。

## 什么是 termux？

Termux 是一个 *Android 终端模拟器* 和 **Linux 环境** 应用程序，可直接使用 **无需 root** 或设置。 自动安装最小的基本系统 - 使用 *PKG/APT 包管理器* 可以使用其他包。 更多[此处...](https://termux.com/)

## 如何安装 termux？

您可以从 Google Play 商店或 f-droid 安装 termux。 

- 在 [Github](https://github.com/termux/termux-app) 上获取
- 从[F-Droid](https://f-droid.org/packages/com.termux/)下载

有关安装的更多信息请参见[此处](https://wiki.termux.com/wiki/Main_Page)

## 前提

在您的手机上安装“Termux”。 建议安装 *Termux X11* 应用程序，因为许多桌面元素都依赖于它。如果您需要 VNC，请使用 AVNC 或 VNCViewer。

- Android 版本：大于等于7.0。
- Linux 内核：大于等于4.1。
- CPU 类型：aarch64, arm(no-longer-support)。
- `tar` `curl` `zstd` `aria2c` `proot` `xz` `gzip`命令。

> 注：Debian 系统目前仍然提供 32 位支持。

## 使用

此项目并不复杂，因此，你可以从 Source Code 构建。

```bash
git clone --depth 1 https://gitee.com/Tidal-team/gl-main.git&&cd gl-main&&cargo r -r
```

### 功能

- 安装/管理 Linux 容器。
- TODO: XXX。

## 使用
### 安装Linux

```bash
<target_file> --option install --distribution <distribution> --verisons <verisons>
```

### 备份Linux

```bash
<target_file> --option bakup --distribution <distribution> --verisons <verisons>
```

可选项> --compress_type <compress_type>
默认备份到> /sdcard/Download/bakup/

### 登录Linux

```bash
<target_file> --option login --distribution <distribution> --verisons <verisons>
```

### 恢复Linux

```bash
<target_file> --option restore --distribution <ddecompression_istribution> --verisons <decompression_verisons> --compress_package <compress_package_path>
```

可选项> --compression_type <decompression_type>

### 删除Linux

```bash
<target_file> --option remove --distribution <distribution> --verisons <verisons>
```

### 清除缓存

```bash
<target_file> --option clean
```

### 列出已安装的Linux

```bash
<target_file> --option list-installed
```

---
联系我们：QQ: 687235389
