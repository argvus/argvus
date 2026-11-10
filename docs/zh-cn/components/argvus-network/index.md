---
title: 网络与蓝牙
description: 通过 ARGVUS 管理 NetworkManager 和蓝牙。
---
`argvus network` 子命令用于报告和控制 NetworkManager，而 `argvus bluetooth` 用于报告和控制蓝牙适配器的状态和可用性。它们都在 `argvus` 内部进程中运行；`networkctl` 和 `bluetoothctl` 仍作为别名存在。控制面板和 Waybar 调用这些命令。

在任一命令上使用 `--help` 可查看已安装的子命令。网络和蓝牙的 UI 集成与可选的高级管理器（如 Blueman）是分开的。

控制中心将网络状态与配置页面分开：**网络 → 状态** 总结当前状态，而接口、Wi-Fi、以太网、VPN、DNS、代理和防火墙在提供者可用时为单独的路径。**蓝牙 → 状态**、**设备** 和 **配对** 覆盖适配器状态和设备操作。这些页面执行由提供者支持的系统操作；任务栏和控制面板则提供快速状态/操作界面。

网络和蓝牙的更改可能需要授权，并可能通过 NetworkManager 或蓝牙服务立即生效。它们与更改 ARGVUS 任务栏指示器不同。

控制中心通过标准输入将 Wi-Fi 密码发送到 nmcli，格式为 `--ask`；密码不会出现在进程参数中，并且会从返回的命令错误中删除。

## 命令行网络管理

使用 `argvus networkctl` 在不使用图形界面的情况下查询和控制网络：

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```
## 命令行蓝牙管理

使用 `argvus bluetoothctl` 作为蓝牙适配器：

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```
当内核在 `/sys/class/bluetooth` 中未列出任何控制器时，`status` 会立即响应 `status=no-controller`，而不会询问 `bluetoothctl`（否则在服务停止时会等待 BlueZ）。任务栏模块和控制面板卡片显示相同状态。

在任一命令上使用 `--help` 可查看与您安装的版本特定的附加选项和子命令。

当 Wi-Fi 密码对话框打开时，扫描更新会保留选定的 SSID 和安全类型。如果该网络消失或其安全性发生变化，确认会以错误终止，并需要重新选择。

手动 DNS 会将 IPv4 和 IPv6 服务器发送到各自的 NetworkManager 设置，然后再更新解析器文件。服务管理的解析器符号链接通过拒绝替换文件来保留。成功返回自动 DNS 时，会恢复并删除备份，因此后续的手动操作可保存当前解析器状态。

关闭网络配置文件编辑器时会等待其当前任务完成，因此保存或删除配置文件不会因为丢弃编辑器而中断。后台刷新仍然可以取消。

Wi-Fi 连接请求共享网络操作保护：正在运行的操作或配置文件编辑器任务会在另一个更改开始之前完成。进度和完成消息遵循所选语言。

蓝牙移除确认即使在设备列表刷新时也会保留原始设备地址。bluetoothctl 的电池百分比会被识别，设备详情查询在有限的并发数和单独超时内运行。

Wi-Fi 密码、手动 DNS 和网络搜索字段接受带括号的粘贴。粘贴的控件和换行符会被省略。不支持仅 Ctrl 或仅 Alt 的字符快捷键，不会插入字母。编辑器接受以 Ctrl+Alt 编码的可打印 AltGr 事件；Ctrl+S 仅在未按 Alt 时保存。共享文本编辑器在可视列中保持垂直导航，并为宽字符对齐水平滚动。
