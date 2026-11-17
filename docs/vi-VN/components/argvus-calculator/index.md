---
title: Máy tính ARGVUS
description: Cơ bản · Khoa học · Lập trình · Tài chính · Chuyển đổi đơn vị
---

# Máy tính ARGVUS

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Chọn hồ sơ trong Cài đặt → Hồ sơ bằng ↑/↓ và Enter hoặc nhấp vào hàng. Hồ sơ và tùy chọn được lưu ngay và khôi phục khi mở lại. Mỗi hồ sơ giữ biểu thức riêng trong phiên. Enter hoặc = thay biểu thức bằng kết quả, không chọn văn bản và đặt con trỏ ở cuối.

Lãi suất là phần trăm mỗi kỳ; dùng cùng kỳ cho lãi suất, số kỳ và khoản trả. Khoản trả vào cuối kỳ; hỗ trợ lãi suất bằng không. Lãi đơn và lãi kép trả tổng tích lũy. Chuyển đổi ngoại tuyến, không có tiền tệ. Lịch sử giữ 100 bản ghi cùng hồ sơ, tùy chọn và đầu vào tài chính; chọn kết quả khôi phục ngữ cảnh. Xóa lịch sử cũng xóa nội dung tệp đã lưu.

## Thiết lập

- `s Thiết lập` → **Hồ sơ** → Cơ bản / Khoa học / Lập trình / Tài chính / Chuyển đổi đơn vị.
- ↑ / ↓ · Enter · Escape / ←.
- **Sao chép kết quả** · **Lịch sử** · **Xóa lịch sử**.
- **Cơ bản**: 370 × 420; **Khoa học**: 440 × 640.
- **Lập trình**: 520 × 640; **Tài chính**: 440 × 640; **Chuyển đổi đơn vị**: 440 × 520.

## Cơ bản / Khoa học

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Đơn vị góc**: Độ / Radian. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 chữ số có nghĩa · 4096 byte · 64 mức phân tích.

## Lập trình

Mặc định: thập phân, 64 bit có dấu. Số học quay vòng theo độ dài từ; dịch phải có dấu mở rộng dấu. Phép chia số nguyên cắt về không. Chia cho không và dịch ngoài [0, bits) là lỗi. Số không có tiền tố dùng cơ số đã chọn. Đổi cơ số chuyển biểu thức hợp lệ; đổi độ dài hoặc dấu diễn giải lại cùng mẫu bit.

- **Cơ số**: 2 / 8 / 10 / 16; **Độ dài từ (bit)**: 8 / 16 / 32 / 64.
- **Có dấu** / **Không dấu** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Có dấu); `-7/2 = -3`.

## Tài chính

Lãi đơn · Lãi kép · Khoản trả cố định · Giá trị hiện tại · Giá trị tương lai · Thay đổi phần trăm.

- `P`: Vốn ban đầu; `FV`: Số tiền tương lai; `A`: Khoản trả mỗi kỳ.
- `r`: Lãi suất mỗi kỳ (%) / 100; `n`: Số kỳ.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Lãi đơn: `P*(1+r*n)`; Lãi kép: `P*G`.
- Khoản trả cố định: `P*G/H`; Giá trị hiện tại: `(FV+A*H)/G`; Giá trị tương lai: `P*G+A*H`.
- Thay đổi phần trăm: `(Số tiền cuối - Số tiền ban đầu) / Số tiền ban đầu * 100`.

## Chuyển đổi đơn vị

Độ dài · Khối lượng · Nhiệt độ · Diện tích · Thể tích · Thời gian · Tốc độ · Lưu trữ dữ liệu.

**Từ** / **Sang**: Thiết lập · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Lịch sử

100 · Hồ sơ + `context` · `Xóa lịch sử`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Văn bản ứng dụng theo Applications Font trong Control Center → Appearance → Desktop Style → Fonts. GTK kế thừa gtk-font-name, giữ kích thước số và tiêu đề. Trình khởi chạy đọc apps_family được tạo và theo dõi thay đổi.

Tiêu đề kết hợp huy hiệu ARGVUS màu nhấn với tên ứng dụng. Chân trang s Settings có thể nhấp nằm trên dải toàn chiều rộng với nền chủ đề riêng, theo phong cách Control Center.
