---
title: Control Center
description: Định cấu hình các phần liên tục của màn hình ARGVUS của bạn.
---

Trung tâm điều khiển là ứng dụng cài đặt bàn phím đầu tiên của ARGVUS. Sử dụng nó khi bạn muốn định cấu hình máy tính để bàn thay vì thực hiện hành động một lần.

## Trung tâm điều khiển và Bảng điều khiển

| Sử dụng… | Vì… |
| --- | --- |
| **Trung tâm điều khiển** | Chủ đề, hình nền, hiệu ứng, bố cục, hoạt động của thanh tác vụ, phông chữ, ứng dụng mặc định, cài đặt bàn phím và phương thức nhập, ngôn ngữ, khu vực và cấu hình hệ thống. |
| **Bảng điều khiển** | Trạng thái hiện tại và các hành động thường xuyên như âm lượng, độ sáng, mạng, thông báo, nguồn và điều khiển phiên. |

Bảng Điều khiển có thể hiển thị các điều khiển nhanh cho một cài đặt được cấu hình vĩnh viễn trong Trung tâm Kiểm soát. Ví dụ, sử dụng bảng để thực hiện nhanh thao tác về giao diện hoặc hiển thị, và Trung tâm Kiểm soát khi bạn muốn thay đổi cấu hình màn hình nền phía sau nó. Xem Bảng Điều khiển và [Cấu hình ở đâu](/docs/user-guide/where-to-configure/).

## Lĩnh vực chính

Trang chủ Trung tâm điều khiển hiện tại cung cấp các khu vực sau:

Ngôi nhà là một danh mục các điểm đến được nhóm thành sáu thẻ. Mỗi điểm đến sẽ mở trực tiếp trang của nó và tìm kiếm sẽ tìm thấy nó dưới bất kỳ thẻ nào.

- **Giao diện** — tùy chỉnh màn hình nền: Kiểu màn hình (chủ đề, chế độ giao diện, màu nhấn, hình nền, biểu tượng và phông chữ), Kiểu ứng dụng (độ trong suốt của thiết bị đầu cuối và trình khởi chạy), Thanh tác vụ, Bảng điều khiển và Đo từ xa tiện ích.
- **Máy tính để bàn & Đầu vào** — định cấu hình không gian làm việc: Hyprland (Windows: Quy tắc, Dấu cách và Chế độ, hoạt ảnh, làm mờ, đường viền, bóng và phiên bản đã cài đặt), Màn hình, Bàn phím & Phím tắt và Chuột & Bàn di chuột.
- **Tùy chọn** — cài đặt cá nhân và khu vực: Ngôn ngữ & Khu vực (ngôn ngữ, múi giờ, định dạng và trung tâm bố cục bàn phím), Ngày & Giờ, Ứng dụng mặc định, Nguồn và Phiên.
- **Phần cứng & Kết nối** — thiết bị và kết nối: Tổng quan về phần cứng, Mạng & Wi-Fi, Bluetooth, Âm thanh và Bộ lưu trữ.
- **Quản trị hệ thống** — quản lý hệ điều hành: Gói, Dịch vụ, Khởi động, Tên máy chủ, Người dùng & Nhóm, Tường lửa và Thông báo (Không làm phiền).
- **Cài đặt** — chính Trung tâm điều khiển, được liệt kê sau các thẻ: Ngôn ngữ (ngôn ngữ người dùng) và Kiểu (biểu tượng trang trí và độ trong suốt). Nhấn `s` để mở nó.
- **Công cụ & Chẩn đoán** — công cụ dành cho nhà phát triển và bảo trì: ARGVUS Welcome, Projects, Snippets (văn bản được lưu cho `argvus snippets`, được nhập bằng `SUPER + ALT + N`), khóa SSH (cặp khóa trong `~/.ssh` do `argvus keys` quản lý), khóa GPG (khóa được quản lý bởi `argvus gpg`; gpg-agent sử dụng pinentry-gtk), Dịch vụ dành cho nhà phát triển, Bác sĩ hệ thống và Giới thiệu về ARGVUS.

Khi Không làm phiền được bật, cửa sổ bật lên thông báo sẽ đóng và thông báo mới sẽ được giữ lại để sử dụng sau này. Bảng điều khiển ẩn lịch sử thông báo cho đến khi tính năng Không làm phiền bị tắt.

## ARGVUS Chào mừng

Chọn **Mở ARGVUS Chào mừng** trong **Công cụ & Chẩn đoán**, tìm kiếm `welcome` hoặc chạy `argvus welcome`. Chào mừng được tích hợp vào Trung tâm điều khiển và sử dụng cửa sổ duy nhất của nó. Enter mở trang cài đặt đã chọn; khi hoàn tất, hãy quay lại hướng dẫn bằng Esc. F6 Tiếp theo kiểm tra bước hiện tại và mở bước tiếp theo trong cùng một cửa sổ. Esc chỉ quay lại và Enter trên danh sách kiểm tra luôn mở trang đã chọn. O xem lại bước này, S bỏ qua mà không đánh dấu và F hoàn thành hướng dẫn ngay cả khi các bước tùy chọn đang chờ xử lý. Hủy bỏ hoặc quay lại không hoàn thành một bước. Tiến bộ hiện tại được bảo tồn.

Hướng dẫn bao gồm ngôn ngữ và khu vực, màn hình, phần cứng và trình điều khiển, mạng, âm thanh, Bluetooth, giao diện, nguồn, ứng dụng mặc định và khởi động phiên. Nó không bao giờ cài đặt trình điều khiển hoặc gói và không tự áp dụng các thay đổi hệ thống. Tiến trình được lưu trữ trong `$XDG_STATE_HOME/argvus/welcome/progress.json` (hoặc `~/.local/state/argvus/welcome/progress.json`).Sau khi máy tính để bàn sẵn sàng, phiên chạy `argvus welcome --if-needed` cho TTY, ARGVUS Greeter và các trình quản lý đăng nhập khác. Nó mở Chào mừng trừ khi tiến trình của mỗi người dùng ghi lại `opened`, hoàn thành hoặc loại bỏ. Kết xuất thành công đầu tiên sẽ lưu `opened` trong `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`, ngăn việc mở lại vào những lần đăng nhập sau. Nó không phụ thuộc vào `/etc/skel`. Các tài khoản hiện tại không có hồ sơ đó cũng nhận được một lần mở tự động. Lỗi trước khi kết xuất cho phép thử lại ở lần đăng nhập tiếp theo. Hướng dẫn sử dụng `argvus welcome` vẫn có sẵn.

Các trang nhà cung cấp chính xác có thể phụ thuộc vào các gói và quyền đã cài đặt. Đừng cho rằng miền tùy chọn bị thiếu là Trung tâm điều khiển bị hỏng.

Khi lưới có nhiều mục hơn mức vừa với màn hình, một thanh cuộn sẽ xuất hiện dọc theo cạnh phải để hiển thị có nhiều mục bên dưới hơn.

## Tìm cài đặt

Màn hình chính đăng ký cài đặt theo tiêu đề, danh mục, mã định danh và từ khóa. Tìm kiếm không phân biệt trọng âm và chấp nhận các thuật ngữ thực tế như `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` và `DPI`. Tiêu đề trùng khớp chính xác được xếp hạng đầu tiên, theo sau là tiền tố tiêu đề, bất kỳ từ nào trong tiêu đề hoặc danh mục có chứa văn bản đã nhập và cuối cùng là từ khóa khớp — vì vậy, việc nhập bất kỳ phần nào của từ (không chỉ phần đầu của từ đó) vẫn có thể tìm thấy đích.

Menu hiển thị nhận biết khả năng. Một tuyến có thể vắng mặt khi nhà cung cấp nó chưa được cài đặt, khi phần cứng không có sẵn hoặc khi phiên hiện tại không thể cung cấp khả năng cần thiết. Do đó, tìm kiếm cũng là một cách hữu ích để khám phá cài đặt này có thể cấu hình những gì.

Cây cài đặt chính hiện bao gồm:

| Khu vực | Điểm đến hiện tại |
| --- | --- |
| Ngoại hình | Kiểu máy tính để bàn: chủ đề, chế độ giao diện, màu nhấn, hình nền, biểu tượng và phông chữ; Kiểu ứng dụng: tính minh bạch của thiết bị đầu cuối và trình khởi chạy; Thanh tác vụ; Bảng điều khiển; Tiện ích từ xa |
| Máy tính để bàn & Đầu vào | Hyprland (Windows: Quy tắc, Dấu cách và Chế độ, hình động, làm mờ, đường viền và bóng), Màn hình, Bàn phím & Phím tắt, Chuột & Bàn di chuột |
| Sở thích | Ngôn ngữ & Khu vực (ngôn ngữ, múi giờ, định dạng, ngôn ngữ hệ thống và bố cục bàn phím), Ngày & Giờ, Ứng dụng mặc định, Nguồn, Phiên |
| Phần cứng & Kết nối | Tổng quan về phần cứng (tóm tắt, CPU, GPU, bộ nhớ, thiết bị), Mạng & Wi-Fi, Bluetooth, Âm thanh, Bộ lưu trữ |
| Quản trị hệ thống | Gói, Dịch vụ, Khởi động, Tên máy chủ, Người dùng & Nhóm, Tường lửa, Thông báo |
| Cài đặt | Ngôn ngữ (ngôn ngữ người dùng), Phong cách (biểu tượng và độ trong suốt) |
| Công cụ & Chẩn đoán | Dự án, Đoạn mã, Khóa SSH, Khóa GPG, Dịch vụ dành cho nhà phát triển, Bác sĩ hệ thống, Giới thiệu về ARGVUS |

Một số trong số này là các trang đầy đủ và một số là các trang con được truy cập thông qua tìm kiếm hoặc tóm tắt tên miền. Danh sách chính xác được xác định bởi các nhà cung cấp ARGVUS đã cài đặt.

Mỗi trang hiển thị đường dẫn của nó trong tiêu đề, ví dụ `Control Center > Hardware & Connectivity > Audio`. Đường dẫn bắt đầu bằng thẻ Trang chủ liệt kê trang, do đó, cùng một trang sẽ đọc giống nhau từ Trang chủ, từ tìm kiếm và từ CLI. Tên gốc và tên thẻ được dịch: giao diện tiếng Bồ Đào Nha hiển thị `Central de Controle > Hardware e conectividade > Áudio`.

## Công dụng của từng khu vực

### Ngoại hìnhNgoại hình là khu vực tùy chỉnh chính. Nhóm Desktop của nó giữ chủ đề (với chế độ Sticky hoặc Spaced), chế độ xuất hiện, màu nhấn, hình nền, chủ đề biểu tượng và trang Phông chữ, còn nhóm Kiểu ứng dụng của nó giữ độ trong suốt của thiết bị đầu cuối và trình khởi chạy. Thanh tác vụ, Bảng điều khiển và Đo từ xa tiện ích được liệt kê ngay trong Giao diện trên trang chủ và sử dụng lại trang trong suốt được chia sẻ của Giao diện cho các bề mặt của chính chúng. Blur là một cài đặt chung, trong Hyprland > Blur và áp dụng cho mọi bề mặt ARGVUS đã bật tính năng làm mờ. Trang đó chỉnh sửa công tắc Bật và các giá trị làm mờ Hyprland (Kích thước, Tốc độ, Độ sáng, Độ nhiễu, Độ tương phản, Độ rung và độ tối Độ rung) dưới dạng bản nháp và không có gì thay đổi cho đến khi **Áp dụng** được chọn. Hoạt ảnh hoạt động theo cách tương tự trong Hyprland > Hoạt ảnh: nút Bật, các đường cong Tùy chỉnh (Bézier, Spring, Linear và Ease-out) và năm danh mục hoạt ảnh tạo thành một bản nháp được áp dụng cùng với **Áp dụng**. Bézier và Spring có các tham số có thể chỉnh sửa được (điểm Bézier và khối lượng lò xo, độ cứng và độ giảm chấn); Linear và Ease-out là những đường cong cố định nên trang của chúng chỉ liệt kê các điểm kiểm soát và không thể chỉnh sửa. Các lựa chọn kiểu dáng tuân theo những gì Hyprland 0.56.2 đưa ra: Mở và Đóng cung cấp slide, popin và gnomed, lối vào và thoát lớp cung cấp slide, popin và fade, và các không gian làm việc để lại cung cấp slide, fade, slidefade, slidevert và slidefadevert. Các điều khiển này phối hợp với các thành phần giao diện và phiên thay vì chỉ thay đổi cửa sổ Trung tâm điều khiển.

### Thanh tác vụ

Thanh tác vụ định cấu hình vị trí của bảng điều khiển, khoảng cách xung quanh nó và các cửa sổ, chiều cao và đường viền của nó, nhóm biểu tượng tiện ích và các khối ngày/giờ, cùng với độ trong suốt của chính nó. Nó được liệt kê trong phần **Hình thức** ở nhà. Ở chế độ Chú ý, **Dấu cách > Dưới** mặc định là `2`; Spaced giữ đường cơ sở `18`-pixel.

**Chiều cao** chọn 24, 28 (mặc định), 32 hoặc 36 pixel. **Biên giới** có công tắc **Bật** và **Độ dày** từ 1 đến 3 pixel, với 1 làm mặc định. Khi tắt viền, cửa sổ thanh tác vụ, nhóm bên phải và trình phát MPRIS không có viền. Những thay đổi này là bản nháp cho đến khi **Áp dụng** được chọn. Chiều cao và đường viền được áp dụng sẽ được giữ nguyên khi chủ đề thay đổi; cho đến khi chúng được áp dụng, chủ đề sẽ quyết định chúng.

Trang **Biểu tượng** mở ra **Các mô-đun tùy chọn**, **Bản trình bày không gian làm việc**, **Trình khởi chạy** và **Tiện ích**. Các mô-đun tùy chọn có thể được hiển thị hoặc ẩn riêng lẻ: trình phát âm thanh, Tiêu điểm, trình khởi chạy, mạng, cấu hình nguồn, bố trí bàn phím, bộ nhớ, CPU và nhiệt độ, ngày, đồng hồ, pin, khay hệ thống, thông báo, thiết bị di động, ghi âm, Bluetooth, đầu vào/đầu ra âm thanh và luôn thức. Không gian làm việc vẫn khả dụng và có thể hiển thị tên của chúng (mặc định) hoặc biểu tượng ứng dụng. Tiêu đề cửa sổ tập trung, Cài đặt và tác vụ Nguồn luôn có sẵn. Khả năng hiển thị của trình khởi chạy được kiểm soát trong các mô-đun Tùy chọn; menu con của nó thay đổi biểu tượng tùy chỉnh. Các thay đổi có hiệu lực sau khi **Áp dụng**, thao tác này sẽ khởi động lại thanh tác vụ.

### Bảng điều khiển

Bảng điều khiển định cấu hình mức độ hiển thị và thứ tự của thẻ hành động nhanh, cùng với các phiên và tính minh bạch của chính nó.

Trang sẽ mở ra và chấp nhận dữ liệu nhập ngay khi đọc cài đặt thẻ. Các thẻ phụ thuộc vào phần cứng (Bluetooth và Độ sáng) được kiểm tra ở chế độ nền: cho đến khi quá trình kiểm tra đó kết thúc, chúng vẫn duy trì tính khả dụng được nhìn thấy lần cuối trong cửa sổ này và danh sách sẽ tự cập nhật khi có kết quả.

### Widget Đo từ xa

Widget Telemetry định cấu hình các phiên và tính minh bạch của tiện ích thông tin hệ thống.

### Phông chữPhông chữ được lấy từ **Giao diện > Kiểu màn hình > Phông chữ**. Nó cho phép bạn chọn dòng và kích thước cho thanh tác vụ, thông tin đo từ xa/hệ thống, Bảng điều khiển, giao diện hệ thống ARGVUS, ứng dụng, thiết bị đầu cuối và trình duyệt. Nó cũng hiển thị các cài đặt kết xuất phông chữ như khử răng cưa, gợi ý, chế độ pixel phụ và HDR. Họ chính mặc định là IBM Plex Mono; các mục tiêu riêng lẻ có thể được thay đổi và khôi phục độc lập từ trang Phông chữ.

