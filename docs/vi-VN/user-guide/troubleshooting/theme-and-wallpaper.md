---
title: Khắc phục sự cố về chủ đề và hình nền
description: Chẩn đoán việc truyền bá chủ đề và tạo các tệp xuất hiện.
---

Kiểm tra phần `appearance` chuẩn trong `$XDG_CONFIG_HOME/argvus/config.json`, sau đó kiểm tra các tệp được tạo trong `$XDG_CONFIG_HOME/argvus/data/generated`. Các tệp dấu chấm kế thừa chỉ là đầu vào di chuyển.

`argvus-config` là người viết duy nhất dưới `data/generated/`. Để phục hồi một cái bị hỏng
chủ đề hoặc hình nền, xây dựng lại toàn bộ thế hệ từ tài liệu kinh điển và
áp dụng kết quả thay vì chỉnh sửa trực tiếp người tiêu dùng được tạo:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```

`argvus theme-switch` (chạy `argvus config theme-apply`) và `argvus config accent`
là người điều phối: họ xác thực, khóa, hiển thị phần giới thiệu, cam kết thông qua
`argvus-config` và sau đó điều hòa những người tiêu dùng bên ngoài không thể đọc được
`config.json`. Họ không bao giờ viết `data/generated/` và giao dịch chủ đề thì thực hiện
không ghi các tệp GTK, bộ lưu trữ hoặc `mode.css` thông qua các tập lệnh riêng biệt.
Sử dụng `argvus config accent --apply` để chỉ chạy lại phần hòa âm.

Các thay đổi chủ đề chính thức sẽ áp dụng các giá trị giao diện đã lưu của chúng và cập nhật các ứng dụng tiêu dùng trong thời gian chạy bị ảnh hưởng. Việc áp dụng chủ đề tùy chỉnh sẽ khởi động lại thanh tác vụ, các dịch vụ tải lại Hyprland, hình nền và Bảng điều khiển. Việc chuyển đổi Chú ý hoặc Khoảng cách sẽ áp dụng lại khoảng cách, đường viền và lề thanh tác vụ của chế độ đó, sau đó khởi động lại thanh tác vụ, đo từ xa và các dịch vụ tải lại Hyprland. Màu sắc thông báo của Dunst là ngoại lệ khi chế độ Không làm phiền đang hoạt động: quá trình tải lại của chúng bị trì hoãn cho đến khi tắt DND. Nếu một bề mặt khác vẫn giữ màu cũ, hãy kiểm tra xem bản dựng chủ đề có đang hoạt động không và đó có phải là chủ đề bạn đã chọn hay không, sau đó chạy lại quá trình tải lại:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```

Mọi tệp chủ đề đều đến từ bản dựng: chủ đề chưa được cài đặt (không có `theme.toml` trong `/usr/share/argvus/themes/` hoặc `~/.local/share/argvus/themes/`) hoặc không hiển thị sẽ bị từ chối và chủ đề trước đó vẫn giữ nguyên. `argvus theme check DIRECTORY` hiển thị mẫu và dòng của từng lỗi kết xuất. Thanh Waybar chỉ tải lại màu của nó bằng `reload_style_on_change` trong JSONC của nó. Nếu một công tắc có vẻ bị kẹt, hãy kiểm tra xem nó có còn chạy hay không; nó tự ghi lại trong `theme-switch.active`, bên cạnh khóa của nó:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```

Một pid trong điểm đánh dấu vẫn còn hoạt động có nghĩa là quá trình chuyển đổi chưa kết thúc. Công tắc đợi tối đa 60 giây để khóa chủ đề và trình chiếu, cũng như 5 giây cho lệnh tải lại; điểm đánh dấu có quá trình đã thoát đã cũ và công tắc tiếp theo sẽ bỏ qua nó. Bao gồm đầu ra của các lệnh trên khi báo cáo sự cố.

Nếu chỉ lời chào đăng nhập trước có màu trắng hoặc sử dụng chủ đề dự phòng, hãy so sánh nguồn riêng tư với nguồn chiếu công khai của nó:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```

Sửa chữa phép chiếu mà không thay đổi cấu hình chào mừng:

```sh
sudo argvus greeter-setup --sync-themes
```

Tệp đó được xuất bản để người điều phối giao diện xác thực trước và
sống bên ngoài `data/generated/`, vì vậy `argvus config apply` đơn giản thì không
làm mới nó.

Nếu người chào báo cáo `Invalid color name: 'null'` trong bộ đệm Kitty, hãy cập nhật
Gói `argvus terminal` và tạo lại bộ đệm bằng cách mở lại lời chào.
Dấu chuẩn vắng mặt là màu mặc định, không phải màu Kitty; thiết bị đầu cuối hiện tại
launcher từ chối JSON `null` và các giá trị kế thừa không hợp lệ trước khi viết
`active_tab_foreground`.

Nếu thiết bị đầu cuối mặc định ngừng mở sau một số thay đổi chủ đề, hãy kiểm tra bộ đệm Kitty dẫn xuất thay vì chỉnh sửa thủ công:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```

Lệnh trên đọc tùy chọn kinh điển. `.active-theme` chỉ là một
phép chiếu tương thích. Bộ đệm được tạo lại bởi `argvus terminal` và
đồng bộ với ứng dụng chủ đề. Thiếu hoặc cũ
`/usr/lib/argvus/loading-theme/splash` cho biết bản cập nhật gói chưa hoàn chỉnh;
xây dựng lại và cài đặt các gói phối hợp trước khi đánh giá phiên trực tiếp
kết quả.

Nếu viền cửa sổ giữ nguyên màu chủ đề trước đó thì không chỉnh sửa Lua của Hyprland
tập tin theo cách thủ công. Các thay đổi về chủ đề và giọng điệu được chiếu từ `config.json`, sau đó`argvus sessionctl reload` tải lại Hyprland và áp dụng lại các chế độ hoạt động, không hoạt động,
và nhóm các màu đường viền trong suốt vòng đời của phiên:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```

Đường viền *tùy chọn* được thay đổi thông qua `argvus config borders` (hoặc
`borders-switch.sh` đại biểu) trong Trung tâm điều khiển và Bảng điều khiển; tải lại
áp dụng lại các màu đã cam kết trong suốt vòng đời của phiên và không
lấy lại chúng.

Kiểm tra kế hoạch và các giá trị tổng hợp hiệu quả:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```

Nếu `hyprctl` không thể tiếp cận ổ cắm Hyprland hiện tại, thì hình chiếu đường viền là
được báo cáo là thành phần bị lỗi và phụ thuộc không được coi là thành công
đã tải lại. Kiểm tra nhật ký phiên người dùng trước khi thử lại.