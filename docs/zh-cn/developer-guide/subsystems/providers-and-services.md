---
title: 供应商和服务
description: 提供商包及其系统接口。
---
提供者项目向 UI 使用者公开专注的接口：

- `argvus network` 和 `argvus bluetooth` 集成了 NetworkManager 和 Bluetooth 辅助工具；
- `argvus-display` 将显示器状态与 Hyprland 集成；
- `argvus-power` 集成了空闲、锁定和电源操作；
- `argvus-firewall` 管理 `argvus-firewall.service`；
- argvus 中的 `argvus removable-devices` 辅助工具使用 UDisks2；
- `argvus` 中的 `argvus control-center` `argvus`  辅助工具使用提供者 API 和系统 D-Bus 接口。

控制中心是这些提供者的客户端，而不是它们的替代品。
