---
title: Tài khoản
description: Hồ sơ người dùng cục bộ, tên hiển thị, hình đại diện và thành viên nhóm, thông qua tài khoản argvus.
---

`argvus accounts` liệt kê và chỉnh sửa tài khoản người dùng cục bộ của máy tính để bàn ARGVUS. Đó là chuyện riêng tư
thực thi của gói `argvus`, được cài đặt tại `/usr/lib/argvus/commands/accounts`. các
Lệnh `argvus` khởi động nó và chỉ các thao tác thay đổi tài khoản hoặc dữ liệu hệ thống khác
chạy bằng root.

## Lệnh

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```

## Hiển thị tên và hình đại diện

Tên hiển thị được lưu trữ trong trường GECOS tiêu chuẩn. Tên người dùng Unix không bao giờ thay đổi.

Hình đại diện là hình ảnh ở định dạng PNG, JPEG hoặc WebP, tối đa 20 MiB. Hình ảnh được thay đổi kích thước thành 256×256
pixel và được lưu dưới dạng PNG với chế độ `0644` tại `~/.face`, vị trí freedesktop được đọc bởi
lời chào và bởi hầu hết các nhà quản lý hiển thị. Hình đại diện của riêng bạn được viết bên trong thư mục chính của bạn và
không yêu cầu ủy quyền.

## Ủy quyền

Chỉ `list`, `show`, `self`, `self groups` và hình đại diện của riêng bạn hoạt động mà không được phép. Mỗi
thay đổi khác yêu cầu tác nhân polkit của máy tính để bàn ủy quyền. Quy tắc ARGVUS
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) quyết định ai phải xác thực:

- người dùng trong nhóm quản trị viên (`wheel`, mặc định Arch hoặc `sudo`) xác nhận bằng chính họ
  mật khẩu;
- bất kỳ người dùng nào khác xác nhận bằng mật khẩu gốc.

| Hoạt động | Không có sự cho phép | Được ủy quyền |
| --- | --- | --- |
| Liệt kê người dùng, hiển thị tài khoản, liệt kê nhóm | ✓ | ✓ |
| Thay đổi hình đại diện của riêng bạn | ✓ | ✓ |
| Thay đổi tên hiển thị của riêng bạn | — | ✓ |
| Thay đổi mật khẩu của riêng bạn | — | ✓ (sau khi chứng minh mật khẩu hiện tại) |
| Thay đổi tên, hình đại diện, nhóm hoặc mật khẩu của người dùng khác | — | ✓ |

Việc thay đổi mật khẩu của chính bạn trước tiên sẽ kiểm tra mật khẩu hiện tại. Mật khẩu sai bị từ chối
trước khi lời nhắc ủy quyền xuất hiện.

Không cần chạy lệnh với `sudo`. Lệnh yêu cầu polkit ủy quyền khi
nó cần nó.

## Giới hạn bảo mật

- `passwd` đọc mật khẩu hiện tại, mới và xác nhận từ đầu vào tiêu chuẩn, mỗi mật khẩu trên một dòng,
  và không bao giờ coi chúng là đối số. Dòng mật khẩu hiện tại có thể trống khi quản trị viên
  đặt lại mật khẩu của người dùng khác. Ví dụ:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  Tương tác, nhập ba dòng và nhấn Enter sau mỗi dòng. Đầu vào được hiển thị trên
  thiết bị đầu cuối khi nó được gõ.
- Việc chỉnh sửa tài khoản dưới UID 1000 (và `nobody`) bị từ chối nên sẽ khóa, xóa tài khoản
  và hết hạn mật khẩu của nó. Tài khoản của riêng bạn không thể bị khóa hoặc xóa.
- Đổi tên hoặc xóa nhóm dưới GID 1000 hoặc nhóm quản trị viên (`wheel`, `sudo`), là
  từ chối; các thành viên của nó vẫn có thể được thay đổi, ngoại trừ việc bạn không thể xóa chính mình khỏi một
  nhóm quản trị viên.
- Thành viên quản trị viên là nhóm `wheel` hoặc nhóm `sudo`. Trung tâm điều khiển bổ sung thêm tính năng mới
  quản trị viên tới `wheel` khi hệ thống có nó và tới `sudo` nếu không. Loại bỏ của riêng bạn
  tư cách thành viên quản trị viên thông qua một chỉnh sửa bị từ chối.

## Ví dụ

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```

## Khắc phục sự cố

- `pkexec was not found`: cài đặt `polkit`. Gói `argvus` phụ thuộc vào nó.
- Lời nhắc ủy quyền không xuất hiện: kiểm tra xem tác nhân xác thực polkit có đang chạy không
  trong phiên của bạn. Gói `argvus` phụ thuộc vào `hyprpolkitagent`, cung cấp một gói.
- Lỗi `unix_chkpwd` khi đổi mật khẩu: gói `pam` cung cấp trợ giúp.
- Chạy `argvus accounts --verbose ...` để in các bước lệnh ra stderr.

Để biết ghi chú của nhà phát triển về ranh giới đặc quyền, hãy xem `docs/internal/argvus-accounts/`.Khi quản trị viên khóa hoặc xóa tài khoản được định cấu hình để đăng nhập tự động, trước tiên nhà cung cấp tài khoản sẽ vô hiệu hóa mục đăng nhập tự động được chào đón. Nếu việc dọn dẹp đó không thành công, hoạt động tài khoản sẽ bị từ chối. Nếu việc khóa hoặc xóa sau đó không thành công, tính năng đăng nhập tự động vẫn bị vô hiệu hóa. Đăng nhập tự động cho tài khoản khác được giữ nguyên; mở khóa tài khoản không cho phép đăng nhập tự động. Các phiên hiện tại không được đóng và lời chào không được khởi động lại.