### Ứng dụng mặc định

Các ứng dụng mặc định chọn các chương trình đã cài đặt cho các vai trò như thiết bị đầu cuối, trình quản lý tệp, trình soạn thảo văn bản, trình chỉnh sửa thiết bị đầu cuối, trình duyệt, trình xem hình ảnh, trình xem PDF, trình phát video, trình phát âm thanh, công cụ lưu trữ và trình khởi chạy. Lựa chọn được ARGVUS lưu và có thể cập nhật các liên kết XDG tiêu chuẩn. Trình quản lý tệp TUI sử dụng trình chỉnh sửa thiết bị đầu cuối đã chọn khi mở tệp văn bản; trình soạn thảo văn bản đồ họa vẫn là mặc định cho các liên kết tệp trên máy tính để bàn.

### Đoạn trích

Đoạn mã là trang dành cho văn bản mà `argvus snippets` gõ bằng `SUPER + ALT + N`. Nó nằm trong **Công cụ & Chẩn đoán → Đoạn trích** (tìm kiếm `snippets` hoặc chạy `argvus control-center snippets`).

- **Đoạn mã mới** — nhập **Tên** và **Nội dung**, sau đó **Lưu đoạn mã**. Tên đã tồn tại sẽ thay thế nội dung của đoạn mã đó.
- **Đoạn đã lưu** — được liệt kê cùng với vị trí của chúng (`1  email`) và nội dung trên một dòng. Nhập vào nội dung chỉnh sửa nó. **Type** nhập đoạn mã và **Xóa** yêu cầu xác nhận.
- **Mở bộ chọn** — mở bộ chọn tương tự như `SUPER + ALT + N`.

Quá trình nhập bắt đầu 3 giây sau khi bạn nhấn Enter, vì chính Trung tâm điều khiển có tiêu điểm tại thời điểm đó. Nhấp vào trường sẽ nhận được văn bản trong thời gian đó. Trang chỉ chỉnh sửa nội dung một dòng; một đoạn mã nhiều dòng có thể được tạo trong một thiết bị đầu cuối với `argvus snippets add`.

Trang đọc các mục từ `argvus-config` và ghi chúng thông qua `argvus snippets`, do đó trình khởi chạy sẽ tiếp tục xác thực. Khi `argvus snippets` chưa được cài đặt, trang sẽ thông báo như vậy.

### Khóa SSH

Khóa SSH là trang dành cho các cặp khóa mà `argvus keys` quản lý. Nó nằm trong **Công cụ & Chẩn đoán → Khóa SSH** (tìm kiếm `ssh keys` hoặc chạy `argvus control-center ssh-keys`).

Trang này có bốn phần, từ trên xuống dưới:

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```

####Cách sử dụng

1. Mở **Trung tâm điều khiển → Công cụ & Chẩn đoán → Khóa SSH**. Từ Trang chủ, bạn cũng có thể tìm kiếm `ssh keys`.
2. **Tìm khóa:** nhấn `/`, sau đó nhập để chỉ hiển thị các cặp có tên hoặc nhận xét trùng khớp. `Enter` kết thúc tìm kiếm và chạy hàng được đánh dấu, như trong các danh sách khác, vì vậy hãy kiểm tra hàng trước khi nhấn nó; `Esc` xóa tìm kiếm.
3. **Tạo một cặp:** trong **Generate**, chọn **Name**, nhấn `Enter`, nhập tên và nhấn lại `Enter`. Chọn **Loại** và nhấn `Enter` để chuyển đổi giữa `ed25519` và `rsa`. Chọn **Tạo cặp khóa** và nhấn `Enter`. Cặp mới xuất hiện ngay bên dưới **Phím** mà không có cụm mật khẩu.
4. **Sao chép khóa chung:** trong khối của cặp, chọn **Sao chép khóa chung** và nhấn `Enter`. Nội dung của tệp `.pub` sẽ chuyển đến bảng tạm Wayland, sẵn sàng dán vào `authorized_keys` của máy chủ hoặc máy chủ Git. Khóa riêng không bao giờ được sao chép. Điều này cần `wl-clipboard` (`wl-copy`); không có nó, trang này cho biết khóa không được sao chép.
5. **Nạp một cặp vào tác nhân:** trong khối của cặp, chọn **Nạp vào tác nhân** và nhấn `Enter`. Khóa không có cụm mật khẩu sẽ được tải cùng lúc mà không có cửa sổ. Khóa có cụm mật khẩu sẽ mở thiết bị đầu cuối ARGVUS, nơi OpenSSH yêu cầu; đóng thiết bị đầu cuối bằng `Enter` khi nó thông báo khóa đã được tải. Trang sẽ kiểm tra tác nhân vài giây một lần khi nó mở, do đó, hàng sẽ tự thay đổi thành **Dỡ bỏ khỏi tác nhân**. Để cập nhật ngay lập tức, hãy sử dụng **Làm mới** trong **Hành động**.6. **Xóa key khỏi tác nhân:** trong khối của cặp chọn **Dỡ khỏi tác nhân**. Để xóa mọi khóa, hãy sử dụng **Dỡ bỏ tất cả các khóa khỏi tác nhân** trong **Hành động**. Các tập tin quan trọng vẫn còn trên đĩa.
7. **Xóa một cặp:** trong khối của cặp, chọn **Xóa** và nhấn `Enter`. Xác nhận tập trung vào **Hủy**, vì vậy `Enter` đi lạc sẽ không làm gì cả. Nhấn `y` để xác nhận hoặc `n`/`Esc` để hủy. Không thể xóa khóa được tải trong tác nhân; dỡ nó trước.

Di chuyển xung quanh: `↑`/`↓` hoặc `j`/`k` di chuyển qua danh sách, `Enter` kích hoạt một hàng, `Esc` quay lại và `?` hiển thị trợ giúp. `Tab` chuyển tiêu điểm vào **Hành động**, trong đó `←`/`→` chọn một nút và `Enter` chạy nút đó. `↑`/`↓` đưa trọng tâm trở lại danh sách.

Cụm mật khẩu không bao giờ được nhập vào Trung tâm điều khiển và không bao giờ được chuyển dưới dạng đối số lệnh. Trang đọc danh sách với `argvus keys list` và chỉ ghi thông qua `argvus keys`. Khi `argvus-keys` chưa được cài đặt, trang sẽ thông báo như vậy.

#### Khắc phục sự cố

| Triệu chứng | Nguyên nhân và cách khắc phục |
| --- | --- |
| `Could not open a connection to your authentication agent` sau **Tải vào đại lý** | Thiết bị đầu cuối không nhận được ổ cắm đại lý. Hyprland giữ nguyên môi trường như ban đầu và tác nhân có thể đã bắt đầu muộn hơn. Cập nhật `argvus-launcher`, quay trở lại `$XDG_RUNTIME_DIR/ssh-agent.socket` và kiểm tra `systemctl --user status argvus-ssh-agent.service`. |
| **Đại lý** hiển thị `Agent unavailable` | Trình khởi chạy không thể tiếp cận được đại lý. Kiểm tra dịch vụ như trên. |
| Trang này nói `argvus-keys is not installed` | Cập nhật gói `argvus`: `argvus keys` là một lệnh con của nó. |
| **Xóa** hiển thị lỗi | Chìa khóa được tải vào tác nhân và nó được giữ lại. Trước tiên hãy dỡ nó ra, sau đó xóa lại. |

### Khóa GPG

Khóa GPG là trang dành cho khóa mà `argvus gpg` quản lý. Nó nằm trong **Công cụ & Chẩn đoán → Khóa GPG** (tìm kiếm `gpg keys` hoặc chạy `argvus control-center gpg-keys`).

Trang này có các phần sau, từ trên xuống dưới:

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```

`▾` đánh dấu phần được mở rộng và `▸` là phần được thu gọn. `Enter` hoặc `Space` mở rộng hoặc thu gọn nó.

####Cách sử dụng

1. Mở **Trung tâm điều khiển → Công cụ & Chẩn đoán → Khóa GPG**. Từ Trang chủ, bạn cũng có thể tìm kiếm `gpg keys`.
2. **Pinentry** được cố định thành `pinentry-gtk`. Trang này chỉ hiển thị nó. Lần đầu tiên khóa được tạo, ARGVUS ghi mã pin đó vào `gpg-agent.conf`, giữa các dòng đánh dấu của chính nó và giữ các dòng khác của bạn. Nếu bạn đã viết một dòng `pinentry-program` bên ngoài các điểm đánh dấu đó, quá trình tạo sẽ dừng lại với một lỗi để bạn có thể xóa dòng đó.
3. **Tạo khóa:** trong **Tạo khóa ▾**, nhập **Họ tên** và **E-mail**. Cả hai đều được yêu cầu và **Tạo khóa** vẫn bị tắt cho đến khi cả hai đều có văn bản. **Bình luận** là tùy chọn. ID người dùng trở thành `Name (comment) <e-mail>`.
   - **Thuật toán** chuyển đổi giữa **RSA** và **DSA ElGamal**. Khóa DSA nhận được khóa con mã hóa ElGamal; khóa RSA sẽ nhận được một khóa RSA.
   - **Độ mạnh khóa** xoay quanh các kích thước của thuật toán: RSA 1024, 2048 và 4096; DSA 1024, 2048 và 3072. gpg không tạo khóa DSA trên 3072 bit, vì vậy 4096 không được cung cấp cho DSA.
   - **Ngày hết hạn** mở trang riêng. Nhập ngày vào bố cục ngôn ngữ của bạn: `DD/MM/YYYY` bằng tiếng Bồ Đào Nha, tiếng Tây Ban Nha, tiếng Pháp, tiếng Ý, tiếng Đức và tiếng Nga; `YYYY/MM/DD` bằng tiếng Anh, tiếng Trung, tiếng Nhật và tiếng Hàn. Ngày phải sau ngày hôm nay. Bật **Không bao giờ hết hạn** để đặt khóa vĩnh viễn; trường ngày sau đó bị vô hiệu hóa và không được sử dụng.- **Cụm mật khẩu** mở trang riêng của nó. **Bảo vệ bằng cụm mật khẩu** được bật theo mặc định; nhập **Cụm mật khẩu** và sau đó **Xác nhận cụm mật khẩu**. Cả hai phải phù hợp. Tắt bảo vệ sẽ xóa cả hai trường và trang yêu cầu xác nhận trước khi tạo khóa không được bảo vệ.
4. Chọn **Tạo khóa** và nhấn `Enter`. Khóa nhận được khóa ký và khóa con mã hóa. Cụm mật khẩu được gửi tới trình khởi chạy trên đầu vào tiêu chuẩn của nó, không bao giờ dưới dạng đối số lệnh. Nếu quá trình tạo không thành công, các trường khác vẫn giữ nguyên nhưng cụm mật khẩu sẽ bị xóa và phải nhập lại.
5. **Hủy** xóa mọi trường của biểu mẫu và trả ngày và kích thước về mặc định.
6. **Nhập:** trong **Nhập ▾**, chọn **Nhập khóa chung…** hoặc **Nhập khóa riêng…**. Trình quản lý tệp mở ra để bạn có thể chọn tệp. Khóa riêng sẽ yêu cầu xác nhận trước và cụm mật khẩu của nó sẽ được gpg hỏi khi khóa được sử dụng. Nếu trình quản lý tệp là trình quản lý đồ họa, nó không thể báo cáo tệp lên trang; sau đó trang sẽ mở ra một trường nơi bạn nhập đường dẫn.
7. **Phím >>** mở móc khóa. Mỗi khóa hiển thị danh tính, dấu vân tay, loại, thời hạn sử dụng và liệu khóa riêng của nó có nằm trong khóa hay không. Mỗi phím có những hành động sau:
   - **Chỉnh sửa khóa >>** mở trang riêng của khóa (mô tả bên dưới).
   - **Xuất khóa công khai…** yêu cầu đường dẫn tệp, được đề xuất là `~/<fingerprint>.asc`. Tệp được tạo với chế độ `600`. Một tập tin hiện có không bao giờ được thay thế.
   - **Xuất khóa riêng tư…** chỉ xuất hiện đối với các khóa có phần riêng tư. Nó yêu cầu xác nhận, sau đó yêu cầu đường dẫn tệp, được đề xuất là `~/<fingerprint>-private.asc`. Tệp được tạo với chế độ `600`.
   - **Sao chép khóa chung** và **Sao chép dấu vân tay** sao chép vào bảng tạm Wayland bằng `wl-copy`.
   - **Phím xóa…** yêu cầu xác nhận, với tiêu điểm là Hủy. Nó loại bỏ chìa khóa và phần riêng tư của nó.
8. **Tìm khóa:** trên trang **Phím**, thanh tìm kiếm ở trên cùng sẽ lọc các khóa. Gõ để thu hẹp danh sách; tìm kiếm khớp với danh tính (bao gồm nhận xét) và dấu vân tay. `Esc` xóa tìm kiếm.
9. **Chỉnh sửa khóa >>** hiển thị thông tin chi tiết về khóa, sau đó thực hiện các thao tác sau:
   - **Ngày hết hạn >>** thay đổi thời hạn sử dụng của khóa và của từng khóa con của nó. Sử dụng **Không bao giờ hết hạn** để loại bỏ thời hạn sử dụng. **Áp dụng** lưu thay đổi.
   - **Thay đổi cụm mật khẩu** hỏi cụm mật khẩu cũ và mới trong cửa sổ pinentry.
   - **Ký tên với khóa >>** xác nhận khóa này bằng một trong các khóa khác của bạn có phần riêng tư. Cụm mật khẩu của người ký được hỏi. Trang yêu cầu xác nhận trước.
   - **Thu hồi key >>** liệt kê các lý do: không có lý do, key bị xâm phạm, key bị thay thế, key không còn được sử dụng. Mỗi người yêu cầu xác nhận. Khóa bị thu hồi không thể sử dụng được nữa.
   - **Tin cậy** đặt mức độ tin cậy của bạn đối với khóa: không được đặt, cận biên, đầy đủ hoặc tối thượng.
   - **Xuất bản lên máy chủ khóa…** gửi khóa chung tới máy chủ khóa được đặt trong `gpg.conf`, sau khi xác nhận. **Tìm nạp từ máy chủ khóa** tải xuống khóa từ máy chủ khóa đó.

Di chuyển xung quanh: `↑`/`↓` hoặc `j`/`k` di chuyển qua danh sách, `Enter` kích hoạt một hàng, `Esc` quay lại và `?` hiển thị trợ giúp.

Cụm mật khẩu chỉ được nhập vào các trường ẩn của trang cụm mật khẩu hoặc vào cửa sổ ghim. Trang gửi cụm mật khẩu tạo tới `argvus gpg` trên đầu vào tiêu chuẩn và trình khởi chạy chuyển cụm mật khẩu đó tới gpg trên bộ mô tả tệp, do đó, nó không bao giờ xuất hiện trong danh sách quy trình, trong đối số lệnh hoặc trong nhật ký. Những thay đổi chính chạy ở chế độ nền; trong khi một người chạy, những người khác chờ đợi. Trang đọc chuỗi khóa bằng `argvus gpg list` và chỉ ghi thông qua `argvus gpg`. Khi `argvus gpg` chưa được cài đặt, trang sẽ thông báo như vậy.

Chưa có trong trang: tìm kiếm khóa trên máy chủ khóa theo tên hoặc e-mail. Sử dụng **Tìm nạp từ máy chủ khóa** bằng dấu vân tay đã biết.#### Khắc phục sự cố

