---
title: Hyprland integration
description: ARGVUS compositor configuration and integration contracts.
---

`argvus-hyprland` gói `/usr/share/argvus/hyprland/config/hyprland.lua`, tổ hợp phím và tập lệnh trợ giúp. `argvus-session` khởi chạy Hyprland và áp dụng thiết lập tương thích phiên; `argvus-config` chiếu các tệp chủ đề, bố cục, đường viền và khoảng cách Hyprland vào `data/generated/hypr/` và `argvus-appearance` điều chỉnh bộ tổng hợp trên `hyprctl`; `argvus-display` cung cấp phần ghi đè màn hình.

Đường dẫn gói cũ không phải là một phần của bố cục đã cài đặt hiện tại.