---
title: Nhập và xuất chủ đề
description: Xuất chủ đề đang hoạt động dưới dạng chủ đề của riêng bạn, nhập kho lưu trữ chủ đề, áp dụng và xóa nó.
---

Xuất và nhập tác phẩm có **chủ đề**, ở cùng định dạng với chủ đề chính thức. Việc xuất sẽ lưu chủ đề hiện hoạt cùng với màu sắc và hình nền nổi bật của bạn làm chủ đề mới của riêng bạn. Quá trình nhập sẽ cài đặt một kho lưu trữ chủ đề trong số các chủ đề của bạn. Cả hai đều nằm trong **Trung tâm điều khiển → Giao diện → Chủ đề** và chủ đề của bạn được liệt kê trong **Chủ đề tùy chỉnh**.

## Chủ đề được xuất chứa những gì

Kho lưu trữ, `argvus-theme-<id>.tar.gz`, chứa một thư mục chủ đề, được bố trí giống như một chủ đề chính thức:

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```

Tệp kê khai dựa trên chủ đề đang hoạt động, với tên của bạn, id mới, màu nổi bật của bạn làm điểm nhấn trong bảng màu và hình nền bên cạnh nó. Cấu hình giao diện duy trì các cài đặt hình ảnh hiện được áp dụng trong ARGVUS, bao gồm khoảng cách cửa sổ, đường viền, lề thanh tác vụ, bản trình bày bảng điều khiển, màu sắc, hiệu ứng và phông chữ. Hình nền và màu nhấn tùy chỉnh được lưu trữ cùng với chủ đề. Việc áp dụng bất kỳ chủ đề nào sẽ áp dụng lại hồ sơ hoàn chỉnh của nó (hoặc ARGVUS mặc định khi không có hồ sơ nào tồn tại), thay thế các thay đổi trực quan độc lập được thực hiện kể từ lần áp dụng gần đây nhất. Chuyển đổi Chú ý hoặc Khoảng cách luôn áp dụng lại hình dạng cửa sổ và thanh tác vụ của chế độ đó; lưu hình học cá nhân trong một chủ đề tùy chỉnh đã xuất để luôn có sẵn khi chủ đề đó được áp dụng. Các cài đặt không trực quan như tùy chọn âm thanh, bàn phím và mạng không phải là một phần của chủ đề.

Giao diện được áp dụng vẫn tồn tại trong suốt quá trình khởi động lại và tải lại phiên. Việc chuẩn bị phiên sẽ xây dựng lại các tệp cần thiết từ cấu hình hiện tại mà không khôi phục hình dạng của chủ đề gốc.

Các cài đặt được thay đổi sau khi áp dụng một chủ đề chỉ là tạm thời trong lần thay đổi chế độ hoặc chủ đề tiếp theo. Xuất chủ đề từ **Trung tâm điều khiển → Giao diện → Chủ đề → Xuất chủ đề** để giữ nguyên cấu hình hình ảnh hiện tại và khôi phục nó sau bằng cách áp dụng chủ đề đã xuất đó.

Chủ đề của bạn được cài đặt trong `~/.local/share/argvus/themes/<id>/`. Giải nén một kho lưu trữ ở đó cũng cài đặt chủ đề.

## Xuất chủ đề đang hoạt động

1. Mở **Trung tâm điều khiển → Giao diện → Chủ đề**.
2. Chọn **Xuất chủ đề** hoặc nhấn `e` trên trang Chủ đề.
3. Nhập tên cho chủ đề. Trường này hiển thị tập tin sẽ được ghi.
4. Nhấn `Enter`.

Id được tạo từ tên: chữ thường, chữ số và dấu gạch nối. Khi chủ đề đã cài đặt hoặc kho lưu trữ trong thư mục chính của bạn sử dụng id, `-2`, `-3`, v.v. sẽ được thêm vào. Kho lưu trữ được ghi vào thư mục chính của bạn, ví dụ `~/argvus-theme-my-theme.tar.gz`. ARGVUS kiểm tra chủ đề trước khi viết: tệp kê khai phải tải và mọi thành phần phải hiển thị ở cả hai chế độ.

## Nhập chủ đề

1. Sao chép kho lưu trữ `.tar.gz` vào thư mục chính của bạn (`~`). ARGVUS liệt kê các kho lưu trữ có chủ đề.
2. Mở **Trung tâm điều khiển → Giao diện → Chủ đề → Nhập chủ đề** hoặc nhấn `i` trên trang Chủ đề.
3. Chọn kho lưu trữ và nhấn `Enter`.

ARGVUS kiểm tra kho lưu trữ trước khi cài đặt nó. Nó phải chứa một thư mục duy nhất có `theme.toml`, chỉ có các đường dẫn tương đối (không có `..`, không có liên kết), tối đa 128 mục nhập và giới hạn kích thước cho mỗi tệp; id phải khớp với thư mục, hình nền phải nằm trong kho lưu trữ và mọi thành phần phải hiển thị ở cả hai chế độ. Hồ sơ giao diện đi kèm phải sử dụng các phần được hỗ trợ và JSON hợp lệ. Nếu bất kỳ kiểm tra nào không thành công, không có gì được cài đặt.

Chủ đề được thêm vào **Chủ đề tùy chỉnh**. Việc nhập không áp dụng: bạn chọn thời điểm.

### Xung đột tên

- Nếu một trong các chủ đề của bạn có cùng id, ARGVUS sẽ hỏi **Thay thế chủ đề của bạn bằng cùng một ID?**. Chọn **Thay thế** để thay thế hoặc nhấn `Esc` để giữ lại mã của bạn. Chủ đề được thay thế sẽ được giữ làm bản sao lưu cho đến khi cài đặt chủ đề mới.- Một kho lưu trữ có id của một chủ đề chính thức bị từ chối. Xuất lại dưới tên khác.

## Áp dụng chủ đề tùy chỉnh

1. Mở **Trung tâm điều khiển → Giao diện → Chủ đề → Chủ đề tùy chỉnh**.
2. Chọn chủ đề và nhấn `Enter`.

Cấu hình xuất hoàn chỉnh của chủ đề được áp dụng, bao gồm hình nền, khoảng cách, đường viền, màu nhấn, hiệu ứng, phông chữ và bản trình bày bảng điều khiển. Giá trị hình ảnh của nó thay thế các cài đặt hiện tại. ARGVUS sau đó khởi động lại thanh tác vụ, dịch vụ tải lại Hyprland, hình nền và Bảng điều khiển để mỗi bề mặt sử dụng chủ đề tùy chỉnh.

## Xóa chủ đề tùy chỉnh

1. Mở **Trung tâm điều khiển → Giao diện → Chủ đề → Chủ đề tùy chỉnh**.
2. Chọn chủ đề và nhấn `d`.
3. Xác nhận bằng `y` hoặc nhấn `n` hoặc `Esc` để hủy.

Thư mục chủ đề bị xóa khỏi `~/.local/share/argvus/themes`. Nếu đó là chủ đề đang hoạt động thì trước tiên ARGVUS sẽ chuyển sang **ARGVUS Dark**.

## Hồ sơ của các phiên bản trước

Các phiên bản trước đã xuất *hồ sơ chủ đề* (`profile.tar.gz`), một bản sao của cài đặt giao diện. Lần đầu tiên trang Chủ đề mở ra, mỗi cấu hình trong `~/.local/share/argvus/themes/<name>/` sẽ được chuyển đổi thành một chủ đề được tạo từ chủ đề cơ bản, với màu sắc nổi bật và hình nền. Cấu hình ban đầu và danh sách `custom-themes.json` cũ chuyển sang `~/.config/argvus/data/backups/theme-profiles-<seconds>/` và dòng trạng thái báo cáo chuyển đổi. Cấu hình có chủ đề cơ bản chưa được cài đặt sẽ chỉ được chuyển sang bản sao lưu.

## Chia sẻ chủ đề

Chủ đề được xuất là một kho lưu trữ thông thường. Sao chép nó sang một máy khác chạy ARGVUS và nhập nó vào đó.

## Khắc phục sự cố

- **Kho lưu trữ không được liệt kê.** Nó phải nằm ngay trong thư mục chính của bạn, kết thúc bằng `.tar.gz` và giữ `<id>/theme.toml`.
- **Việc nhập bị từ chối.** Dòng trạng thái cho biết lý do: chủ đề chính thức có cùng id, hình nền bị thiếu, tệp kê khai không tải hoặc thành phần không hiển thị.
- **Việc xuất bị từ chối.** Tên cần ít nhất một chữ cái hoặc chữ số và phải cài đặt chủ đề đang hoạt động.