| Triệu chứng | Nguyên nhân và cách khắc phục |
| --- | --- |
| Trang này nói `argvus gpg is not available` | `argvus-launcher` được cài đặt trước lệnh GPG. Chạy `argvus gpg list` trong thiết bị đầu cuối: `unknown command` có nghĩa là gói phải được xây dựng lại và cài đặt. Chìa khóa cũng cần có `gnupg`. |
| **Tạo khóa** vẫn bị tắt | Nhập cả **Tên đầy đủ** và **E-mail**. |
| Một dòng trạng thái cho biết `argvus-gpg: pinentry-gtk is not installed` | Cài đặt gói `pinentry`, gói này cung cấp `pinentry-gtk`. |
| Thế hệ dừng lại với lỗi `pinentry-program` | `gpg-agent.conf` đặt pinentry bên ngoài khối ARGVUS. Xóa dòng đó; ARGVUS sau đó tự viết. |
| **Pinentry** hiển thị `Not set` | gpg-agent sử dụng mặc định tích hợp sẵn cho đến khi khóa đầu tiên được tạo từ trang. |
| Trang cho biết `argvus-launcher` đã cài đặt cũ hơn Trung tâm điều khiển | Cài đặt lại `argvus-launcher`: chạy `make build` rồi chạy `make install` trong kho lưu trữ của nó và mở lại trang. Các hành động chính sẽ tắt cho đến khi các phiên bản khớp nhau. |
| Trình quản lý tệp mở ra nhưng trang yêu cầu đường dẫn | Trình quản lý tệp là trình quản lý đồ họa và không thể báo cáo tệp đã chọn. Nhập đường dẫn vào trường. |
| **Sao chép khóa công khai** báo không thể sao chép | `wl-copy` chưa được cài đặt hoặc phiên Wayland không khả dụng. Cài đặt `wl-clipboard`. |
| **Chỉnh sửa khóa >>** cho biết khóa không còn trong ổ khóa | Móc khóa đã thay đổi kể từ khi trang được mở. Quay lại **Phím>>** và làm mới. |
| Một dòng trạng thái hiển thị `argvus-gpg: ...` | Trình khởi chạy đã từ chối hành động và văn bản đưa ra lý do, chẳng hạn như thời hạn sử dụng không hợp lệ, tệp đã tồn tại hoặc `--yes` bị thiếu. |
| Một hành động chờ đợi và không có gì xảy ra | Cửa sổ pinentry bị ẩn hoặc không được mở. Kiểm tra phiên cho thấy `WAYLAND_DISPLAY` hoặc `DISPLAY`. |
| Việc nhập khẩu bị từ chối | Tệp không chứa khóa thuộc loại được chọn. Khóa công khai được thực hiện **Nhập khóa công khai…**; khóa riêng thông qua **Nhập khóa riêng…**. |

### Ngôn ngữ & Khu vực

Thẻ này liệt kê Ngôn ngữ (ngôn ngữ khu vực), Múi giờ, Ngày & Giờ, Ngôn ngữ hệ thống và Bố cục bàn phím và mỗi thẻ đều mở trực tiếp từ nhà. Các trang bàn phím cũng chứa sơ đồ bàn phím của biến thể và bảng điều khiển. Tìm kiếm tại nhà cũng hiển thị các phím tắt và cài đặt chuột/bàn di chuột dưới dạng các tuyến cài đặt trực tiếp.

Danh sách ngôn ngữ khám phá các danh mục đã cài đặt và hiển thị tên gốc của tất cả mười ngôn ngữ. Kích hoạt ngôn ngữ hệ thống UTF-8 tương ứng trước khi chọn nó.

Cả **Tùy chọn > Ngôn ngữ & Khu vực > Ngôn ngữ** và **Cài đặt > Ngôn ngữ** đều lưu cùng một tùy chọn người dùng: `LANG` trong `$XDG_CONFIG_HOME/locale.conf` (`~/.config/locale.conf` mặc định), ví dụ `LANG=en_US.UTF-8`. Nhận xét và các biến khu vực được bảo tồn. Không có tệp chung hoặc cài đặt ngôn ngữ ARGVUS song song nào được ghi. Người dùng chưa được định cấu hình sẽ nhận được tiếng Anh; Các tệp `session.language` và `data/language` kế thừa bị bỏ qua. Đăng xuất và đăng nhập lại để áp dụng ngôn ngữ cho toàn bộ phiên và ứng dụng.

**Vị trí hệ thống** thay đổi `/etc/locale.gen` và chạy `locale-gen`, vì vậy việc áp dụng nó cần có sự cho phép của quản trị viên. Thông báo **[WARN]** liên tục phía trên chân trang xác định các trang có cài đặt quản trị. Thông báo tương tự bao gồm múi giờ, ngày/giờ, tên máy chủ, sơ đồ bàn phím bảng điều khiển, chỉnh sửa tài khoản và nhóm, tường lửa, dịch vụ hệ thống, bộ điều chỉnh CPU, cấu hình và chính sách nguồn, cấu hình khởi động, hoạt động gói, ảnh chụp nhanh và các thay đổi mạng được bảo vệ. Tùy chọn cá nhân, dịch vụ người dùng và các trang chỉ đọc không nhận được thông báo này. Thông báo mang tính thông tin; ủy quyền chỉ được yêu cầu khi một hành động cần nó.

### Phần cứng và màn hìnhKhi các khả năng tương ứng được cài đặt, Trung tâm điều khiển có thể hiển thị thông tin phần cứng như CPU, GPU, bộ nhớ, nguồn và thiết bị. Trang đầu vào định cấu hình hoạt động của chuột và bàn di chuột; khu vực hiển thị xử lý độ phân giải màn hình, tốc độ làm mới, tỷ lệ, vị trí, hướng, màn hình chính, VRR và HDR khi được phiên chạy hỗ trợ. Các thay đổi về hiển thị sẽ được áp dụng ngay lập tức. Sau một thay đổi có thể khiến màn hình không sử dụng được (độ phân giải hoặc tốc độ làm mới, vị trí, phản chiếu, tắt màn hình, độ sâu màu) hoặc sau **Áp dụng**, Trung tâm điều khiển sẽ hỏi xem có nên giữ thay đổi đó hay không, với tiêu điểm là **Hoàn nguyên**: nhấn `y` để giữ thay đổi đó; `n`, `Esc` hoặc `Enter` hoàn nguyên cấu hình đó và nếu không có câu trả lời trong vòng 15 giây, cấu hình trước đó sẽ tự động quay lại. Xóa cấu hình đã lưu của màn hình bị ngắt kết nối và xóa hồ sơ yêu cầu xác nhận; hồ sơ mở trang riêng của họ bằng **Áp dụng hồ sơ**, **Đổi tên** và **Xóa hồ sơ**. Bố cục được lưu thông qua `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` và `monitor-remove`, vì vậy phần `/displays` chuẩn của `config.json` là nguồn duy nhất của bố cục màn hình. Chỉ danh mục hồ sơ vẫn ở `display/config.toml`.

### Khả năng kết nối và âm thanh

Khu vực mạng hiển thị các trang kết nối và trạng thái của nhà cung cấp mạng đã cài đặt, bao gồm Wi-Fi, Ethernet, VPN, DNS và các tuyến proxy khi khả dụng. Tường lửa được liệt kê trong **Quản trị hệ thống**. Bluetooth có điều kiện hỗ trợ Bluetooth. Vùng âm thanh được cung cấp khi khả năng tùy chọn của nó được bật và tích hợp với dịch vụ âm thanh đã cài đặt.

Trang chủ **Mạng & Wi-Fi** và trang **Trạng thái** của nó điền vào khi mỗi truy vấn NetworkManager trả lời, thay vì sau câu hỏi chậm nhất: trước tiên là danh sách thiết bị, sau đó là radio, địa chỉ và DNS của giao diện được kết nối, các kết nối đã lưu và mạng Wi-Fi mà NetworkManager đã biết (quét mới chỉ chạy từ trang Wi-Fi có `r`). Cho đến khi danh sách thiết bị xuất hiện, các hàng có nội dung *Đang tải...* và công tắc Wi-Fi chỉ được cung cấp khi máy tính có thẻ Wi-Fi. Bắt đầu một hành động (chuyển đổi Wi-Fi, kết nối, DNS) trong khi một trang đang làm mới sẽ thay thế việc làm mới và trang sẽ làm mới lại khi hành động kết thúc.

#### Kết nối, lấy cảm hứng từ nmtui

**Mạng & Wi-Fi > Kết nối** liệt kê mọi cấu hình kết nối NetworkManager, được nhóm theo loại, với thiết bị mà nó đang hoạt động trên đó (`★ Active · eno1`), cho dù đó là cổng của bridge, bond hay nhóm và liệu đó có phải là chỉ đọc hay không. `Enter` mở hồ sơ trong trình chỉnh sửa, `a` kết nối hoặc ngắt kết nối nó, `/` lọc theo tên và `r` làm mới. **Kích hoạt kết nối** là cùng một danh sách trong đó `Enter` đưa cấu hình lên hoặc xuống, như trong `nmtui`. Các trang chi tiết của một giao diện, mạng Wi-Fi đã lưu và VPN có hàng **Chỉnh sửa kết nối** để mở hồ sơ của chúng.

**Thêm kết nối** tạo cấu hình Ethernet, Wi-Fi, Hotspot, DSL (PPPoE), Bridge, Bond, Team, VLAN, đường hầm IP, Loopback và WireGuard và một cấu hình VPN cho mỗi plugin NetworkManager VPN đã cài đặt (OpenVPN có biểu mẫu riêng; các plugin khác chỉnh sửa cài đặt của chúng dưới dạng mục nhập `key=value`). Nó cũng nhập WireGuard `.conf` hoặc tệp plugin như `.ovpn` (`nmcli connection import`). Nhóm cần gói `libteam`, DSL cần `ppp` và VPN cần plugin của nó (ví dụ `networkmanager-openvpn`); không có chúng, hàng sẽ bị tắt và cho biết cần cài đặt gì. WEP, InfiniBand, MACsec và Open vSwitch không được cung cấp.Trình chỉnh sửa là một danh sách gồm các phần có thể gập lại: **Chung** (tên, giao diện, kết nối tự động, mức độ ưu tiên, khả dụng cho tất cả người dùng, theo đồng hồ đo), cài đặt riêng của loại, **IPv4** và **IPv6** (phương thức, địa chỉ, cổng, máy chủ DNS, miền tìm kiếm, tuyến đường, bỏ qua DNS hoặc tuyến đường tự động, không bao giờ sử dụng làm tuyến đường mặc định, số liệu tuyến đường, tiện ích mở rộng quyền riêng tư, tạo địa chỉ). Các lựa chọn và số thay đổi với `←/→`; `Enter` gõ một giá trị; danh sách (địa chỉ, DNS, tuyến đường, IP được phép) mở trang riêng của chúng, trong đó `Enter` chỉnh sửa một mục nhập, xóa mục đó sẽ xóa mục đó và **Thêm mục** thêm một mục nhập. Các trường không áp dụng cho phương pháp đã chọn sẽ bị vô hiệu hóa. Một cầu nối, trái phiếu hoặc nhóm chọn Ethernet **Cổng** của mình trên một trang phụ; WireGuard chỉnh sửa **Các đối tác ngang hàng** của nó trên một trang phụ và có thể **Tạo cặp khóa** (với `wg`, từ `wireguard-tools`, một phần phụ thuộc của gói `argvus`); khóa chung của khóa riêng được hiển thị bên dưới nó.

Không có gì đến NetworkManager cho đến khi **Save**, tính năng này vẫn bị tắt khi không có gì thay đổi hoặc khi một trường không hợp lệ, sau đó cho biết trường nào và lý do. Chỉ những cài đặt đã thay đổi mới được ghi. Việc lưu kết nối đang hoạt động sẽ yêu cầu trước tiên rồi kết nối lại kết nối đó, do đó, cài đặt mới sẽ được áp dụng ngay lập tức (điều này có thể làm gián đoạn mạng trong thời gian ngắn). `Esc` với những thay đổi chưa được lưu sẽ hỏi trước khi loại bỏ chúng. **Xóa kết nối**, trong vùng Nguy hiểm, yêu cầu xác nhận với tiêu điểm là Hủy; xóa một cây cầu, trái phiếu hoặc nhóm cũng xóa các cổng của nó.

Mật khẩu, khóa riêng tư và khóa chia sẻ trước không bao giờ được đọc lại: các hàng của chúng ghi *Không thay đổi* cho đến khi một giá trị mới được nhập và giá trị mới được hiển thị bị che đi. Chúng được viết thông qua đầu vào tiêu chuẩn của `nmcli connection edit`, không bao giờ dưới dạng đối số lệnh. Các thay đổi được thực hiện với tư cách là người dùng phiên: các thành viên cục bộ của `wheel` được cho phép theo quy tắc polkit của NetworkManager và những người dùng khác được tác nhân polkit của phiên yêu cầu nhập mật khẩu quản trị viên.

### Nguồn và phiên

Power cung cấp các điều khiển nguồn hệ thống và các trang chính sách có sẵn cho quá trình cài đặt, bao gồm cả nút chuyển đổi Keep Awake. Khi được bật, Keep Awake sẽ ngăn chính sách nhàn rỗi của ARGVUS khởi động bộ hẹn giờ khóa màn hình và nguồn hiển thị tự động. Phiên hiển thị trạng thái phiên ARGVUS, các thành phần, tự động khởi động, chẩn đoán và nhật ký. Trên **Nguồn**, thanh **Hành động** trong danh sách giữ **Tạm dừng**, **Ngủ đông** và **Làm mới** (`r` cũng làm mới): `Tab` hoặc `←/→` di chuyển đến đó, `Enter` chạy nút lấy nét, `↑`/`↓` đưa tiêu điểm trở lại danh sách và `Esc` quay trở lại. Đình chỉ và Hibernate yêu cầu xác nhận trước. Các trang này có thể yêu cầu quyền của hệ thống và phải được phân biệt với các hành động nhanh trong Bảng điều khiển.

### Công cụ hệ thống

Phần hệ thống có thể hiển thị thông tin khởi động, gói, dịch vụ, lưu trữ, chẩn đoán, quản trị người dùng/nhóm và thông tin hệ thống. Đây là những công cụ quản trị hoặc chẩn đoán; một trang có thể ở chế độ chỉ đọc hoặc yêu cầu ủy quyền tùy thuộc vào hoạt động.

**Dịch vụ** có bốn trang: đơn vị hệ thống, đơn vị người dùng, đơn vị lỗi và nhật ký (Nhật ký). Mỗi danh sách đơn vị hiển thị một dòng `Filter:`, mà `f` quay vòng và một hộp `Search:` có viền, được mở bằng `/`. Mỗi đơn vị hiển thị trạng thái Hoạt động và Đã bật dưới dạng các dấu chấm đầy hoặc trống. Trong Nhật ký, `b` chuyển đổi giữa lần khởi động hiện tại và lần khởi động trước đó, `u` quay vòng thiết bị và `p` quay vòng mức độ ưu tiên; dòng `Filter:` hiển thị các giá trị đang được sử dụng. `Enter` mở một đơn vị hoặc một mục nhật ký.**Dịch vụ dành cho nhà phát triển** báo cáo ba nguồn hướng tới phát triển trong một trang chỉ đọc: dịch vụ systemd của người dùng hiện tại, các cổng TCP hiện đang nghe (`ss`) và các vùng chứa đang chạy từ bất kỳ thời gian chạy vùng chứa đã cài đặt nào (Podman, Docker). Nó chỉ xuất hiện trên lưới Home khi có ít nhất một trong số này có sẵn trên hệ thống - một máy không có phiên người dùng systemd, không có Podman và không có Docker sẽ không hiển thị mục nhập. `r` làm mới ba nguồn một cách độc lập, do đó, một nguồn chậm hoặc bị lỗi (ví dụ: thời gian chạy vùng chứa đã ngừng phản hồi) không chặn các nguồn khác hiển thị dữ liệu của chúng.

