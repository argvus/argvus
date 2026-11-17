---
title: Máy tính ARGVUS
description: Máy tính ARGVUS — GTK4 / ARGVUS
---

# Máy tính ARGVUS

`argvus calculator` · launcher: **Máy tính ARGVUS**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Khoa học**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radian**: Bỏ chọn: độ.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Xóa**; Backspace: **Xóa ký tự**; Tab / Shift+Tab.
- **Sao chép kết quả**; **Lịch sử**: 100; **Xóa lịch sử**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Lịch sử

Lịch sử được giữ sau khi đóng (tối đa 100 kết quả). Xóa lịch sử cũng xóa dữ liệu đã lưu. Cài đặt: ↑/↓ để di chuyển, Enter hoặc nhấp để kích hoạt, Escape để quay lại.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Luôn mở trong cửa sổ nổi nhỏ ở giữa (370 × 420); chế độ khoa học có thể mở rộng cửa sổ.

Màn hình chính chỉ có biểu thức, kết quả và bàn phím. Menu cung cấp chế độ khoa học, đơn vị góc, sao chép và màn hình lịch sử riêng. Quay lại hoặc Escape trở về tính toán; chọn kết quả lịch sử để dùng lại. Chế độ cơ bản khôi phục cửa sổ nhỏ và giữ biểu thức.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Phóng to cửa sổ sẽ tăng kích thước các phím. Mỗi lần khởi chạy mới dùng kích thước mặc định. Màn hình phụ có mũi tên quay lại ở trên.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

Chế độ cơ bản hoặc khoa học được lưu ngay và khôi phục khi mở lại.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
