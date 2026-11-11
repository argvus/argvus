---
title: Các câu hỏi thường gặp
description: Các câu hỏi thường gặp về ARGVUS, kiến trúc và cách sử dụng hàng ngày của nó.
---

Trang này trả lời các câu hỏi phổ biến về ARGVUS là gì, nó được phân phối như thế nào và nơi định cấu hình nó. Để biết các thủ tục chi tiết, hãy theo các liên kết trong mỗi câu trả lời.

##ARGVUS là gì?

ARGVUS là môi trường máy tính để bàn mô-đun dành cho Wayland và Arch Linux, được xây dựng dựa trên Hyprland. Nó không phải là môi trường máy tính để bàn độc lập với Hyprland; đó là trải nghiệm máy tính để bàn được lắp ráp dựa trên Hyprland và được điều chỉnh cho phù hợp với Arch Linux. ARGVUS điều phối phiên đồ họa, bề mặt shell, ứng dụng cài đặt, giao diện, dịch vụ máy tính để bàn và các công cụ tích hợp thông qua một bộ sưu tập các gói.

Nó không chỉ là cấu hình Hyprland hoặc tập hợp các dotfile cá nhân: nó cung cấp điểm vào phiên được hỗ trợ, cấu hình và trạng thái giao diện được chia sẻ cũng như quyền sở hữu được xác định đối với các thành phần như thanh tác vụ, Trung tâm điều khiển, thông báo, kết nối mạng và hiển thị. Xem phần [giới thiệu](/docs/introduction/) để biết tổng quan về cách các phần khớp với nhau.

## ARGVUS có miễn phí không?