**Gói** được nhóm theo nguồn. **Kho lưu trữ chính thức** và khi trình trợ giúp AUR (`paru` hoặc `yay`) được cài đặt, **AUR (Kho lưu trữ người dùng Arch)** mỗi trình trợ giúp sẽ mở **Cài đặt**, **Xóa** và **Cập nhật**; **Bộ nhớ đệm**, **Lịch sử**, **Hạ cấp** và **Mirrors** theo sau. Ba tác vụ chia sẻ một màn hình: hộp **Tìm kiếm** được cố định ở trên cùng lấy các chữ cái bạn nhập và các gói được liệt kê bên dưới nó trong khối **Kết quả** (**Cập nhật** trên Cập nhật), trong đó một danh sách trống cho biết lý do (ví dụ **Không có kết quả**, **Không có mục nào phù hợp** hoặc **Không có bản cập nhật đang chờ xử lý**). **Cài đặt** tìm kiếm các kho lưu trữ (`pacman -Ss`) hoặc AUR khi bạn nhấn `Enter` ở thanh Tìm kiếm trên cùng; **Xóa** liệt kê các gói đã cài đặt (danh sách AUR hiển thị các gói được xây dựng bên ngoài kho, `pacman -Qm` và trong khi menu AUR được hiển thị, chúng sẽ rời khỏi danh sách Chính thức); **Cập nhật** liệt kê các bản cập nhật đang chờ xử lý (`pacman -Qu` hoặc `paru -Qua`/`yay -Qua` cho AUR). Trên một gói, `Enter` chạy nhiệm vụ của màn hình và `→` mở trang gói, với **Cài đặt** (hoặc **Cài đặt lại**) và, trong **Vùng nguy hiểm**, **Xóa**; `←` hoặc `Esc` quay lại. **Kho lưu trữ chính thức → Xóa** bắt đầu bằng **Mồ côi**, trong đó `Space` đánh dấu các gói (`[x]`), `Enter` mở một gói và **Xóa được đánh dấu** loại bỏ chúng cùng nhau. Từ danh sách Kết quả cập nhật, `Tab` di chuyển đến thanh **Hành động** trong danh sách (**Nâng cấp tất cả** và **Làm mới cơ sở dữ liệu** hoặc **Nâng cấp tất cả (AUR)** trên AUR), `←`/`→` di chuyển giữa các nút của nó, `Enter` chạy nút tập trung và `↑`/`↓` (hoặc gõ) đưa tiêu điểm trở lại danh sách; **Nâng cấp tất cả** bị mờ và bỏ qua trong khi không có bản cập nhật nào được liệt kê. Đường dẫn và tìm kiếm toàn cầu đi theo cùng một đường dẫn, ví dụ **Gói > Kho lưu trữ chính thức > Xóa > Trẻ mồ côi**. Việc dọn dẹp bộ nhớ đệm nằm trong Vùng nguy hiểm của trang Bộ nhớ đệm và **Gương → Định cấu hình gương** đặt các tùy chọn phản chiếu trước **Tạo bản xem trước**. Mọi thao tác gói đều hiển thị kế hoạch của nó hoặc cảnh báo PKGBUILD đối với AUR và yêu cầu xác nhận (`y` xác nhận, `n` hoặc `Esc` hủy) trước khi chạy và đầu ra của nó được hiển thị trong khi chạy. Các hoạt động AUR chạy trình trợ giúp với tư cách là người dùng của bạn (`-S <package>` để cài đặt hoặc cập nhật một, `-Sua` để cập nhật tất cả); Việc xóa AUR sử dụng `pacman -Rns` giống như các gói chính thức. Khi các gói đã cài đặt khác phụ thuộc vào gói bạn xóa (chính thức hoặc AUR), xác nhận sẽ đặt tên cho chúng và đưa ra hai cách: **Chỉ xóa** gói đã chọn (`pacman -Rdd`, bỏ qua kiểm tra phụ thuộc, do đó các gói cần gói này có thể ngừng hoạt động) hoặc **Xóa với các gói phụ thuộc** (`pacman -Rcns`, với số lượng gói mà nó xóa). Tiêu điểm bắt đầu vào **Hủy**, thao tác này sẽ giữ lại mọi gói hàng; `Enter` chạy hàng được tập trung và không có phím tắt `y` trong lựa chọn này.Thanh **Tìm kiếm** trên cùng sở hữu truy vấn gói. `/` tập trung vào nó mà không xóa văn bản của nó; gõ cũng di chuyển tiêu điểm ở đó. Chỉnh sửa bằng các phím mũi tên, `Home`, `End`, `Backspace`, `Delete` hoặc dán. Trong **Cài đặt**, `Enter` gửi truy vấn kho lưu trữ/AUR ngay cả khi có kết quả trước đó, sau đó chuyển tiêu điểm đến kết quả; tìm kiếm chính thức yêu cầu truy vấn không trống và tìm kiếm AUR yêu cầu ít nhất hai ký tự. Việc chỉnh sửa truy vấn Cài đặt sẽ hủy yêu cầu đang chờ xử lý và xóa các kết quả cũ của nó; một yêu cầu mới đang chờ `Enter`. Trong **Xóa**, **Cập nhật** và **Mồ côi**, việc chỉnh sửa sẽ lọc danh sách đã tải ngay lập tức mà không cần truy vấn kho lưu trữ khác. `↓` chuyển từ Tìm kiếm sang kết quả, `↑` ở kết quả đầu tiên quay lại Tìm kiếm và `Esc` trong Tìm kiếm quay lại danh sách trong khi vẫn giữ nguyên truy vấn. `Enter` trên một kết quả sẽ thực hiện hành động của gói đó. Thanh hành động Cập nhật vẫn có thể truy cập được bởi `Tab` và `/` sẽ trả về Thanh tìm kiếm từ thanh đó.

Tất cả các hoạt động trong **Gói** đều mở ra một thiết bị đầu cuối tương tác tích hợp: các gói chính thức, AUR, dọn dẹp bộ đệm, nhân bản và thay đổi kho lưu trữ/khóa. Trả lời lời nhắc của chương trình bằng cách gõ và nhấn `Enter`; pacman chạy mà không có `--noconfirm` trong thiết bị đầu cuối này. `Esc` ẩn chế độ xem và duy trì quá trình chạy. `F8` mở lại cùng một phiên từ bất kỳ trang Gói nào; **Hiển thị thiết bị đầu cuối** cũng có sẵn trong menu của nó. Phần chân trang hiển thị giao dịch đang chạy, đã hoàn thành hay không thành công. Sau khi lệnh thành công, `Done!` xuất hiện bên trong thiết bị đầu cuối sau đầu ra cuối cùng của nó, kể cả khi thiết bị đầu cuối bị ẩn. `Shift+PageUp` / `Shift+PageDown` cuộn lịch sử, `Shift+End` quay lại đầu ra trực tiếp và `Ctrl+C` gửi một ngắt tới tiến trình con. Trong khi thiết bị đầu cuối được tập trung, `q` và `?` là đầu vào chứ không phải là phím tắt chung. Đầu ra, màu ANSI và con trỏ được giữ nguyên và việc thay đổi kích thước sẽ cập nhật thiết bị đầu cuối con. Lịch sử lưu giữ tới 10.000 dòng cho đến khi thực hiện thao tác tiếp theo hoặc ứng dụng thoát. Mỗi lần chỉ có một giao dịch chạy; ẩn thiết bị đầu cuối của nó không cho phép ứng dụng thoát trong khi nó đang chạy. Không có vỏ miễn phí được mở.

Bản xem trước nâng cấp đọc cơ sở dữ liệu đồng bộ hóa hiện có (`pacman -Su --print`) mà không thay đổi chúng hoặc yêu cầu root. Sau khi xác nhận, bản nâng cấp đầy đủ sẽ chạy `pacman -Syyu` thông qua trình trợ giúp đặc quyền với `pkexec`; đồng bộ hóa có thể tiết lộ các bản cập nhật bổ sung. Việc cài đặt, xóa gói, làm mới cơ sở dữ liệu, dọn dẹp bộ đệm, hạ cấp, thay đổi kho lưu trữ/khóa và ghi nhân bản cũng sử dụng trình trợ giúp đặc quyền đó. Người trợ giúp AUR tiếp tục xây dựng với tư cách là người dùng của bạn.**Kho lưu trữ chính thức → Kho lưu trữ** thêm và quản lý các kho lưu trữ pacman không chính thức. **Thêm** là bản nháp: chọn **Nguồn** của cấu hình (**URL** của tệp `.conf`, **Văn bản** được nhập hoặc dán trong trình chỉnh sửa nhiều dòng hoặc **Biểu mẫu** có **Tên**, **Máy chủ** và **SigLevel**, được viết theo bố cục Arch Wiki) và bên dưới **Khóa ký**, **Khóa** từ **URL** hoặc **Văn bản** (hoặc **Không**). Lựa chọn di chuyển với `←`/`→` hoặc `Enter`; các trường văn bản mở một trình soạn thảo trong đó `Enter` lưu một dòng duy nhất, `Ctrl+S` lưu một số dòng, dán tác phẩm và `Esc` hủy. Tải xuống chỉ chấp nhận địa chỉ `https://`. **Xem xét và thêm** các lượt tải xuống và kiểm tra mọi thứ với tư cách là người dùng của bạn, sau đó yêu cầu xác nhận, liệt kê các kho lưu trữ, máy chủ và SigLevel cũng như ID người dùng và dấu vân tay của khóa theo nhóm bốn, để so sánh với nhóm mà kho lưu trữ xuất bản. Xác nhận nhập khóa và ký tên cục bộ (`pacman-key --add`, `--lsign-key`), ghi `/etc/pacman.d/<name>.conf` (được đặt tên theo phần đầu tiên của nó) và thêm `Include = /etc/pacman.d/<name>.conf` vào `/etc/pacman.conf` (thay vào đó, một dòng nhận xét hiện có cho cùng một tệp sẽ được bật). Sau đó, Trung tâm điều khiển cung cấp `pacman -Syyu`, do đó cơ sở dữ liệu mới được đồng bộ hóa mà không cần nâng cấp một phần. Để lại **Thêm** cùng với bản nháp, hãy hỏi trước khi loại bỏ nó. **Danh sách** hiển thị mọi kho lưu trữ không chính thức, được viết bằng chính `pacman.conf` hoặc được bao gồm từ `/etc/pacman.d/`, được bật hoặc tắt; phần chính thức không bao giờ được liệt kê. Trang kho lưu trữ hiển thị tệp của nó, SigLevel, máy chủ và khóa được ghi, nút chuyển đổi **Đã bật** và trong **Vùng nguy hiểm**, **Xóa**. Vô hiệu hóa nhận xét dòng `Include` hoặc đặt tiền tố `#argvus-off ` cho mỗi dòng của một phần nội tuyến; xóa sẽ xóa dòng hoặc phần và tệp được bao gồm. Các kho lưu trữ của cùng một tệp được bao gồm sẽ được bật, tắt và xóa cùng nhau. Khi Trung tâm điều khiển nhập khóa, **Xóa** cũng đề xuất xóa khóa đó khỏi chuỗi khóa của pacman (`pacman-key --delete`). Mọi thay đổi đối với `/etc/pacman.conf` đều có một bản sao lưu bên cạnh (`pacman.conf.argvus-backup-…`) và các gói đã cài đặt sẽ không bao giờ bị xóa.

### Người dùng và nhóm

Khi có nhà cung cấp tài khoản, hãy mở **Trung tâm điều khiển → Quản trị hệ thống → Người dùng & Nhóm → Người dùng** hoặc tìm kiếm **người dùng**. Giao diện có thể liệt kê các tài khoản thông thường và hệ thống, tạo người dùng, chỉnh sửa siêu dữ liệu tài khoản, quản lý các nhóm chính và bổ sung, thay đổi hoặc khóa mật khẩu, mở khóa, yêu cầu thay đổi mật khẩu ở lần đăng nhập tiếp theo, đặt hoặc xóa hình đại diện và bật đăng nhập tự động.

Việc tạo người dùng yêu cầu tên người dùng, tên đầy đủ, shell, nhóm bổ sung và xác nhận mật khẩu tùy chọn. Mật khẩu trống sẽ tạo ra tài khoản bị khóa mật khẩu; nó không âm thầm tạo ra một mật khẩu có thể sử dụng được. Trang người dùng nhóm các trường tài khoản (tên đầy đủ, shell, nhóm chính và nhóm bổ sung, theo sau là **Lưu thay đổi**), chuyển đổi **Đăng nhập tự động**, hành động **Mật khẩu** (thay đổi, khóa, mở khóa, yêu cầu thay đổi ở lần đăng nhập tiếp theo) và hành động **Hình đại diện**. Tên người dùng, UID/GID và thư mục gốc chỉ được hiển thị để cung cấp thông tin. **Vùng nguy hiểm** ở cuối chứa các hành động phá hoại riêng biệt để xóa tài khoản trong khi vẫn giữ thư mục chính của nó hoặc xóa cả thư mục chính. Đọc văn bản xác nhận cẩn thận trước khi chọn cái sau. Hoạt động tài khoản được đặc quyền và có thể mở lời nhắc ủy quyền hệ thống. Người chào mừng có thể sử dụng hình đại diện thông qua việc tích hợp hình ảnh khuôn mặt tiêu chuẩn của tài khoản.**Đăng nhập tự động** bắt đầu phiên của tài khoản đã chọn ở lần khởi động tiếp theo mà không hiển thị màn hình đăng nhập, quy ước tương tự mà Gnome, LightDM và SDDM sử dụng cho tính năng này: không có mật khẩu nào được lưu trữ ở bất kỳ đâu. Việc kích hoạt nó cho một tài khoản sẽ vô hiệu hóa nó cho bất kỳ tài khoản nào đã được định cấu hình trước đó (mỗi lần chỉ có một tài khoản có thể tự động đăng nhập). Nó áp dụng một lần vào lần khởi động tiếp theo; sau khi đăng xuất thủ công, màn hình đăng nhập bình thường sẽ quay trở lại trong phần còn lại của phiên đó. Việc chuyển đổi được áp dụng ngay lập tức cùng với lời nhắc xác nhận và chỉ dành cho quản trị viên.

Các nhóm có trang danh sách và chỉnh sửa riêng, bao gồm quản lý thành viên và xóa nhóm. Trung tâm điều khiển không thay thế chính sách tài khoản của hệ thống: các tài khoản hệ thống được bảo vệ và các hoạt động yêu cầu ủy quyền vẫn tuân theo các hạn chế của hệ điều hành.

### Cài đặt Trung tâm điều khiển

**Cài đặt** là mục cuối cùng của trang chủ. Trang **Ngôn ngữ** của nó đặt ngôn ngữ người dùng thông qua locale.conf và trang **Style** của nó kiểm soát các biểu tượng trang trí và độ trong suốt của Trung tâm điều khiển. Kiểu không giữ độ mờ: độ mờ là cài đặt Hyprland chung, trong Hyprland > Blur. Trang Kiểu thay đổi cách trình bày của ứng dụng cài đặt; nó không phải là một công tắc vô hiệu hóa các dịch vụ ARGVUS hoặc xóa các nhà cung cấp cài đặt. Việc lưu các biểu tượng có thể yêu cầu ủy quyền trong khi thay đổi trực quan được áp dụng cho ứng dụng hiện tại. Esc trên Style quay lại trang Cài đặt và Esc ở đó sẽ quay lại trang chủ.

### Giới thiệu về ARGVUS

**Giới thiệu về ARGVUS** nằm trong **Công cụ & Chẩn đoán → Giới thiệu về ARGVUS** (tìm kiếm `about`, `hardware` hoặc `firmware` hoặc chạy `argvus control-center about`). Nó có năm tab, được chuyển đổi bằng `Tab`/`Shift+Tab`, `←`/`→` hoặc `h`/`l`: **Hệ thống & Phần cứng**, **Giới thiệu**, **Quyên góp**, **Tín dụng** và **Bản quyền**. Tiêu đề hiển thị đường dẫn đầy đủ, ví dụ `Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware`.

