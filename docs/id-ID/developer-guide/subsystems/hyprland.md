---
title: Integrasi Hyprland
description: ARGVUS compositor configuration and integration contracts.
---

`argvus-hyprland` mengemas `/usr/share/argvus/hyprland/config/hyprland.lua`, pengikatan kunci, dan skrip pembantu. `argvus-session` meluncurkan Hyprland dan menerapkan pengaturan kompatibilitas sesi; `argvus-config` memproyeksikan file tema, tata letak, batas, dan spasi Hyprland ke dalam `data/generated/hypr/`, dan `argvus-appearance` merekonsiliasi kompositor melalui `hyprctl`; `argvus-display` memasok penggantian monitor.

Jalur paket lawas bukan bagian dari tata letak yang terinstal saat ini.