Vâng. ARGVUS được tải xuống và sử dụng miễn phí, và các gói dự án cốt lõi của nó được phân phối dưới dạng phần mềm miễn phí theo[Giấy phép Công cộng GNU phiên bản 3](https://www.gnu.org/licenses/gpl-3.0.html). Điều này cho phép người dùng tự do chạy, nghiên cứu, sửa đổi và phân phối lại phần mềm được bao phủ theo các điều khoản của giấy phép.

ARGVUS được phân phối dưới dạng gói Arch Linux đã ký. Bản thân phần mềm không có giá mua nhưng các gói tải xuống vẫn có thể liên quan đến chi phí lưu trữ hoặc internet thông thường. Các thành phần và ứng dụng do bên thứ ba cung cấp có thể có giấy phép riêng.

## ARGVUS có phải là bản phân phối Linux không?

Không. ARGVUS là lớp môi trường máy tính để bàn được cài đặt trên Arch Linux. Arch Linux cung cấp hệ điều hành, kernel, trình quản lý gói và các gói cơ sở; Hyprland cung cấp thành phần Wayland; ARGVUS điều phối phiên máy tính để bàn và các thành phần tích hợp của nó.

Xem [cài đặt](/docs/getting-started/installation/) để biết mô hình cài đặt được hỗ trợ và [ARGVUS là gì—và không](/docs/introduction/#what-argvus-isand-is-not) để biết sự khác biệt về kiến ​​trúc.

## Tại sao lại là môi trường máy tính để bàn thay vì bản phân phối?

Việc giữ ARGVUS làm môi trường máy tính để bàn giúp người dùng có nhiều quyền kiểm soát hơn đối với phần còn lại của hệ thống. Một bản phân phối thường chọn và duy trì một đường cơ sở hoàn chỉnh, bao gồm kernel, lựa chọn gói, kho lưu trữ, mặc định và chính sách hệ thống. ARGVUS tập trung vào trải nghiệm trên máy tính để bàn trong khi để lại những quyết định rộng hơn cho người dùng và quá trình cài đặt Arch Linux cơ bản.

Mô hình này cũng giữ cho dự án theo mô-đun: quản lý phiên, cấu hình Hyprland, giao diện, thanh tác vụ, cài đặt và nhà cung cấp có thể được phát triển và đóng gói riêng biệt trong khi làm việc cùng nhau như một máy tính để bàn.

## Hệ thống nào được hỗ trợ?

Cơ sở được hỗ trợ hiện tại là Arch Linux. ARGVUS được đóng gói cho Arch Linux và phụ thuộc vào Wayland, Hyprland và các thành phần hệ thống được mô tả trong [yêu cầu cài đặt](/docs/getting-started/installation/requirements/).

Các bản phân phối khác có thể cung cấp một số thành phần ngược dòng tương tự, nhưng điều đó không làm cho bộ gói ARGVUS hoàn chỉnh hoặc tích hợp phiên được hỗ trợ ở đó.

## Làm cách nào để cài đặt ARGVUS?

Làm theo [hướng dẫn cài đặt](/docs/getting-started/installation/). Nó giải thích các yêu cầu, cách định cấu hình kho lưu trữ gói ARGVUS đã ký, cách cài đặt bộ gói và cách bắt đầu phiên đầu tiên.

ARGVUS được phân phối dưới dạng một bộ gói Arch phối hợp. Gói `argvus` là mục tiêu cài đặt chính và kéo theo các thành phần máy tính để bàn mà nó điều phối.

## Làm cách nào để thay đổi cài đặt ARGVUS?Sử dụng **Trung tâm điều khiển** để có cấu hình máy tính liên tục, bao gồm các chủ đề, hình nền, hiệu ứng, bố cục, phông chữ, bàn phím, phương thức nhập, ngôn ngữ, vùng và các ứng dụng mặc định. Các cách hiệu quả nhất để mở nó là phím tắt `SUPER + Alt + C` hoặc trình khởi chạy với `SUPER + D`: gõ **ARGVUS Control Center**. Bạn cũng có thể nhập `argvus` để liệt kê các ứng dụng ARGVUS.

Là một tùy chọn bổ sung, bạn có thể mở nó từ thiết bị đầu cuối bằng:

```sh
argvus --control-center
```

Bạn cũng có thể mở một vùng tập trung bằng `argvus control-center`, ví dụ `argvus control-center appearance themes`. Chạy `argvus control-center --help` để xem các tuyến có sẵn trong phiên bản đã cài đặt.

Sử dụng **Bảng điều khiển** để biết trạng thái hiện tại và các hành động thường xuyên như âm lượng, độ sáng, mạng, thông báo, điều khiển nguồn và phiên. Nó không phải là cơ sở dữ liệu cấu hình thứ hai. Xem [Nơi định cấu hình mọi thứ](/docs/user-guide/where-to-configure/) và [Trung tâm điều khiển](/docs/components/argvus-control-center/) để biết sự khác biệt.

## Cài đặt ARGVUS được lưu trữ ở đâu?

Cấu hình do người dùng sở hữu được lưu trữ bên dưới `$XDG_CONFIG_HOME/argvus` (thông thường là `~/.config/argvus`). Cấu hình logic chuẩn được duy trì ở đó, trong khi các tệp tiêu dùng dẫn xuất cho Hyprland, Waybar và tiện ích đo từ xa, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, thiết bị đầu cuối, phông chữ, hiệu ứng, lịch, đầu vào, bàn phím, nguồn và các thiết bị di động nằm dưới `~/.config/argvus/data/generated/`. Cây đó chỉ được viết bởi `argvus-config` và được xây dựng lại từ cấu hình chuẩn; `argvus-appearance` cam kết thay đổi giao diện và sau đó chỉ điều chỉnh những người tiêu dùng bên ngoài không thể tự đọc cấu hình chuẩn. Rofi là ngoại lệ: `argvus launcher` đóng gói cấu hình và chủ đề của nó, đồng thời các bản sao đã chỉnh sửa của người dùng nằm dưới `~/.config/argvus/data/rofi/`.

Không chỉnh sửa các tệp được tạo dưới dạng phương pháp cấu hình cố định. Sử dụng Trung tâm điều khiển hoặc đường dẫn ghi đè gốc được ghi lại cho thành phần bạn muốn tùy chỉnh. Xem [tệp cấu hình](/docs/reference/configuration-files/) và [cấu hình và trạng thái](/docs/developer-guide/architecture/configuration-and-state/).

## Tôi nên làm gì nếu thay đổi không được áp dụng?

Trước tiên hãy xác nhận rằng thay đổi đã được lưu hoặc áp dụng trên trang sở hữu. Sau đó kiểm tra xem thành phần bị ảnh hưởng có cần tải lại phiên hoặc dịch vụ hay không. Một thay đổi đã cam kết được `argvus-config` chiếu vào các tệp tiêu dùng, nhưng việc chiếu chúng không đảm bảo rằng mọi ứng dụng tiêu dùng đang chạy đều đã tải lại chúng. `argvus sessionctl reload` áp dụng thế hệ đã được cam kết từ tệp kê khai chiếu thay vì chiếu lại và bỏ qua quá trình tải lại đồ họa khi trạng thái chuẩn không thay đổi.

Sử dụng [hướng dẫn khắc phục sự cố](/docs/user-guide/troubleshooting/) có liên quan để cài đặt, khởi động phiên, đồ họa, màn hình, Hyprland, chủ đề và hình nền. Khi báo cáo sự cố, hãy bao gồm thành phần, cài đặt đã thay đổi, lệnh hoặc trang được sử dụng và đầu ra của trạng thái liên quan hoặc lệnh chẩn đoán.

## Tôi có thể tìm hiểu thêm ở đâu?

- [Cấu hình đầu tiên](/docs/user-guide/getting-started/) dành cho luồng phiên đầu tiên được đề xuất.
- [Hướng dẫn sử dụng](/docs/user-guide/) để biết hướng dẫn cụ thể về tính năng.
- [Tham khảo](/docs/reference/) cho các lệnh, đường dẫn, dịch vụ và hợp đồng cấu hình.
- [Hướng dẫn dành cho nhà phát triển](/docs/developer-guide/) về kiến ​​trúc, quyền sở hữu gói và quy trình đóng góp.