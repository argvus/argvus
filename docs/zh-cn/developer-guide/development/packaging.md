---
title: 包装
description: 构建并检查 ARGVUS Arch 软件包。
---
每个组件在其仓库下拥有自己的 Arch 打包。检查 `packaging/archlinux/`、包功能和已安装文件列表，然后再记录有效负载。`/usr/share/argvus` 下的包默认值和 `/etc/argvus` 下的管理员配置不可互换。

对于运行时验证，将包内容与 `pacman -Ql <package>` 进行比较，并单独检查活动用户覆盖。

本地构建会将 Cargo 对象保存在 `build/cargo-target` 中，位于包源代码树之外，因此重复 `make build` 只会重新编译已更改的内容。`make clean` 会移除该目录，下次构建将从头开始。

不要仅根据仓库名称推断运行时有效负载。一个包可能包含二进制文件、用户服务、`/usr/share/argvus` 默认值、`/etc/argvus` 管理员配置或资源。安装后通过仓库的 `PKGBUILD` 和 `pacman -Ql` 来确认实际有效负载。
