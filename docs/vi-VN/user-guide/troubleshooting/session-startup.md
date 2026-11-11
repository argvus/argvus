---
title: Khắc phục sự cố khởi động phiên
description: Chẩn đoán lỗi phiên người dùng chào hỏi, Hyprland và systemd.
---

Chạy:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```

Kiểm tra riêng lời chào, `argvus start`, Hyprland và đơn vị người dùng liên quan. Việc xây dựng gói thành công không chứng minh được tải trọng đã cài đặt hoặc ghi đè người dùng đang hoạt động là hiện hành.

Sử dụng séc theo thứ tự sau:

1. Xác nhận rằng lời chào đang sử dụng thiết lập lời chào ARGVUS đã cài đặt.
2. Đọc nhật ký phiên và tìm cấu hình Hyprland Lua bị thiếu hoặc không hợp lệ.
3. Kiểm tra trạng thái sẵn sàng và mục tiêu bằng `systemctl --user status argvus-session.target`.
4. Kiểm tra thành phần bị lỗi thay vì khởi động lại mọi dịch vụ cùng một lúc.

Nếu `argvus-session.target` không hoạt động trong khi lớp phủ tải đang hoạt động, hãy kiểm tra
lỗi trình chiếu trước khi khởi động lại dịch vụ:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

`argvus-config` là người viết duy nhất dưới `data/generated/`, vì vậy `project` là
cách có thẩm quyền để tái tạo một thế hệ trong khi chẩn đoán. Khởi động phiên
chạy nó vì chưa có kế hoạch cam kết nào tồn tại; kế hoạch nó viết cho
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` là một
`argvus sessionctl reload` sau này được áp dụng mà không cần chiếu lại.

Phiên dừng lớp phủ tải khi chuẩn bị nghiêm trọng hoặc khởi động mục tiêu
thất bại. Lỗi trực quan của người tiêu dùng nên sử dụng dự phòng được ghi lại hoặc
báo cáo rõ ràng; nó không được ẩn như một phiên thành công.

## Đăng nhập chậm

Nếu `greeter.handoff-takeover-failed` xuất hiện mà không có `handoff.master-acquired`,
kiểm tra `handoff.master-unavailable card=<name> errno=<n>` để mua lại
lỗi. Một người quan sát đã mở trong khi một nhà soạn nhạc khác là chủ nhân có thể trả về lỗi
13 (`EACCES`) ngay cả sau khi bộ tổng hợp đó phát hành bản gốc. Người trợ giúp mở lại
cùng một GPU để có được chủ một cách ngầm định; `handoff.master-reopened` ghi lại điều này
recovery và bộ đệm khung mới được chuẩn bị trên bộ mô tả đó. Người chào hủy người trợ giúp không thành công (`greeter.handoff-cancelled`), vì vậy nó
không thể thêm một lần chờ khác trước khi bộ tổng hợp của người dùng bắt đầu. Một phiên gửi
`release` trong khi chờ mua lại sẽ nhận được `handoff.release-before-master`:
trình trợ giúp đóng các bộ mô tả DRM của nó trước khi xác nhận. Một nhà soạn nhạc lời chào
vụ tai nạn ở Hyprland/Aquamarine là một phát hiện riêng biệt; hủy bỏ người trợ giúp nào
không chứng minh được rằng vấn đề về trình điều khiển hoặc bộ tổng hợp đã được khắc phục.

`greeter.vt-switch-returned active=<tty>` ghi lại VT hoạt động khi đăng nhập
chuyển đổi cuộc gọi trở lại. Thời gian chờ vẽ cũng ghi lại `active-vt=<tty>` trong
lỗi. Đây là những ảnh chụp nhanh, không phải bằng chứng về việc chuyển giao DRM đã hoàn thành.

Để so sánh trước/sau, hãy lưu công việc và sử dụng năm lần đăng nhập cho mỗi phiên bản trên
cùng một thiết lập máy, chủ đề và màn hình. Trước mỗi lần đăng xuất sau, hãy sao chép
`/run/argvus-greeter/timing.log`, nhật ký phiên và nhật ký khởi động ở một nơi riêng biệt
thư mục mẫu trong `/tmp/argvus/`; lần chạy chào tiếp theo sẽ thay thế thời gian của nó
nhật ký. Ghi lại ID khởi động với từng mẫu và chỉ so sánh dấu thời gian trong đó
khởi động. Đo lường xác thực để chuyển giao khung đầu tiên, mục nhập trình bao bọc phiên,
lần phóng Hyprland cuối cùng và dừng quay tải. So sánh mức trung bình và tệ nhất
thời gian xác thực để dừng quay và kiểm tra trực quan các khung màu đen,
nhấp nháy hoặc con quay bị kẹt. Chỉ riêng thời lượng của bề mặt máy tính để bàn không phải là
thời gian đăng nhập từ đầu đến cuối. Lặp lại các thử nghiệm thất bại và phục hồi trong phiên dùng một lần,
mà không cần kết thúc phiên chứa tác phẩm chưa được lưu.

