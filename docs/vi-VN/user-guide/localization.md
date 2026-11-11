---
title: Localization
description: Sử dụng ARGVUS bằng tiếng Anh hoặc tiếng Bồ Đào Nha.
---

`argvus-i18n` cung cấp các danh mục dùng chung, lệnh `argvus i18n validate` và mô-đun QML `org.argvus.i18n`. ARGVUS hiện cung cấp các catalog bằng tiếng Anh và tiếng Bồ Đào Nha.

Các ứng dụng gốc và thành phần shell sử dụng các danh mục được chia sẻ. Do đó, một chuỗi đã dịch có thể được sử dụng bởi nhiều kho lưu trữ.

Sử dụng **Trung tâm điều khiển → Ngôn ngữ & Vùng** để định cấu hình ngôn ngữ, ngôn ngữ khu vực, ngôn ngữ hệ thống được tạo, múi giờ, ngày/giờ và bàn phím. Danh mục ARGVUS hiện tại cung cấp tiếng Anh và tiếng Bồ Đào Nha; điều này tách biệt với danh sách các ngôn ngữ hệ thống mà hệ điều hành đã tạo.

Việc thay đổi ngôn ngữ hiển thị sẽ lưu tùy chọn ARGVUS và tự động khởi động lại `argvus-control-panel.service`, do đó Bảng điều khiển sẽ phản ánh thay đổi ngay lập tức. Các ứng dụng và bề mặt shell khác có thể yêu cầu tải lại phiên hoặc phiên mới. Việc tạo hoặc thay đổi các mục nhập `/etc/locale.gen` cần có sự cho phép và chạy `locale-gen`; việc chọn một ngôn ngữ không được tạo sẽ bị từ chối. Bố cục bàn phím, biến thể và sơ đồ bàn phím của bảng điều khiển là các cài đặt riêng biệt và không nên nhầm lẫn với ngôn ngữ giao diện.

## Khi biến môi trường ghi đè thay đổi

Trước tiên, các ứng dụng ARGVUS phát hiện ngôn ngữ từ tệp ngôn ngữ ARGVUS liên tục, sau đó là `LC_ALL`, `LC_MESSAGES` và `LANG`. Do đó, các biến môi trường cục bộ có thể khiến Trung tâm điều khiển dường như bỏ qua thay đổi ngôn ngữ, đặc biệt khi dịch vụ người dùng hoặc phiên đồ họa vẫn giữ môi trường cũ hơn.

Kiểm tra các giá trị hiệu quả với:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```

Nếu `~/.config/locale.conf` tồn tại và xác định `LANG`, `LC_ALL` hoặc `LC_MESSAGES`, hãy đổi tên tạm thời để nó không còn giới thiệu lại các giá trị đó nữa. Giữ một bản sao lưu thay vì xóa nó:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```

Áp dụng lại ngôn ngữ trong **Trung tâm điều khiển → Ngôn ngữ & khu vực**. Bảng điều khiển được khởi động lại tự động; nếu các thành phần khác vẫn hiển thị ngôn ngữ trước đó, hãy đăng xuất và đăng nhập để phiên đồ họa và `systemd --user` nhận được môi trường mới. Lặp lại các lệnh chẩn đoán sau đó. Xem [Biến môi trường](/docs/reference/environment-variables/) để biết các hợp đồng kỹ thuật và quyền ưu tiên.