**Hệ thống & Phần cứng** ở chế độ chỉ đọc và nhóm các dữ kiện của máy này thành ba phần, với các giá trị được căn chỉnh trong một cột:

| Phần | Lĩnh vực |
| --- | --- |
| Khởi động | Phần sụn (nhà cung cấp, phiên bản và ngày), Loại (UEFI hoặc Legacy (BIOS)), Khởi động an toàn (chỉ UEFI), Ảo hóa, Bộ tải khởi động |
| Hệ thống | ARGVUS, Tên máy chủ, Hệ điều hành, Nhà phân phối, Hạt nhân, Kiến trúc, Trình quản lý cửa sổ (Hyprland), Bộ tổng hợp (Wayland), trình điều khiển GPU, GTK, Chủ đề (với Tối hoặc Sáng), Hiển thị (một dòng trên mỗi màn hình), Âm thanh (PipeWire), Shell (Quickshell), Đăng nhập (helloetd), Init (systemd), Gói, Thời gian hoạt động, Ngôn ngữ |
| Phần cứng | Pin (một dòng trên mỗi pin), CPU (với tần số tối đa), Luồng, Lõi, GPU, Bộ nhớ (đã sử dụng / tổng cộng), Hoán đổi, Bộ lưu trữ, Model (chỉ khi nó khác với bo mạch chủ), Bo mạch chủ |

Mọi giá trị đều được đọc từ hệ thống đang chạy mà không có đặc quyền và trang hoạt động giống nhau trên mọi máy. Giá trị không thể đọc được hiển thị dưới dạng `N/A` và một dòng không áp dụng sẽ bị bỏ qua:

- **Firmware**, **Model** và **Bo mạch chủ** đến từ DMI (`/sys/class/dmi/id`); các giá trị giữ chỗ như `To be filled by O.E.M.` bị bỏ qua. **Loại** và **Khởi động an toàn** đến từ `/sys/firmware/efi` và **Ảo hóa** từ `systemd-detect-virt`.
- **Bootloader** được đọc từ biến `LoaderInfo` EFI, biến systemd-boot, Limine và các bộ tải khởi động khác tuân theo Giao diện bộ tải khởi động xuất bản. GRUB không xuất bản nó nên nó được `/boot/grub` (hoặc `/efi/grub`) công nhận và hiển thị cùng với phiên bản của gói `grub`.
- Các phiên bản của ARGVUS, Wayland, Mesa, PipeWire, Quickshell, hellod, systemd và GRUB đến từ một truy vấn `pacman -Q` và được hiển thị mà không có bản phát hành gói và kỷ nguyên (`1:25.2.3-1` trở thành `25.2.3`). Phiên bản Hyprland đến từ `hyprctl version` hoặc từ gói khi không có phiên trả lời.- **Trình điều khiển GPU** đặt tên cho trình điều khiển được liên kết với từng thẻ DRM trong `/sys/class/drm`. Trình điều khiển độc quyền hiển thị phiên bản mô-đun của nó (ví dụ `nvidia 580.82.07`); trình điều khiển trong nhân hiển thị phiên bản Mesa (ví dụ `amdgpu (Mesa 25.2.3)`).
- **Hiển thị** liệt kê các màn hình đang hoạt động từ `hyprctl monitors -j`: đầu nối, độ phân giải, tốc độ làm mới và tỷ lệ (tỷ lệ bị bỏ ở mức 1).
- **Gói** tính tất cả các gói đã cài đặt, gói được cài đặt rõ ràng và gói nước ngoài (AUR hoặc được xây dựng cục bộ), từ `pacman -Qq`, `-Qqe` và `-Qqm`.
- **Thời gian hoạt động** đến từ `/proc/uptime` và là giá trị khi trang được mở. **Ngôn ngữ** đến từ `LANG` trong `/etc/locale.conf`.
- **Pin** liệt kê các loại pin của máy tính từ `/sys/class/power_supply`, được chọn theo loại nên không tính các thiết bị ngoại vi như chuột không dây. Mỗi dòng hiển thị mức sạc, trạng thái, tình trạng (công suất tối đa so với công suất thiết kế) và chu kỳ sạc khi phần sụn báo cáo chúng. Máy không có pin hiển thị **Không được phát hiện**.
- **CPU** thêm tần số tối đa cao nhất trong số tất cả các lõi (cpufreq), đây là tần số tối đa được xếp hạng trên các bộ xử lý lai; nó bị bỏ lại ở nơi không có cpufreq, như trong hầu hết các máy ảo. **Chủ đề** và **Lõi** đến từ `/proc/cpuinfo`.
- **Bộ nhớ** và **Hoán đổi** đến từ `/proc/meminfo` (bao gồm zram); một hệ thống không có trao đổi sẽ hiển thị **Đã tắt**.
- **Bộ lưu trữ** mô tả ổ đĩa trong `/`: kiểu máy, loại ổ đĩa (NVMe, SSD, HDD hoặc eMMC/SD; ổ đĩa ảo không hiển thị loại), hệ thống tệp cũng như dung lượng đã sử dụng và tổng dung lượng. Đĩa được tìm thấy thông qua các phân vùng, LUKS và LVM cho đến thiết bị vật lý.

Các giá trị được đọc ở chế độ nền trong lần đầu tiên trang được vẽ, do đó cả Trung tâm điều khiển đều không khởi động cũng như việc mở trang chờ chúng; trong khi đó, tab hiển thị **Đang tải thông tin hệ thống...**. Nhấn `c` trên tab này để sao chép toàn bộ báo cáo dưới dạng văn bản thuần túy vào bảng nhớ tạm (với `wl-copy`), chẳng hạn như để dán báo cáo vào báo cáo lỗi.

## Khả năng tìm kiếm và tùy chọn

Màn hình chính có một danh sách các tuyến đường có thể tìm kiếm được. Cụm từ tìm kiếm bao gồm các khái niệm hướng tới người dùng như giao diện, chủ đề, hình nền, phông chữ, phím tắt, chuột, bàn di chuột, màn hình, mạng và phiên. Các nhà cung cấp tùy chọn chỉ được đưa vào khi tính năng của chúng được biên dịch và khả năng thời gian chạy của chúng được phát hiện, do đó menu hiển thị có chủ ý nhận biết khả năng.

Trong khi tìm kiếm Trang chủ đang hoạt động, các khóa có thể in được sẽ được nhập vào truy vấn, bao gồm `j` và `k`; sử dụng các phím mũi tên `↑` và `↓` để di chuyển qua các tuyến đường phù hợp. Nhấn `Enter` để mở tuyến đã chọn, `Backspace` để chỉnh sửa truy vấn và `Esc` để xóa truy vấn hoặc thoát khỏi chế độ tìm kiếm.

## Quy trình làm việc xuất hiện

Mở **Giao diện** để xem các điều khiển trực quan tích hợp. Các nhóm của nó là:

- **Màn hình** — **Chủ đề** (chọn một nhóm đi kèm và chế độ Dính hoặc Giãn cách hoặc xuất, nhập, áp dụng và xóa các chủ đề của riêng bạn, xem [Nhập và xuất chủ đề](/docs/components/argvus-themes/import-and-export/)), **Chế độ hiển thị** (Dính hoặc Giãn cách, độc lập với chủ đề), **Màu nhấn** (dấu nhấn đi kèm hoặc màu RGB gồm sáu chữ số hợp lệ), **Hình nền** (hình nền đi kèm hoặc tùy chỉnh file) **Biểu tượng** (chủ đề biểu tượng đã cài đặt: tìm kiếm danh sách bằng `/`, chọn một biểu tượng có Dấu cách hoặc Enter, sau đó chọn **Áp dụng**; không có gì thay đổi trước khi Áp dụng) và **Phông chữ** (mục tiêu phông chữ và cài đặt kết xuất phông chữ, được mở trên trang Phông chữ).
- **Phong cách ứng dụng** — độ trong suốt của **Terminal** và **Launcher**.Thẻ Giao diện giữ các bề mặt sử dụng các cài đặt này: **Widget Telemetry** (bật bề mặt đo từ xa, chọn các khối có sẵn và sắp xếp lại chúng bằng `Shift+Up`/`Shift+Down` trên một khối tập trung) và **Bảng điều khiển** (bật, tắt và sắp xếp lại các thẻ bảng điều khiển với `Shift+Up`/`Shift+Down` trên thẻ tập trung, được áp dụng ngay lập tức thông qua cách sử dụng kéo và thả nguyên thủy giống như bảng điều khiển thực).

Những hành động này cập nhật trạng thái ARGVUS logic và áp dụng cấu hình thời gian chạy bị ảnh hưởng. Xem [Giao diện](/docs/user-guide/appearance/), [Chủ đề](/docs/user-guide/appearance/) và [Windows và bố cục](/docs/argvus-hyprland/windows-and-layout/).

## Sử dụng bàn phímỨng dụng cài đặt hỗ trợ điều hướng bằng bàn phím. Mỗi trang là một danh sách duy nhất: sử dụng các phím mũi tên hoặc `j`/`k` để di chuyển, `Enter` để mở hoặc chạy hàng đã chọn, `Space` để chuyển đổi, `Esc` để quay lại, `/` để tìm kiếm danh sách và `?` để được trợ giúp; chân trang hiển thị các phím áp dụng cho hàng đã chọn. Tác vụ là các hàng của danh sách, không phải nút và `Tab` không di chuyển đến chúng, ngoại trừ trên các trang có thanh **Tác vụ** trong danh sách (Ứng dụng mặc định, Múi giờ, Bố cục bàn phím, Ngôn ngữ hệ thống và Khóa SSH cũng như Tạo/Chỉnh sửa phím tắt). Các hàng chỉ hiển thị thông tin sẽ bị bỏ qua. Khi trường văn bản, bộ lọc tìm kiếm hoặc danh sách đang hoạt động, `q` và `?` được nhập dưới dạng văn bản thay vì thoát hoặc mở trợ giúp. Trên **Ứng dụng mặc định**, **Khôi phục mặc định** là nút trong thanh **Hành động** trong danh sách: `Tab` hoặc `←/→` tập trung vào nó và `Enter` yêu cầu đặt lại. **Đặt lại mặc định** trên các trang khác và **Khôi phục tất cả phím tắt** là các hàng trong **Vùng nguy hiểm** ở cuối trang của chúng; `r` yêu cầu thiết lập lại tương tự và trên danh sách phím tắt `r` khôi phục phím tắt đã chọn sau khi xác nhận. Trên **Ngôn ngữ & Vùng → Bố cục bàn phím**, `/` tập trung vào hộp **Tìm kiếm** ở đầu trang và `Esc` xóa nó; `●` đánh dấu bố cục đang được sử dụng. Nó tuân theo thanh tác vụ và phím tắt để xoay vòng bố cục và trang cập nhật trong vòng khoảng một giây mà không cần ghi bất cứ điều gì vào cấu hình. `Enter` đặt bố cục mặc định, bố cục đầu tiên mà Hyprland bắt đầu và bố cục được gắn nhãn `Default` chi tiết; `Space` thêm hoặc xóa bố cục (`[x]`). Các phím này chỉ thay đổi bản nháp và nút **Áp dụng** trong thanh **Hành động** bên dưới danh sách sẽ ghi bố cục và mặc định cho Hyprland trong một bước. Từ danh sách, `Tab` hoặc `←/→` di chuyển tiêu điểm đến thanh đó trong khi **Áp dụng** được bật (khi đó chân trang hiển thị `←/→ Actions`); `←/→` chuyển qua các nút và `↑/↓` quay lại danh sách. Việc rời khỏi trang có những thay đổi chưa được áp dụng sẽ yêu cầu xác nhận trước. Trên **Ngôn ngữ & Vùng → Múi giờ** bố cục giống nhau: `/` tập trung vào hộp **Tìm kiếm** ở trên cùng, `Enter` chọn một vùng trong bản nháp (được đánh dấu bằng `●`) và nút **Áp dụng** trong thanh **Hành động** bên dưới danh sách sẽ lưu nó vào danh sách của người dùng. `environment.d/20-timezone.conf`; `Tab`, `←/→` và `↑/↓` hoạt động như trên Bố cục bàn phím. Rời khỏi khu vực chưa được áp dụng, hãy yêu cầu xác nhận trước. Trên **Vị trí hệ thống** bố cục giống như Múi giờ: `/` tập trung vào hộp **Tìm kiếm** ở trên cùng, `Enter` hoặc `Space` đánh dấu hoặc bỏ đánh dấu một ngôn ngữ (`[x]`) trong bản nháp và nút **Áp dụng** trong thanh **Hành động** bên dưới danh sách ghi lựa chọn vào `/etc/locale.gen` và chạy `locale-gen`. Bởi vì việc đó sẽ tạo lại các ngôn ngữ hệ thống nên **Áp dụng** yêu cầu xác nhận trước; `Tab`, `←/→` và `↑/↓` hoạt động như trên hai loại còn lại. Trên **Màn hình & Đầu vào → Phím tắt → Tất cả Phím tắt**, hộp tìm kiếm cũng nằm ở trên cùng, với `/` tập trung vào nó, nhưng trang không có thanh **Hành động**: danh sách của nó giữ các phần phím tắt và **Vùng nguy hiểm**, còn chân trang chỉ liệt kê `r Restore default` trên chính danh sách đó chứ không phải trên **Chỉnh sửa phím tắt**.Các trang thu thập các thay đổi trước khi lưu chúng (người dùng, nhóm, tạo tài khoản, thay đổi mật khẩu và cấu hình tường lửa) kết thúc khối đó bằng hàng **Lưu** hoặc **Áp dụng** luôn mờ cho đến khi có gì đó thay đổi. Nếu quay lại sẽ loại bỏ các thay đổi chưa được lưu, Trung tâm điều khiển sẽ hỏi trước. Mọi xác nhận đều hiển thị **Xác nhận** và **Hủy** dưới dạng hai hàng với tiêu điểm là **Hủy**: `Enter` chạy hàng được tập trung, `y` xác nhận và `n` hoặc `Esc` hủy.

Trong **Giao diện**, mỗi trang là một danh sách duy nhất: chân trang chỉ hiển thị các phím áp dụng cho hàng đã chọn và `Space` cũng mở hoặc chọn hàng. Các trang thu thập các thay đổi trước khi áp dụng chúng (Thanh tác vụ, Đo từ xa tiện ích, Bảng điều khiển và thanh trượt hiệu ứng) kết thúc bằng hàng **Áp dụng**, hàng này sẽ bị mờ cho đến khi có gì đó thay đổi. Việc chọn **Áp dụng** chỉ hiển thị màn hình tải khi thay đổi tạo lại một bề mặt: chiều cao thanh tác vụ, nhóm tiện ích, biểu tượng hoặc thay đổi định dạng ngày và giờ sẽ khởi động lại thanh tác vụ; bật hoặc tắt Widget Telemetry hoặc thay đổi khối hoặc thứ tự của nó sẽ khởi động lại tiện ích; bật hoặc tắt Bảng điều khiển hoặc thay đổi thẻ khi Bảng điều khiển đang bật sẽ khởi động lại bảng điều khiển. Sau đó, màn hình sẽ duy trì cho đến khi thay đổi cuối cùng của trang được áp dụng, do đó không có thay đổi nào xuất hiện trước nó và bề mặt được khởi động lại một lần. Đường viền, độ trong suốt, độ mờ, hình động, khoảng trống, phông chữ, hình nền và các cài đặt khác được áp dụng tại chỗ mà không cần màn hình tải. Trên các hàng phần trăm, `←`/`→` (hoặc `+`/`-`) thay đổi giá trị theo bước 5 và `Enter` cho phép bạn nhập giá trị đó; sử dụng `Esc` để quay lại. Nếu quay lại sẽ loại bỏ những thay đổi chưa được áp dụng, Trung tâm điều khiển sẽ hỏi trước. Xóa chủ đề tùy chỉnh (`d`) hoặc thay thế chủ đề đã nhập sẽ yêu cầu xác nhận; tiêu điểm bắt đầu **Hủy**, `y` xác nhận và `n` hoặc `Esc` hủy.