Mỗi bước khởi động ghi lại thời lượng của nó trong nhật ký phiên. Để xem các bước của lần đăng nhập cuối cùng, chậm nhất trước tiên:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```

Để tính thời gian chuyển từ lời chào sang màn hình nền, hãy đọc lời chào và dấu chuyển giao cùng với nhật ký. Tất cả đều sử dụng `CLOCK_MONOTONIC` nên xếp hàng:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` chỉ giữ lần đăng nhập cuối cùng: lời chào, bề mặt chuyển giao DRM (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) và Dấu `greeter.*`. Các dòng `handoff.crtc` ghi lại mọi thay đổi CRTC trong khi quá trình chuyển giao chờ DRM master; một dòng có `fb=0` hoặc `mode_valid=false` hoặc `handoff.output-found ... framebuffer=0`, có nghĩa là bộ tổng hợp lời chào đã tắt đầu ra trước khi chuyển giao, do đó màn hình phải đồng bộ hóa lại. `greeter.watchdog-stop-requested` có nghĩa là phiên chào mừng đã kết thúc mà không cần đợi bộ tổng hợp của nó và `greeter.compositor-exit-deferred` mà bộ tổng hợp được yêu cầu thoát ra sau khi bề mặt chuyển giao không còn nữa; Riêng `greeter.compositor-exit-requested` có nghĩa là nó được yêu cầu thoát ngay lập tức. Nếu dòng lời chào và dòng đăng nhập trong nhật ký hiển thị khoảng cách khoảng nửa giây giữa thời điểm kết thúc phiên chào mừng và thời điểm mở phiên người dùng, thì quá trình phân tích của trình soạn thảo lời chào sẽ diễn ra trong quá trình đăng nhập. Điều đó được mong đợi trong khi nhãn hiển thị `compositor-in-foreground=true`: việc treo máy của thiết bị đầu cuối của người chào sẽ kết thúc bộ tổng hợp trước khi thoát bị trì hoãn. Điều đó xảy ra khi người chào không thể rời khỏi VT trước khi bộ tổng hợp của nó thoát: tìm `greeter.vt-switch-requested`, theo sau là `greeter.handoff-drawing`; khi thiếu cái thứ hai, `greeter.handoff-takeover-failed error=<reason>` sẽ cho biết lý do. `handoff.page-flip` có nghĩa là chuyển giao thay thế khung hình cuối cùng của lời chào mà không có bộ chế độ; `handoff.page-flip-failed errno=<n>` theo sau là `handoff.modeset` có nghĩa là kernel từ chối lật trang. `handoff.overlay plane=<id>` và sau đó là `handoff.retired` có nghĩa là lớp phủ đã ẩn các khung hình đầu tiên của Hyprland cho đến khi trình quay vòng phiên xuất hiện trên màn hình; `handoff.overlay-unavailable` có nghĩa là CRTC không có mặt phẳng lớp phủ trống và `handoff.retire-timeout` có nghĩa là trình quay phiên không bao giờ gửi `retire`. Công cụ quay vòng phiên (`argvus-greeter-handoff`) ghi `first layer-shell frame callback` và `stop requested` vào nhật ký.

Dự kiến ​​sẽ có mục nhập `project.theme-switch` hoặc `prepare.startup-theme` khác 0 sau khi thay đổi giao diện, thay đổi tệp chủ đề, dấu mới hoặc di chuyển tem khởi động cũ hơn. Việc cài đặt lại các tệp giống nhau sẽ bảo tồn bộ nhớ đệm khởi động dựa trên nội dung. Nếu nó xuất hiện trong mỗi lần đăng nhập thì có nghĩa là các tệp chủ đề cụ thể hóa bị thiếu; chạy `argvus theme-switch <theme>` một lần.

Nhật ký phiên được lưu giữ dưới `$XDG_STATE_HOME/argvus/session.log` (thường là `~/.local/state/argvus/session.log`). Sau khi thay đổi ghi đè Hyprland của người dùng, hãy đăng xuất và đăng nhập để điểm vào phiên xác thực và chọn lại. Xem [Phiên đồ họa](/docs/argvus-session/graphical-session/) để biết quy trình khởi động bình thường.