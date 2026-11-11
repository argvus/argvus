---
title: شیل سطحیں۔
description: ٹاسک بار، کنٹرول پینل اور ٹیلی میٹری ملکیت۔
---

`argvus-waybar` پیچ شدہ `/usr/bin/waybar` فراہم کرتا ہے۔ `argvus-taskbar` ٹاسک بار ایکشنز اور UX کا مالک ہے۔ یہ جو ٹاسک بار JSONC اور CSS بھیجتا ہے وہ `config.json` کا تخمینہ ہے، اور `argvus-config` `data/generated/waybar/` کے نیچے ایکٹو لیئر کے علاوہ `data/waybar/` کے تحت ایک عکس والی کاپی لکھتا ہے۔ `argvus-control-panel` Quickshell کارڈز کا مالک ہے۔ `argvus-widget-telemetry` اختیاری ٹیلی میٹری بار اور اس کے بلاک سیمنٹکس کا مالک ہے۔ اس کے JSONC اور CSS کو ظاہری شکل میں تبدیلی پر پیکڈ ڈیفالٹس سے `data/waybar/` پر لکھا جاتا ہے، جس کے بعد `argvus-config` صرف حد بندی شدہ `ARGVUS_TELEMETRY_*` اور فونٹ بلاکس کو دوبارہ لکھتا ہے۔ `argvus-session` اپنی صارف خدمات کا مالک ہے۔

اس رن ٹائم ماڈل میں کوئی موجودہ `argvus-shell` پیکیج یا `argvus-shell.service` نہیں ہے۔