## Kiên trì và thiết lập lại

Các trang cài đặt áp dụng các thay đổi thông qua nhà cung cấp sở hữu của chúng và lưu trạng thái người dùng được hỗ trợ. Các thay đổi về giao diện được lưu trữ trong cấu hình chuẩn và `argvus-config` là thành phần duy nhất ghi các tệp tiêu dùng dẫn xuất trong `~/.config/argvus/data/generated/`; những tệp được tạo đó không phải là nơi để thực hiện chỉnh sửa vĩnh viễn. Các trang đầu vào và liên kết phím có trạng thái cố định và hành động đặt lại riêng. Các thay đổi hệ thống khác có thể yêu cầu quyền hoặc tải lại dịch vụ.

Không có thiết lập lại chung cho mọi cài đặt ARGVUS. Khôi phục một thay đổi từ trang sở hữu nó hoặc sử dụng quy trình khôi phục được ghi lại bằng tài liệu của tính năng liên quan.

Hỗ trợ thiết lập lại có chủ ý cho mỗi tên miền:

- **Giao diện** cung cấp khả năng đặt lại màu đánh dấu về mặc định của chủ đề đang hoạt động. Nó không cung cấp một thiết lập lại diện mạo toàn cầu; khôi phục các giá trị giao diện khác từ các điều khiển của riêng chúng.
- **Phông chữ** có thể khôi phục tất cả cài đặt phông chữ, mục tiêu hoặc cài đặt riêng lẻ.
- **Ứng dụng mặc định** có thể khôi phục tất cả các cài đặt mặc định, một danh mục hoặc một bộ chọn riêng lẻ.
- **Phím tắt** có thể khôi phục một hoặc tất cả các liên kết và sau đó tải lại các liên kết phiên đã tạo.
- **Quy tắc cửa sổ** có trong **Máy tính để bàn & Đầu vào → Hyprland → Windows → Quy tắc** (tìm kiếm `window rules` hoặc chạy `argvus control-center window-rules`). Mỗi quy tắc có một hàng **Không gian làm việc** (`Enter` hoặc `←/→` bước từ 1 đến 10) và một hàng **Các lớp cửa sổ** mở ra trường văn bản có các biểu thức chính quy được phân tách bằng dấu phẩy. **Thêm quy tắc** tạo `rule-N` với không gian làm việc 1 và không có lớp nào. **Quy tắc loại bỏ** của mỗi quy tắc nằm trong **Vùng nguy hiểm** và yêu cầu xác nhận. Mọi thay đổi được ghi vào `hyprland.window_rules` trong `argvus config` và tải lại cấu hình ngay lập tức; các mẫu lớp không hợp lệ bị từ chối và hiển thị dưới dạng lỗi.- **Dự án** nằm trong **Công cụ & Chẩn đoán → Dự án** (tìm kiếm `projects` hoặc chạy `argvus control-center projects`). **Thêm thư mục dự án** và **Thêm thư mục gốc** nằm ở đầu trang và mở trường văn bản có đường dẫn; trình khởi chạy sẽ kiểm tra xem đó có phải là một thư mục hay không và hiển thị lỗi, bao gồm cả thời điểm thư mục đẩy số lượng dự án hiệu quả vượt quá 9 (`SUPER + CTRL + 1..9` không thể giải quyết thêm). Mỗi thư mục được định cấu hình, được đánh dấu là gốc (các thư mục con của nó là các dự án) hoặc một dự án, sẽ có khối riêng bên dưới biểu mẫu với hàng **Xóa** yêu cầu xác nhận. Trang gọi `argvus-projects`, do đó, các thay đổi sẽ được trình khởi chạy ghi lại và áp dụng vào lần chạy `SUPER + O` hoặc `SUPER + CTRL + 1..9` tiếp theo. Nếu `argvus-projects` chưa được cài đặt, trang sẽ thông báo như vậy.
- **Các trang hệ thống khác** chỉ hiển thị các hành động đặt lại, áp dụng, xóa hoặc khôi phục khi nhà cung cấp cơ bản hỗ trợ chúng.

Việc xóa tùy chọn người dùng có thể khiến nhà cung cấp quay trở lại trạng thái đóng gói/mặc định nhưng xóa tệp bằng tay không phải là quy trình khôi phục chung. Ưu tiên hành động đặt lại của trang hoặc lệnh dành riêng cho tính năng được ghi lại.

## Điểm vào CLI

Người trợ giúp là `argvus-control-center`, chạy qua `argvus control-center`. Nó có thể mở các khu vực tập trung trực tiếp, ví dụ:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```

Chạy `argvus control-center --help` cho các tuyến có sẵn trong phiên bản đã cài đặt. Tuyến dòng lệnh là một lối tắt vào cùng một ứng dụng cài đặt, không phải là một hệ thống cấu hình riêng biệt.

Các tuyến trực tiếp hữu ích bao gồm `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (với các trang `status`, `connections`, `activate`, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` và `proxy`), `audio`, `bluetooth`, `boot`, `packages` (với các trang `official`, `search`, `installed`, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` và `mirrors`), `storage`, `system-doctor` (`diagnostics` và `diagnostic` vẫn là bí danh), `about` (với các tab `system`, `about`, `donate`, `credits` và `copyright`), `power`, `session`, `displays` và `appearance`. Ngoại hình cũng chấp nhận các tuyến tập trung như `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` và `widget-telemetry`; màn hình chấp nhận `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` và `hdr`. Sử dụng đầu ra `--help` đã cài đặt khi tạo tập lệnh vì tính khả dụng vẫn phụ thuộc vào bản dựng đã cài đặt.

## Liên quan

- [Cấu hình đầu tiên](/docs/getting-started/)
- Bảng điều khiển
- [Ngoại hình](/docs/user-guide/appearance/)
- [Phím tắt](/docs/argvus-hyprland/keyboard-shortcuts/)
- [Chuột và bàn di chuột](/docs/argvus-hyprland/input/)

### Bác sĩ hệ thống

Mở **Công cụ & Chẩn đoán → System Doctor**, tìm kiếm nó hoặc chạy `argvus control-center system-doctor`. Các tuyến đường và bộ chọn danh mục `diagnostics` và `diagnostic` trước đó vẫn khả dụng. Nhấn `r` để làm mới, Enter để kiểm tra séc, `o` chi tiết để mở trang liên quan và Esc để quay lại. Bộ sưu tập chạy ở chế độ nền và ở chế độ chỉ đọc.Kiểm tra sử dụng số lượng đơn vị bị lỗi của hệ thống, công cụ đồ họa và ảnh chụp nhanh Bộ nhớ để biết mức sử dụng hệ thống tệp và tình trạng SMART. Một vấn đề cần có bằng chứng; các công cụ không khả dụng hoặc dữ liệu SMART không thể truy cập được hiển thị là không khả dụng và thăm dò không thành công hoặc hết thời gian chờ là lỗi. Các danh mục không có bằng chứng được thu thập không có nghĩa là việc kiểm tra thành công. Các bản cập nhật gói sử dụng cơ sở dữ liệu pacman cục bộ và không đồng bộ hóa các kho lưu trữ; một tệp khóa có thể đã cũ và không chứng minh được một thao tác đang chạy. SMART có thể yêu cầu quyền và không yêu cầu quyền truy cập nâng cao. Bản tóm tắt ghi lại bộ sưu tập đã hoàn thành cuối cùng dưới dạng thời gian Unix.

Storage và System Doctor chia sẻ ảnh chụp nhanh trong bộ nhớ trong tối đa 10 giây. Nhấn `r` sẽ thu thập dữ liệu mới một cách rõ ràng; yêu cầu đồng thời sử dụng lại bộ sưu tập vừa hoàn thành. Đầu ra của quy trình con Chẩn đoán và Lưu trữ được giới hạn ở 1 MiB trên mỗi luồng và đầu ra quá khổ được coi là bộ sưu tập không thành công.

## Ánh sáng ban đêm

Mở **Máy tính để bàn & Đầu vào > Hiển thị > Đèn ngủ** hoặc chạy `argvus control-center displays night-light`. Định cấu hình nhiệt độ, kích hoạt thủ công và lịch trình cục bộ cố định thông qua `argvus config display night-light`. Xem [Ánh sáng ban đêm](/docs/user-guide/night-light/) để biết các yêu cầu về lệnh, trạng thái và thời gian chạy.

## Ảnh chụp nhanh

Mở **Trung tâm điều khiển > Quản trị hệ thống > Ảnh chụp nhanh**, tìm kiếm
`Snapper` hoặc chạy `argvus control-center snapshot`. Tính năng vận chuyển hàng hóa `snapshot`
được bao gồm trong `full` (mặc định) và độc lập với `storage`.
Trang này được đánh dấu **Thử nghiệm** vì tính năng này vẫn đang được phát triển
và có thể không đầy đủ.

Trang này chỉ quản lý cấu hình Snapper hiện có có `SUBVOLUME`
`/` và `FSTYPE` của nó là `btrfs`. Nó kiểm tra ngàm chứa `/` bằng
`findmnt`; cấu hình có tên `root` trỏ đến nơi khác không được chấp nhận.
Nhiều cấu hình phù hợp yêu cầu lựa chọn. Gói vòm `snapper`
là tùy chọn và phải được cài đặt thủ công. Snapper phải hỗ trợ `--jsonout` và
cột hoạt động/mặc định rõ ràng. Không có cấu hình nào được tạo tự động.

Trên thư mục gốc không phải Btrfs, trang hiển thị **Hệ thống tập tin này không phải là Btrfs** và cung cấp
không có thao tác chụp nhanh. Phát hiện lỗi, thiếu Snapper, thiếu root
lỗi cấu hình, từ chối cấp phép và lỗi phụ trợ là các trạng thái riêng biệt.
Nếu truy vấn của người dùng bình thường bị từ chối, hãy nhấn **a - Ủy quyền** để yêu cầu Polkit
truy cập. Các hành động quản trị chạy trong trình trợ giúp `system-settings` bị hạn chế,
không ở mức TUI cao; xác thực có thể được yêu cầu lại trong các truy vấn sau này.
Cần có tác nhân Polkit đồ họa hiện có để xác thực.

Phím: **↑/↓** chọn, **Nhập** chi tiết/cấu hình, **c** tạo, **d** xóa,
**p** chính sách lưu giữ, **r** làm mới, **Esc** quay lại, **?** trợ giúp. Về việc giữ lại
xem, **↑/↓** chọn một chính sách, **e** chỉnh sửa, **c** thuật toán dọn dẹp chu kỳ và
**Nhập** xác nhận mở. Sáng tạo sẽ mở ra một mô tả tùy chọn
(tối đa 256 ký tự có thể in được); một mô tả trống sử dụng “hướng dẫn sử dụng ARGVUS
ảnh chụp nhanh”. Cả việc tạo và xóa đều yêu cầu xác nhận, ban đầu tập trung vào
Hủy bỏ. Việc xóa áp dụng cho một ảnh chụp nhanh đã chọn và từ chối ảnh chụp nhanh 0,
ảnh chụp nhanh hiện đang hoạt động và ảnh chụp nhanh mặc định. Người trợ giúp kiểm tra lại những điều này
điều kiện trước khi gọi Snapper. Danh sách làm mới sau khi hoạt động; bị gián đoạn
hoạt động không bao giờ được thử lại tự động. Trước tiên hãy kiểm tra danh sách được làm mới.

Để so sánh các ảnh chụp nhanh, hãy chọn cơ sở và nhấn **b**, sau đó chọn một cơ sở khác
đích và nhấn **v**. Trang hiển thị ID được đặt hàng, số lượng được tạo,
các đường dẫn đã bị xóa và sửa đổi cũng như sáu chỉ báo trạng thái của Snapper. Chọn một đường dẫn
và nhấn **Enter** để yêu cầu nội dung khác biệt; **Esc** quay lại đường dẫn đã thay đổi
và sau đó là danh sách ảnh chụp nhanh. Các chỉ báo mã hóa việc tạo/xóa hoặc nội dung/loạiở vị trí 1, quyền ở 2, chủ sở hữu ở 3, nhóm ở 4, thuộc tính mở rộng ở
5 và ACL ở 6. Dấu chấm có nghĩa là không có thay đổi nào ở vị trí đó. Đường dẫn liệt kê trạng thái;
diff hiển thị nội dung văn bản. Đầu ra khác biệt nhị phân/không phải UTF-8 không được hiển thị. Khác biệt
đầu ra trên 256 KiB được báo cáo là bị cắt bớt và không được hiển thị. không rõ ràng hoặc
dòng trạng thái không đúng định dạng sẽ bị từ chối. Định dạng dòng của Snapper không thể biểu thị đường dẫn
với các ngắt dòng rõ ràng; những cái tên như vậy không được hỗ trợ. Quyền
lỗi có thể được thử lại thông qua **a — Ủy quyền**; ảnh chụp nhanh đã bị xóa hoặc đã thay đổi
cấu hình gốc yêu cầu làm mới danh sách. Kiểm tra phụ trợ sử dụng giả mạo
`ProcessRunner` và không yêu cầu Snapper, Btrfs hoặc root.

Từ so sánh, chọn một đường dẫn và nhấn **r** để xem lại việc khôi phục tệp
từ ảnh chụp nhanh đích. Người trợ giúp xác minh rằng cả hai ảnh chụp nhanh vẫn
thuộc về cấu hình gốc, đường dẫn vẫn được so sánh và
nguồn là một tệp thông thường ở chế độ xem `/.snapshots/<id>/snapshot` tiêu chuẩn của Snapper.
Sự khác biệt giới hạn của Snapper được hiển thị khi ở dạng văn bản; khác biệt nhị phân hoặc quá khổ là
được xác định mà không tải toàn bộ nội dung của chúng vào giao diện người dùng. **Nhập** mở một
xác nhận riêng ghi rõ nguồn và đích; **Esc** hủy mà không cần
thay đổi tập tin. Xác nhận tạo ảnh chụp nhanh an toàn `single` trước, sau đó
sử dụng Snapper `undochange` chỉ cho đường dẫn đó và kiểm tra xem nó có khớp với đường dẫn đó không
ảnh chụp nhanh đã chọn. ID ảnh chụp nhanh an toàn được báo cáo và ảnh chụp nhanh được lưu giữ.
Khôi phục chia sẻ khóa được sử dụng bởi các nâng cấp gói ARGVUS, do đó nó không thể trùng lặp
một giao dịch pacman chính thức của Trung tâm điều khiển.

Phiên bản đầu tiên chỉ chấp nhận các tệp thông thường thuộc sở hữu gốc dưới quyền sở hữu gốc
các thư mục mà nhóm hoặc nhóm khác không thể ghi được. Nó từ chối các thư mục,
liên kết tượng trưng, liên kết cứng, tập tin đặc biệt, điểm gắn kết/khối con, nội bộ của Snapper
thư mục và vị trí người dùng có thể ghi. Tệp bị thiếu trong ảnh chụp nhanh đã chọn
không bao giờ bị xóa. Root của Snapper `undochange` không đảm bảo ứng dụng hoặc
tính nhất quán của cơ sở dữ liệu và không mang tính nguyên tử; ảnh chụp nhanh an toàn là điểm khôi phục,
không phải là một sự đảm bảo. Snapper chỉ bảo tồn siêu dữ liệu được bản dựng của nó hỗ trợ. riêng biệt
các tập tin con và tập tin bên ngoài thư mục gốc được định cấu hình vẫn nằm ngoài hoạt động.

Ảnh chụp nhanh thủ công là `single`, chỉ đọc theo mặc định của Snapper, không cần dọn dẹp
thuật toán được chỉ định bởi trang này. Bộ tính giờ Snapper hiện tại và các hoạt động bên ngoài
tiếp tục nộp đơn. Ảnh chụp nhanh không phải là hình ảnh toàn hệ thống: các tập con riêng biệt,
bao gồm `/home` riêng biệt, không được bao gồm. Phiên bản này cung cấp **không
khôi phục toàn bộ hệ thống, cài đặt hoặc tạo cấu hình Snapper**. Nhấn
**p** trên cấu hình gốc được hỗ trợ để kiểm tra các cài đặt lưu giữ đã biết.
Các giá trị bị thiếu được gắn nhãn là giá trị dự phòng ARGVUS, không phải giá trị được đọc từ cài đặt
Mẫu tạo cá hồng; các khóa không xác định chỉ mang tính chất chẩn đoán. Nhấn **e** để chỉnh sửa giá trị được hỗ trợ. Việc xác nhận cho thấy cái cũ và
giá trị mới; sau khi lưu, trang sẽ đọc lại cấu hình và xác minh nó.
Nhấn **c** để xoay vòng các thuật toán dọn dẹp và **Enter** để xem lại và xác nhận một thuật toán.
Dọn dẹp sử dụng thuật toán chính thức của Snapper; không có bản xem trước chính xác và
số lượng bị loại bỏ được báo cáo được đo bằng cách so sánh các danh sách sau khi thực hiện. Arch Snapper 0.13.2 cung cấp
`snapper-timeline.timer` và `snapper-cleanup.timer`; việc tạo dòng thời gian yêu cầu
bộ hẹn giờ dòng thời gian và các chính sách dọn dẹp được định cấu hình sẽ không chạy nếu không dọn dẹp
lập kế hoạch. Trang chỉ đọc trạng thái đơn vị và không bật bộ hẹn giờ. Giới hạn không gian yêu cầu cấu hình Btrfs
qgroups và không được trình bày ở đây.

### Thao tác gói và ảnh chụp nhanh

Việc cài đặt, cập nhật và gỡ bỏ gói không yêu cầu Snapper, Btrfs hoặc
cấu hình ảnh chụp nhanh. **Nâng cấp tất cả** chạy `pacman -Syyu` thông qua đặc quyền
người trợ giúp với `pkexec`; các bản cập nhật chính thức riêng lẻ chạy `pacman -S <package>`.ARGVUS không tạo ảnh chụp nhanh tự động cho các hoạt động này. Ảnh chụp nhanh
trang vẫn độc lập và giữ lại các yêu cầu Btrfs/Snapper của riêng nó.
Các móc được cấu hình riêng trong pacman vẫn được hệ thống kiểm soát.

### Xác thực thủ công

- Trên root ext4/xfs, mở trang bằng tiếng Anh và tiếng Bồ Đào Nha. Trong tiếng Bồ Đào Nha,
  xác minh thông báo chính xác **Este sistema de arquivos không é Btrfs** và xác minh
  c/d đó không mở được thao tác.
- Trong máy ảo Btrfs dùng một lần, kiểm tra Snapper vắng mặt, Snapper không có root
  cấu hình và một cấu hình có tên `root` trỏ đến một tập con khác.
- Với cấu hình gốc hiện có, liệt kê ảnh chụp nhanh, mở chi tiết, tạo
  ảnh chụp nhanh có dấu cách/ký tự đặc biệt, hủy thao tác xóa, sau đó chỉ xóa
  ảnh chụp nhanh mới được tạo sau khi xác nhận số lượng và mô tả của nó.
- Kiểm tra xác thực bị từ chối/hủy bỏ, ảnh chụp nhanh hoạt động/mặc định được bảo vệ,
  làm mới, điều hướng bàn phím, thay đổi kích thước và cả hai ngôn ngữ. Đừng thử xóa
  chống lại các ảnh chụp nhanh cần thiết để phục hồi.

Chỉ có gói `argvus` thay đổi. Xây dựng lại từ `de/argvus` với `make build` thành
sản xuất gói `.pkg.tar.zst`; cài đặt nó là một hành động điều hành riêng biệt.

Ngôn ngữ trong Ngôn ngữ & Vùng lưu ngôn ngữ trong hồ sơ người dùng (`~/.config/locale.conf`) mà không thay đổi `/etc/locale.conf`. Phiên sử dụng tùy chọn này sau khi đăng nhập lại. Trong phần Chào mừng, F6 Tiếp theo hoặc nút Tiếp theo ở chân trang ghi lại quá trình xem xét và tiến bộ; Shift+Tab tập trung vào nút và Enter kích hoạt nó, kể cả trên các trang thông tin.

Bluetooth chỉ xuất hiện trong Chào mừng khi Trung tâm điều khiển phát hiện bộ điều hợp. Tiến trình hiện có cho bước Bluetooth ẩn được giữ nguyên.

Ngôn ngữ và bố cục bàn phím phiên là tùy chọn cá nhân và không yêu cầu đặc quyền của quản trị viên. Cấu hình mới mặc định có bố cục bàn phím tiếng Anh và tiếng Mỹ; sở thích cá nhân hiện tại được bảo tồn. Ngày/giờ, NTP, tạo ngôn ngữ hệ thống và sơ đồ bàn phím bảng điều khiển (TTY) vẫn giữ nguyên hành vi quản trị trên toàn hệ thống của chúng.

Đối với bàn phím phiên Brazil, hãy chọn `br`, áp dụng bố cục, sau đó chọn `abnt2` trong Biến thể. Điều này viết cấu hình Hyprland cá nhân mà không có sự cho phép hành chính; sơ đồ bàn phím của bàn điều khiển là riêng biệt. ABNT2 sử dụng khóa chết: `~`, tiếp theo là `a` tạo ra `ã`, và `~`, tiếp theo là Space tạo ra `~`. Việc áp dụng biến thể sẽ kích hoạt bố cục mặc định đã định cấu hình của nó; lỗi kích hoạt được báo cáo.

Múi giờ lưu TZ theo `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` mà không được phép. Đăng xuất và đăng nhập lại để các ứng dụng kế thừa nó. Ngày & Giờ tiếp tục hiển thị và chỉnh sửa đồng hồ hệ thống theo múi giờ hệ thống, với ủy quyền quản trị; NTP không thay đổi

Cấu hình hiển thị và phân công không gian làm việc được áp dụng thông qua cấu hình Lua được tạo và tải lại Hyprland. Nguồn giám sát sử dụng bộ điều phối Lua DPMS được Hyprland 0.56.2 hỗ trợ.

Khả năng kết xuất bóng được giới hạn ở mức 1–4 (mặc định 4) để tương thích với Hyprland 0,57. Các giá trị được lưu trữ cũ hơn 4 sẽ được kẹp khi tải cấu hình.

## Cửa sổ chao đảo

Mở **Desktop & input → Hyprland → Animations → Windows** và sử dụng
Điều khiển **Lắc lư**. Tìm kiếm `wobble` hoặc mở trang trực tiếp:

```sh
argvus control-center hyprland animations windows
```

Wobble bị tắt theo mặc định. Giá trị mặc định ARGVUS của nó là độ cứng `100`, giảm chấn
`10`, khối lượng `1`, cường độ `1` và độ phân giải lưới `8`. Độ cứng và khối lượng chấp nhận
`0.0001–1000`, giảm chấn `0–1000`, cường độ `0–2` và chia lưới một số nguyên `2–32`.
Độ phân giải lưới cao hơn làm tăng chi phí kết xuất. Sử dụng các phím mũi tên để điều chỉnh
giá trị hoặc Enter để nhập chúng. **Khôi phục mặc định Wobble** thay đổi bản nháp;**Áp dụng** lưu tất cả các chỉnh sửa hoạt ảnh đang chờ xử lý và tải lại Hyprland một lần mà không cần
khởi động lại phiên. Việc quay lại trong Ảnh động sẽ giữ lại bản nháp được chia sẻ.

Công tắc Ảnh động toàn cầu sẽ ngăn chặn sự rung chuyển trong khi vẫn giữ nguyên trạng thái riêng lẻ của nó.
cài đặt. Các giá trị chuẩn nằm dưới `effects.wobble_*` ở trạng thái ARGVUS;
`hypr/state.lua` được tạo và không được chỉnh sửa thủ công.

Điều này yêu cầu Hyprland **0.57.0 hoặc mới hơn**, được thực thi bởi `argvus-hyprland`'s
phụ thuộc gói. Phiên bản 0.56 không được hỗ trợ cho cấu hình này. các
cường độ tối đa theo[phát hành mã nguồn 0.57.0](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
giới hạn ở `2`, mặc dù tài liệu mới hơn có thể liệt kê `3`.

Lời chào mở đầu bằng logo ARGVUS và CHÀO MỪNG. Nhấn F6 Tiếp theo để hiển thị danh sách kiểm tra mà không cần hoàn thành bước nào.

Cài đặt mới mặc định theo múi giờ cá nhân `America/New_York` và chỉ bật bố cục bàn phím `us`. Cài đặt cá nhân hiện tại được giữ nguyên; múi giờ của hệ thống không thay đổi.

Các tùy chọn mô-đun thanh tác vụ cho Pin và Bluetooth chỉ xuất hiện khi phát hiện thấy phần cứng tương ứng. Ẩn Giữ thức ẩn chỉ báo của nó mà không thay đổi chất ức chế huyền phù. Tiện ích hỗ trợ ngăn di chuột thu gọn. Bản trình bày biểu tượng không gian làm việc hiển thị các biểu tượng cho các cửa sổ đang mở; khay hệ thống điều khiển các biểu tượng ứng dụng nền.


Trang phím tắt bao gồm **bộ chọn tác nhân AI (gói argvus-ai tùy chọn)** với SUPER + A mặc định. Trang này sử dụng ID danh mục app.ai, do đó, việc ánh xạ lại và vô hiệu hóa sẽ tuân theo cấu hình keyboard_shortcuts chuẩn giống như các mục nhập khác. Liên kết Lua thực thi của nó được cung cấp bởi argvus-ai; riêng mục danh mục sẽ không kích hoạt nó nếu không có gói tùy chọn đó.


Trung tâm điều khiển > Giao diện > Thanh tác vụ > Biểu tượng > Các mô-đun tùy chọn hiện bao gồm **Đại lý AI**. Nó mặc định được bật và được lưu trữ trong /taskbar/icons/ai_agents_enabled. Chuyển đổi nó trong bản nháp trên thanh tác vụ và Áp dụng để duy trì và tạo lại nhóm tiện ích; việc vô hiệu hóa nó không thay đổi các mục nhập phím tắt SUPER + A hoặc trình khởi chạy ứng dụng.

Công tắc tự khởi động chỉnh sửa mục nhập `.desktop` của người dùng hoặc tạo bản sao người dùng của mục nhập hệ thống. Tên, lệnh và cờ kích hoạt đến từ `[Desktop Entry]`; các nhóm hành động bổ sung trên máy tính để bàn không thay đổi.

Xác nhận dịch vụ giữ lại phạm vi hệ thống/người dùng đã chọn cùng với đơn vị và hành động. Một đơn vị có cùng tên trong phạm vi khác không thể chuyển hướng một hành động đã được xác nhận; các trang chi tiết cũng giữ nguyên phạm vi ban đầu.

Bộ lưu trữ bao gồm các điểm gắn kết lồng nhau như `/home`, `/boot` và `/boot/efi` từ kho lưu trữ hệ thống tệp. Diagnostics sử dụng cùng một khoảng không quảng cáo để kiểm tra việc sử dụng hệ thống tập tin gốc và hệ thống tập tin gốc hiện có.

Các ảnh chụp nhanh an toàn khôi phục mới sử dụng thuật toán dọn dẹp `number` của Snapper. Việc loại bỏ chúng tuân theo chính sách lưu giữ số đã định cấu hình khi quá trình dọn dẹp diễn ra. Các ảnh chụp nhanh hiện có không có thuật toán dọn dẹp sẽ không bị thay đổi và việc tạo thủ công sẽ giữ nguyên chính sách hiện có.

So sánh ảnh chụp nhanh báo cáo riêng các bản ghi đường dẫn không được hỗ trợ. Các tệp hợp lệ vẫn có sẵn để xem xét; các đường dẫn có mã hóa không hợp lệ, ký tự điều khiển hoặc các thành phần không được hỗ trợ không cung cấp các hành động khác biệt hoặc khôi phục.

Cài đặt nguồn ghi danh sách thả vào đăng nhập ARGVUS trong `[Login]`, tạo thư mục của nó khi cần. Lỗi tải lại được báo cáo. Ánh xạ hành vi của nắp pin tới `HandleLidSwitch`; bối cảnh `default` cũ là bí danh, trong khi AC và hành vi được gắn đế sử dụng các khóa đăng nhập riêng biệt của chúng.

Ứng dụng Mirrorlist chỉ chấp nhận các nhận xét, dòng trống và các mục nhập HTTP/HTTPS `Server`. Tải trọng được giới hạn ở 64 KiB và được kiểm tra trước khi nâng lên và một lần nữa bởi người trợ giúp đặc quyền; các chỉ thị pacman như `SigLevel` và `XferCommand` đều bị từ chối.Dọn dẹp bộ nhớ đệm cho các gói đã gỡ cài đặt không giữ lại phiên bản nào được lưu trong bộ nhớ đệm. Từ chối hoạt động kho lưu trữ bao gồm chia sẻ bởi nhiều phần kho lưu trữ. Tên máy chủ có khoảng trắng xung quanh và lựa chọn ngôn ngữ được tạo trống sẽ bị từ chối. Tệp cài đặt ghi từ chối các đích liên kết tượng trưng và các tệp tạm thời hiện có.

Tìm kiếm gói và tải trang sẽ loại bỏ kết quả từ trang hoặc truy vấn trước đó. Sau đó, trang hiện tại sẽ được tải lại nên kết quả cũ không thể đánh dấu một chế độ xem khác là được làm mới gần đây.

Đầu ra giao dịch được vệ sinh và giới hạn. Màn hình giữ lại tới 10.000 dòng hoặc 4 MiB và báo cáo đầu ra bị bỏ qua. Luồng quy trình được ghi lại tôn trọng giới hạn được định cấu hình của chúng, mặc định là 8 MiB để thực thi trực tiếp; sản lượng dư thừa sẽ được xả và báo cáo sau khi quá trình kết thúc.

Kiểm kê khởi động đọc hạt nhân Arch `pkgbase` để định vị hình ảnh hạt nhân, tệp initramfs và cài đặt trước mkinitcpio trong khi hiển thị bản phát hành hạt nhân riêng biệt. Mảng mkinitcpio theo nghĩa đen được đọc mà không bao gồm dấu ngoặc đơn hoặc thực thi cấu hình.

Các thay đổi về nguồn điện cho nắp, nút nguồn và chế độ giữ nguyên chỉ được lưu vào cấu hình chuẩn sau khi hành động trong thời gian chạy thành công. Việc hủy ủy quyền sẽ giữ nguyên giá trị đã lưu trước đó. Khi không có phần đăng nhập ARGVUS nào cung cấp giá trị, cấu hình logind chính sẽ được đọc dưới dạng dự phòng.

Sau khi thay đổi lưu giữ không thành công, trang sẽ đọc lại cài đặt hiện tại trước khi hiển thị lại giá trị của chúng. Lựa chọn cấu hình bắt đầu ở mục đầu tiên.

Các hoạt động của gói ARGVUS chia sẻ một khóa độc quyền với khả năng khôi phục có chọn lọc. Việc cài đặt, gỡ bỏ, cài đặt lại, hạ cấp, làm mới siêu dữ liệu, dọn dẹp bộ đệm và các thay đổi về kho lưu trữ hoặc bản sao đều bị từ chối trong khi khóa đó được giữ. Sự phối hợp này bao gồm các hoạt động được bắt đầu thông qua trình trợ giúp ARGVUS; các công cụ pacman và AUR bên ngoài không sử dụng khóa này.

Trong khi thao tác nền đang áp dụng các thay đổi, `q` và `Ctrl+C` sẽ giữ Trung tâm điều khiển mở và hiển thị thông báo chờ. Điều này cũng áp dụng sau khi điều hướng khỏi trang đã bắt đầu hành động. Đợi kết quả của nó, sau đó yêu cầu thoát lại. Việc làm mới chỉ đọc thông thường không chặn lối thoát.

Mật khẩu Wi-Fi, DNS thủ công và các trường tìm kiếm mạng chấp nhận dán trong ngoặc. Các điều khiển đã dán và ngắt dòng được bỏ qua. Các phím tắt ký tự chỉ Ctrl hoặc chỉ Alt không được hỗ trợ sẽ không chèn chữ cái. Người chỉnh sửa chấp nhận các sự kiện AltGr có thể in được mã hóa dưới dạng Ctrl+Alt; Ctrl+S chỉ lưu mà không có Alt. Trình soạn thảo văn bản được chia sẻ giữ điều hướng dọc trong các cột trực quan và căn chỉnh cuộn ngang cho các ký tự rộng.

Các thay đổi về âm lượng và tắt tiếng cho micrô và đầu ra không mặc định không còn ghi đè các tùy chọn đầu ra mặc định đã lưu. Phần phụ trợ chỉ duy trì các tùy chọn đó khi nó có thể xác định nút đã chọn làm đầu ra mặc định hiện tại. Trang hiển thị âm lượng được khuếch đại trên 100% theo báo cáo của PipeWire; chỉnh sửa âm lượng rõ ràng vẫn bị giới hạn ở mức 0–100%.

Tổng quan về phần cứng xác định các mẫu GPU riêng biệt với địa chỉ PCI và loại thiết bị của chúng, giữ nguyên tên thương mại trong ngoặc. Tóm tắt của nó hiển thị trình kết xuất OpenGL được truy vấn cho phiên và tất cả tên thiết bị được trả về bởi đầu dò Vulkan, bao gồm cả các thiết bị phần mềm. Những kết quả toàn cầu này không được trình bày dưới dạng khả năng của từng GPU vật lý.

Khi quản trị viên khóa hoặc xóa tài khoản được định cấu hình để đăng nhập tự động, trước tiên nhà cung cấp tài khoản sẽ vô hiệu hóa mục đăng nhập tự động được chào đón. Nếu việc dọn dẹp đó không thành công, hoạt động tài khoản sẽ bị từ chối. Nếu việc khóa hoặc xóa sau đó không thành công, tính năng đăng nhập tự động vẫn bị vô hiệu hóa. Đăng nhập tự động cho tài khoản khác được giữ nguyên; mở khóa tài khoản không cho phép đăng nhập tự động. Các phiên hiện tại không được đóng và lời chào không được khởi động lại.


## Tạo và quản lý phím tắt**Màn hình & Phương thức nhập → Phím tắt** mở ra một menu có **Tạo phím tắt** và **Tất cả phím tắt**. Bí danh CLI hiện có (`keybindings`, `shortcuts`, `keyboard`) mở menu này; thêm `create` hoặc `list` để mở trực tiếp một đứa trẻ. Tìm kiếm toàn cầu cũng tìm thấy cả hai đứa trẻ.

**Tạo phím tắt** yêu cầu mô tả, tổ hợp bàn phím và loại hành động. Chọn **Phím tắt hiện tại** và nhấn `Enter`, sau đó nhấn tổ hợp phím mong muốn. **Thay đổi phím tắt** bắt đầu chụp tương tự; cũng không mở trình soạn thảo văn bản. Chỉ ghi lại những thay đổi ở bản nháp. Nếu một sự kết hợp đã được sử dụng thì xung đột sẽ được hiển thị ngay lập tức, ngay cả trước khi mô tả và lệnh được điền. Xác nhận **Thay thế** sẽ chọn kết hợp và ghi lại lựa chọn của bạn; nó không tạo lối tắt hoặc vô hiệu hóa lối tắt khác cho đến khi **Tạo**/**Lưu**. Việc hủy bỏ xung đột sẽ bảo tồn sự kết hợp trước đó. **Lệnh** chạy khi nhấn phím tắt; sự sáng tạo không bao giờ chạy nó. **Bộ điều phối Hyprland** sử dụng tên bộ điều phối Lua được hỗ trợ, chẳng hạn như `window.float`, với các đối số tùy chọn dưới dạng đối tượng JSON, ví dụ: `{"action":"toggle"}`. Các bản đồ con, liên kết chuột và cờ nâng cao vẫn nằm ngoài biểu mẫu này.

Sử dụng `Tab` để chuyển đổi giữa biểu mẫu và **Hành động**, `←/→` để di chuyển giữa **Tạo** và **Hủy** và `Enter` để chạy hành động đã chọn. Hủy hoặc `Esc` hủy bản nháp. Sau khi tạo, **Tất cả phím tắt** sẽ mở ra với mục mới được chọn và hiển thị. Các mục tùy chỉnh xuất hiện trong **Phím tắt tùy chỉnh**; tìm kiếm phù hợp với mô tả, phím và hành động. `Enter` chỉnh sửa mô tả, hành động và phím, `Space` bật/tắt và chọn hàng **Xóa** rồi nhấn `Enter` sẽ mở ra xác nhận tập trung vào Hủy. Xác nhận xóa bản ghi và quay lại danh sách được làm mới; hủy bỏ giữ nó.

Các phím tắt đã bật trong cùng một ngữ cảnh không thể âm thầm sử dụng lại tổ hợp: xung đột có thể định cấu hình yêu cầu xác nhận để thay thế và không thể thay thế các phím tắt được bảo vệ. Thay thế sẽ lưu lối tắt mới và vô hiệu hóa xung đột trong một giao dịch cấu hình. **Khôi phục mặc định** chỉ áp dụng cho các mục chính thức; **Khôi phục tất cả các phím tắt** bảo tồn các bản ghi tùy chỉnh. Nếu việc khôi phục sẽ xung đột với một phím tắt tùy chỉnh đang hoạt động, trước tiên hãy thay đổi hoặc tắt phím tắt tùy chỉnh đó.

Lưu và áp dụng chạy trong nền. Nếu lưu không thành công, bản nháp vẫn có sẵn. Nếu ứng dụng vào phiên không thành công sau khi lưu, trang sẽ cho biết nó đã được lưu và giữ lại cùng một ID để thử lại. Bản ghi tùy chỉnh tồn tại trong `/hyprland/custom_shortcuts` ở cấu hình chuẩn, được chiếu thành `data/generated/hypr/custom-shortcuts.lua`; không chỉnh sửa các tập tin được tạo.

Lỗi lối tắt vẫn tồn tại bên trong TUI: cửa sổ bật lên tự kích thước theo thông báo được gói và giữ **Enter OK** hiển thị. Chẩn đoán phím tắt kỹ thuật đi tới tạp chí cục bộ (`journalctl -t argvus-control-center`) thay vì stdout/stderr. Việc xóa được lưu nhưng không thể áp dụng sẽ báo cáo sự khác biệt đó; tải lại cấu hình phiên để xóa liên kết hoạt động trước đó.

**Trạng thái cập nhật** trong **Thanh tác vụ → Biểu tượng → Mô-đun tùy chọn** hiển thị số lượng cập nhật có sẵn và được bật theo mặc định. **Kiểm tra AUR** bị tắt theo mặc định và yêu cầu `paru` hoặc `yay`; áp dụng bản nháp trên thanh tác vụ để lưu một trong hai cài đặt. Kiểm tra chạy khi khởi động và hàng giờ; nhấp vào chỉ báo sẽ mở trang cập nhật Gói chính thức.

## Chế độ cửa sổ toàn cầu

Mở **Trung tâm điều khiển → Màn hình nền & Đầu vào → Hyprland → Windows → Chế độ** và chọn **Cửa sổ xếp chồng (Mặc định)** hoặc **Cửa sổ nổi**. Bộ chọn tương tự có sẵn từ biểu tượng chế độ cửa sổ trong nhóm tiện ích trên thanh tác vụ.Tùy chọn này áp dụng cho mọi màn hình và không gian làm việc và được ghi nhớ sau khi đăng xuất. Nổi chuyển đổi các cửa sổ ứng dụng hiện có và các cửa sổ mới mở. Ngói chỉ khôi phục các cửa sổ mà chính sách này đã chuyển đổi; các hộp thoại và các cửa sổ khác vốn đã nổi vẫn nổi. Một cửa sổ được ghim rõ ràng trong khi Nổi đang hoạt động cũng vẫn nổi để bảo toàn mã pin của nó. Bố cục lát gạch được tính toán lại; cây phân chia chính xác trước đó của nó không được khôi phục.

`SUPER + SHIFT + Space` giữ trạng thái bình thường trên mỗi cửa sổ trong Ngói. Trong chế độ Nổi, nó không chuyển cửa sổ về chế độ xếp lát. Các bảng, thanh tác vụ và thông báo là các bề mặt lớp vỏ và không bị ảnh hưởng. Toàn màn hình và tối đa hóa tiếp tục sử dụng hoạt động bình thường của Hyprland.

Tùy chọn chuẩn là `/hyprland/window_mode` trong `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Phép chiếu được tạo ra của nó là `data/generated/hypr/window-mode.lua`; không chỉnh sửa nó. Sổ đăng ký khôi phục trên mỗi cửa sổ nằm trong `$XDG_RUNTIME_DIR/argvus`, nằm trong phạm vi phiên bản bộ tổng hợp và vẫn tồn tại khi tải lại cấu hình. Nó không được sử dụng lại trong một phiên mới.

Nếu ứng dụng đang chờ xử lý hoặc bị lỗi, giao diện sẽ phân biệt ứng dụng này với tùy chọn đã lưu. Chọn lại tùy chọn để thử lại. Nếu không có bộ tổng hợp, tùy chọn sẽ được lưu cho lần đăng nhập tiếp theo.

## Nút cửa sổ và thanh ứng dụng

Chế độ nổi thêm thanh tiêu đề ARGVUS với **Thu nhỏ**, **Tối đa hóa / Khôi phục** và **Đóng** đối với các cửa sổ ứng dụng thông thường. Bấm đúp vào thanh tiêu đề sẽ chuyển đổi tối đa hóa. Trang trí bổ sung được ẩn trong Ngói và toàn màn hình. Các ứng dụng tự vẽ các điều khiển có thể hiển thị cả hai thanh; Các điều khiển ARGVUS thống nhất là có chủ ý.

Thanh Waybar thứ hai xuất hiện trên mọi màn hình, ở cạnh đối diện với thanh tác vụ chính. Nó chỉ liệt kê các cửa sổ của không gian làm việc hiện tại của màn hình đó, bao gồm cả các cửa sổ được thu nhỏ của nó. Các tab nhóm ẩn không tạo thêm mục. Các mục được thu nhỏ vẫn giữ lại liên kết màn hình/không gian làm việc ban đầu của chúng. Nhấp vào cửa sổ không hoạt động để kích hoạt, cửa sổ đang hoạt động để thu nhỏ cửa sổ hoặc cửa sổ thu nhỏ để khôi phục cửa sổ đó; menu ngữ cảnh của nó cung cấp khả năng khôi phục, thu nhỏ, tối đa hóa/khôi phục và đóng. Các biểu tượng ứng dụng không xác định sẽ quay trở lại biểu tượng cửa sổ.

`SUPER + Alt + Up` đặt thanh chính ở trên cùng và thanh cửa sổ ở dưới cùng. `SUPER + Alt + Down` thì ngược lại. Việc lặp lại một hướng sẽ khiến các vị trí không thay đổi. Ngói chỉ có thanh chính. Việc quay lại Ngói sẽ khôi phục tất cả các cửa sổ được thu nhỏ trước khi xóa thanh cửa sổ, sau đó giữ nguyên chính sách xếp kề/nổi ban đầu.

Giảm thiểu sử dụng không gian làm việc `special:argvus-minimized` dành riêng. Không sử dụng không gian làm việc đó một cách thủ công. Quá trình `argvus windows service` ghi lại danh tính, không gian làm việc/màn hình ban đầu, hình học, toàn màn hình/phóng to và trạng thái pin trong sổ đăng ký thời gian chạy dành riêng cho phiên bản tổng hợp trong `$XDG_RUNTIME_DIR/argvus`. Các bản ghi vẫn tồn tại sau khi khởi động lại dịch vụ hoặc tải lại cấu hình nhưng không vượt qua các phiên tổng hợp. Đóng một cửa sổ sẽ xóa bản ghi của nó. Nếu màn hình biến mất, quá trình khôi phục sẽ sử dụng đầu ra có sẵn. Thanh khôi phục bị thiếu hoặc bị lỗi sẽ ngăn cản việc giảm thiểu mới; việc khôi phục không thành công sẽ giữ lại bản ghi và thanh khôi phục.

Phiên sở hữu `argvus-windows.service` và `argvus-window-bar.service`. Cái sau giữ con Waybar của nó không hoạt động trong Ngói. Plugin hyprbars được đóng gói bởi `argvus-hyprland`, được xây dựng cho Hyprland 0.57.0 ABI trên x86_64; nó không được biên dịch trong thư mục chính của người dùng. Xây dựng lại plugin bằng bản cập nhật ABI tổng hợp.

Các lệnh: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` và `minimize|restore|activate|maximize|close ADDRESS`. Các địa chỉ cửa sổ đến từ danh sách. `position top --persist` lưu trữ/dự án tùy chọn mà không cần khởi động lại thanh chính. Chi tiết lỗi về cấu hình và thời gian chạy được ghi vào nhật ký; lỗi hành động hiển thị cũng hiển thị thông báo đã dịch.

## Menu con Windows và đóng ứng dụng**Các nhóm Máy tính để bàn & Đầu vào → Hyprland → Windows** **Quy tắc**, **Không gian** và **Chế độ**.
Quy tắc chỉnh sửa vị trí cho mỗi ứng dụng; Spaces chỉnh sửa các khoảng trống bên trong và bên ngoài; Chế độ
chứa bộ chọn Ngói/Nổi ngay lập tức, tách biệt với biểu mẫu quy tắc.
Tìm kiếm sẽ mở ra các trang tương tự và Quay lại quay lại Windows trước Hyprland.

Các tuyến đi thẳng là `argvus control-center hyprland windows`, theo sau là tùy chọn
bởi `rules`, `spaces` hoặc `mode`. `window-rules` hiện có và
Các tuyến đường `appearance window-spaces` vẫn là bí danh và hiển thị các đường dẫn mới.

`SUPER + Q` đóng cửa sổ Trung tâm điều khiển chuyên dụng ngay lập tức mà không cần
Xác nhận cửa sổ hệ điều hành của Kitty. Các bản nháp không được áp dụng sẽ bị loại bỏ khi cửa sổ
được đóng lại theo cách này. Phần ghi đè nằm trong phạm vi cửa sổ `argvus-control-center`
lớp; thiết bị đầu cuối thông thường và các TUI khác vẫn duy trì chính sách xác nhận chặt chẽ của họ.

`SUPER + Alt + Space` chuyển đổi chế độ Xếp chồng/Nổi toàn cục, sử dụng cùng tùy chọn với thanh tác vụ và Trung tâm điều khiển. Lệnh tương đương là `argvus config window-mode toggle`. Các lời gọi đồng thời được tuần tự hóa; chuyển đổi sử dụng tùy chọn đã lưu ngay cả khi ứng dụng đang chờ xử lý. Phím tắt có thể được thay đổi hoặc vô hiệu hóa trong Phím tắt; `SUPER + Shift + Space` giữ hành động trên từng cửa sổ.