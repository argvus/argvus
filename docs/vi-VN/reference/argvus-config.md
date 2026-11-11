---
title: ARGVUS Configuration
description: Hợp đồng cấu hình JSON logic được ARGVUS sử dụng.
---

ARGVUS lưu giữ các tùy chọn của người dùng được quản lý trong các tệp JSON mô-đun trong `$XDG_CONFIG_HOME/argvus/config/` (thường là `~/.config/argvus/config/`). Chúng tạo thành mô hình logic chuẩn cho các tùy chọn do ARGVUS quản lý và `argvus-config` là thành phần duy nhất ghi các tệp bên dưới `data/generated/`. Các tệp gốc và tệp được tạo vẫn là các dự báo tương thích hoặc ghi đè dành riêng cho ứng dụng.

## Hướng dẫn sử dụng: Bắt đầu nhanh

Đối với hầu hết người dùng, việc cấu hình diễn ra thông qua Trung tâm điều khiển ARGVUS hoặc công cụ dòng lệnh này:

```sh
# View your configuration location and validate it
argvus config path
argvus config validate

# Get a specific value
argvus config get /appearance/theme
argvus config get /appearance/theme --raw    # Without JSON formatting

# Set a value (must be valid JSON)
argvus config set /appearance/theme '"gruvbox-dark"'

# Apply changes to reload the session
argvus config apply
```

Đối với quản trị viên và cấu hình nâng cao, hãy tiếp tục đọc phần bên dưới.

Người dùng gốc chỉ chứa tài liệu chuẩn và dữ liệu được quản lý:

```text
~/.config/argvus/
├── config.json
└── data/
```

Tệp được ghi nguyên tử, giữ riêng tư cho người dùng và được bảo vệ bởi `.config.lock`, bên ngoài cây `data/` có thể được hoán đổi trong quá trình chiếu. Phiên bản trước được lưu trữ tại `data/backups/config.json.bak`.

##Dòng lệnh

Lệnh con `argvus config` được cung cấp trong quá trình bởi gói `argvus`:

```sh
argvus config path
argvus config validate
argvus config ensure
argvus config migrate
argvus config status
argvus config explain /layout/gaps_out
argvus config get /appearance/theme --effective --raw
argvus config set /appearance/accent '"#61AFEF"'
argvus config patch appearance-patch.json
argvus config audio volume +5
argvus config audio mute toggle
argvus config apply-theme universe --variant sticky --accent '#EEEEEE' --gtk-mode dark
argvus config apply-theme universe-spaced --variant spaced --accent '#EEEEEE' --gtk-mode dark
argvus config theme-manifest universe-spaced
argvus config unset /appearance/accent
argvus config apply
argvus config rebuild
argvus config project
argvus config project --force
```

`ensure` hiện thực hóa `config.json` mới với các giá trị mặc định của lược đồ hoặc điền vào
các trường bị thiếu mà không thay thế các giá trị rõ ràng. Nó không thành công trên JSON không đúng định dạng.
`migrate` nhập trạng thái giao diện cũ hiện có mà không thay thế giá trị
đã có mặt trong `config.json`. `argvus sessionctl` chạy `recover`, `ensure`,
`migrate` và `project` trước khi áp dụng tải lại sau khi thay đổi cấu hình:

```sh
argvus sessionctl apply-config
```

`apply` và `rebuild` là bí danh để xây dựng lại mọi phép chiếu từ
`config.json`; `project` là cách viết lịch sử và chấp nhận `--force`.
`apply` cũng ủy quyền tải lại thời gian chạy sau cam kết cho `argvus sessionctl`.
Đặt `ARGVUS_NO_RUNTIME=1` để chuẩn bị hoặc kiểm tra riêng biệt.
Việc tạo ra xảy ra trong quá trình dàn dựng và sau cùng thì cây `data/` chỉ được hoán đổi
máy chiếu thành công. Do đó xóa `data/generated/` và chạy
`argvus config apply` là bản dựng lại có thể tái tạo mà không cần xuất bản một phần
chuỗi đầu ra Hyprland hoặc Thanh tác vụ. Các bản ghi lệnh bắt nguồn
phần băm và quyết định thời gian chạy trong
`$XDG_STATE_HOME/argvus/config-projection.json` (thông thường
`~/.local/state/argvus/config-projection.json`). Không phải bản kê khai cũng như
các tập tin được tạo ra là nguồn của sự thật. `apply` thực hiện tải lại sau cam kết
thông qua `argvus sessionctl`; `project` vẫn chỉ chiếu để đảm bảo tính tương thích.

Mọi đột biến liên tục đều sử dụng cùng một quy tắc: nếu tài liệu chuẩn thay đổi,
CLI gọi một `argvus sessionctl reload` tập trung sau khi xác nhận. các
kết quả tải lại được ghi lại trong `data/internal/last-reload.json`.

Do đó, `argvus sessionctl reload` là bước *áp dụng*, không phải là bước chiếu.
Khi tệp kê khai đến báo cáo đang chờ xử lý, nó sẽ khôi phục mục tiêu
người tiêu dùng được mô tả theo chương trình và không gọi lại `argvus config project`;
việc định hướng lại sẽ xây dựng lại một cây giống hệt nhau và loại bỏ kế hoạch đột biến
vừa sản xuất. Nó chỉ quay trở lại quá trình di chuyển và chiếu khi không có cam kết
kế hoạch tồn tại, ví dụ như trong khi khởi động phiên.

### Tải lại mục tiêu

`argvus config session-plan` trả lời việc tải lại các mục tiêu của
các phần đã thay đổi và `argvus sessionctl` chỉ chạy những phần đó. sự giật gân
chỉ bao gồm việc tải lại khi mục tiêu tạo lại một bề mặt: thanh Waybar hoặc
Bảng điều khiển di chuyển trở lại màn hình sẽ dịch chuyển mọi cửa sổ. Mọi thứ khác
tải lại tại chỗ.

| Thay đổi | Mục tiêu | giật gân |
| --- | --- | --- |
| Chủ đề, màu nổi bật hoặc chế độ GTK | `hyprland`, `consumers`, `wallpaper`, `polkit` | không |
| Chỉ hình nền | `wallpaper` (hyprpaper IPC; dịch vụ chỉ khởi động lại cho Shakebg) | không |
| `effects` (làm mờ, hình động, độ trong suốt, bóng) | `hyprland`, `consumers` | không || `layout` của cửa sổ (khoảng trống, đường viền) | `hyprland` | không |
| `layout.taskbar` (vị trí, lề) | `hyprland`, `taskbar`, `widget_telemetry` | vâng |
| `fonts` | `consumers` | không |
| `control_panel` (công tắc, card) | `control_panel` | vâng |
| `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays` | `hyprland` | không |
| `power` | `hypridle` | không |

`consumers` chạy `argvus config theme-runtime consumers` (bảng định kiểu thanh,
Bảng điều khiển qua IPC, màu Hyprland, tải lại chủ đề của Dunst, btop và
mèo con). Dunst trì hoãn việc tải lại chủ đề trong khi chế độ Không làm phiền đang hoạt động.
`wallpaper` chạy `argvus config theme-runtime wallpaper`, hiển thị
`/appearance/wallpaper` hiệu quả. Hàm băm `layout.taskbar` được giữ lại
ngoài cách bố trí cửa sổ trong bảng kê khai chiếu, do đó, một sự thay đổi về khoảng cách sẽ xảy ra
không khởi động lại các thanh. Gói `full` dự phòng, được sử dụng khi không có kế hoạch đã cam kết
tồn tại, khởi động lại mọi bề mặt dưới phần giật gân.

Mỗi cấu hình hiệu quả có một thế hệ hoạt động. Nó được lưu trữ trong
`data/internal/generation.json` và trong bảng kê khai chiếu bên dưới
`$XDG_STATE_HOME/argvus/config-projection.json`; nó không phải là một phần của dotfiles
cấu hình. Việc áp dụng lại cấu hình hiệu quả không thay đổi sẽ bảo tồn
thế hệ. `status` báo cáo liệu hàm băm của tệp kê khai, siêu dữ liệu tạo và
tất cả các tệp được tạo đều đồng ý; `apply` sửa chữa trình chiếu cũ hoặc chưa hoàn chỉnh.

`explain /path` đọc trực tiếp trình phân giải và in giá trị hiệu dụng,
nguồn, chủ đề và biến thể. Nó không phân tích các tệp được tạo hoặc tạo tệp thứ hai
quy tắc ưu tiên.

Phép chiếu chủ đề và hiệu ứng được triển khai trong chính `argvus-config`.
Giao dịch chủ đề (`argvus theme-switch`, chạy `theme-apply`), `argvus config accent` và `argvus effects-toggle` không chiếu:
họ xác thực, khóa, cam kết thông qua CLI này và sau đó
điều hòa những người tiêu dùng bên ngoài không thể đọc `config.json`. `argvus effects-toggle`
là một đại biểu mỏng không có đầu ra riêng. Những người giúp đỡ ngoại hình không bao giờ
được máy chiếu gọi và họ không bao giờ được viết bên dưới `data/generated/`.

### Bề mặt chiếu

`project_effective` hiển thị mọi người tiêu dùng thuộc sở hữu của ARGVUS trong một cây phân tầng
trước khi xuất bản:

| Bề mặt | Đầu ra |
| --- | --- |
| Hyprland | `generated/hypr/`, bao gồm hồ sơ chủ đề, `spaces-effective.conf`, `borders-effective.conf`, `hypridle.conf` và `monitors.lua` |
| Bố cục/hiệu ứng | `generated/effects/`, `generated/spaces.conf`, `generated/borders.conf` |
| Thanh cách | `generated/waybar/`, bao gồm `argvus-taskbar.css` và `argvus-widget-telemetry.{jsonc,css}` |
| Chủ đề từ xa widget | `generated/widget-telemetry/` và `generated/waybar/widget-telemetry-themes/` |
| QuickShell | `generated/quickshell/` |
| Thông báo | `generated/notifications/` và `dunstrc` |
| Màn hình khóa | Cấu hình chủ đề `generated/hypr/hyprlock.conf` và Hyprtoolkit |
| GTK | `generated/gtk/` |
| Yazi | `generated/yazi/` |
| Siêu tệp | `generated/superfile/` |
| Hồ sơ thiết bị đầu cuối và Qt | `generated/terminal/`, `generated/qt6ct/` |
| Phông chữ | `generated/fonts.conf` cộng với các khối phông chữ được quản lý trong bảng định kiểu Waybar |
| Thiết bị có thể tháo rời | `generated/removable-devices/theme.css` |
| Hình nền | `generated/hypr/hyprpaper.conf` và `data/hypr/hyprpaper.conf` |
| Bảng điều khiển và mặc định | `data/control-panel/`, `data/control-center/defaults.json` |
| Lịch, đầu vào, bàn phím, nguồn | lịch TOML, đoạn đầu vào, trạng thái bàn phím, điểm đánh dấu duy trì |

Lớp chủ đề cũng được phản chiếu bên ngoài thư mục gốc được tạo:
`data/waybar/argvus-taskbar.{jsonc,css}`,
`data/waybar/argvus-widget-telemetry.{jsonc,css}`, `data/foot/`,
`data/qt6ct/`, `data/hypr/hyprtoolkit.conf` và
`data/hypr/application-style.conf`. Những bản sao đó được thay thế từ gói
mặc định khi phần xuất hiện thay đổi và sau đó máy chiếu sẽ ghi lại
chỉ các khối được quản lý được phân tách của nó - khối phông chữ trong biểu định kiểu Waybar và
khối `ARGVUS_TELEMETRY_*` trong cấu hình đo từ xa — vì vậy văn bản bên ngoài
các điểm đánh dấu không tồn tại khi chuyển đổi chủ đề. `fonts.targets` và
`control_panel.widget_telemetry_blocks` vẫn là chuẩn: Trung tâm điều khiểnvẫn tồn tại chúng thông qua `argvus-config` và không bao giờ tự ghi các tệp này.

Lựa chọn khối đo từ xa của widget là hợp quy tại
`/control_panel/widget_telemetry_blocks`, một mảng gồm bảy tên khối
(`system`, `cpu_gpu`, `memory`, `storage`, `processes`, `network`, `keys`). các
Tệp `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` kế thừa chỉ được đọc dưới dạng
dự phòng di chuyển và không bao giờ được viết. `khối chuyển đổi tiện ích từ xa của argvus
set` and `blocks all` vá mảng chuẩn trong một lần ghi nguyên tử duy nhất và
`apply-state` cho rằng lựa chọn đã được người gọi của nó duy trì.

Khoảng cách Hyprland và các tùy chọn đường viền tuân theo cùng một ranh giới. các
các lệnh không gian cấu hình argvus và các lệnh viền cấu hình argvus (và lịch sử của chúng
trình bao bọc tương thích) cập nhật/bố cục; Rust tạo ra
data/generated/spaces-effect.conf và data/created/borders-effect.conf
và sở hữu xác nhận và ứng dụng hyprctl trực tiếp. .spaces và .borders là
trạng thái chỉ dành cho người di cư và không bao giờ là nguồn gốc của sự thật.

Trạng thái chế độ Accent và GTK sử dụng cùng một ranh giới Rust. Sử dụng `argvus-config
giọng COLOR` or `argvus cấu hình giọng --theme-default` để sở hữu giọng,
và `argvus config mode toggle|set dark|light` cho chế độ GTK. Lịch sử
tập lệnh giọng và chế độ là các trình bao bọc tương thích; `.accent-color`,
`.accent-custom` và `.gtk-mode` chỉ là các tệp di chuyển/chiếu.

Các đường dẫn ARGVUS logic cũng được giải quyết bằng `argvus config paths`. Di sản
Tệp `paths.sh` chỉ hiển thị các chức năng tương thích; ghi đè gốc, người dùng
dữ liệu, dữ liệu được tạo và dự phòng hệ thống được giải quyết bằng mô hình đường dẫn Rust.

`audio` thay đổi mức chìm đầu ra mặc định và ghi kết quả vào `/audio` trong một bước: `argvus config audio volume <N|+N|-N> [--limit PERCENT]` và `argvus config audio mute <on|off|toggle>`. Nó ghi vào kho lưu trữ mà không cần tải lại cấu hình vì PipeWire đã có giá trị. Mô-đun âm lượng Waybar, các phím phương tiện Hyprland và Bảng điều khiển sử dụng nó, vì vậy `argvus config project`, chiếu `/audio` lên bồn rửa, không bao giờ khôi phục ổ đĩa cũ hơn. Âm lượng trên 100% (phím Hyprland cho phép lên tới 150%) được lưu trữ dưới dạng 100.

`patch` áp dụng một đối tượng JSON có khóa là các con trỏ JSON trong một đối tượng bị khóa
thao tác đọc-sửa-ghi. `apply-theme` thực hiện nguyên tử tương đương
giao dịch chủ đề và thay thế các phần giao diện được quản lý từ
hồ sơ xuất hiện tùy chọn của chủ đề hoặc mặc định đóng gói.
Các lệnh đột biến xây dựng `NextUserConfig`, giải quyết `EffectiveConfig`, giai đoạn
tất cả các máy chiếu và chỉ sau đó xuất bản JSON chuẩn. Một hoạt động
Điểm đánh dấu `.config-transaction.json` cho phép `argvus config recover` chiếu lại
cấu hình trước đó sau sự cố giữa dữ liệu và xuất bản JSON.
Giá trị mặc định của chủ đề được khai báo theo bảng kê khai của mỗi chủ đề
(`themes/<theme>/theme.toml`): dấu mặc định (`palette.accent`), bảng màu GTK
và hình nền. Các tệp CSS, Lua, Rasi và Qt được hiển thị từ nó trong bản dựng chủ đề.

Thay đổi chủ đề, bao gồm Sáng/Tối, đặt lại màu đánh dấu về mặc định của chủ đề,
ngay cả khi người dùng chọn màu tùy chỉnh: màu tùy chỉnh chỉ được giữ cho đến chủ đề tiếp theo
thay đổi. Sử dụng `argvus config accent` sau khi thay đổi chủ đề để thiết lập lại. Áp dụng một
chủ đề với `argvus config theme-apply` viết `/appearance/accent` và `/appearance/gtk_mode`
từ bảng kê khai và `argvus config accent` ghi màu đã chọn bằng `accent_custom`
thiết lập, trong một lệnh.

`apply-theme` lấy biến thể rõ ràng là `--variant sticky|spaced` và
giải quyết tên được yêu cầu thông qua bảng kê khai, vì vậy một yêu cầu không bao giờ có thể
giải quyết nội dung của biến thể khác. `theme-manifest THEME` cũng vậy
độ phân giải hiển thị chỉ đọc: nó in mục nhập bảng kê khai cùng với
`asset_id` và `layout_variant` được giải quyết, đó là điều người gọi cần làm
giải quyết các tài sản được đóng gói. Sticky là id không có hậu tố và chỉ Spaced mang mộtHậu tố `-spaced`, do đó `dracula-sticky` và `argvus-dark-dracula-sticky` là
được chấp nhận và chuẩn hóa thành `dracula`.

Với `--raw`, các giá trị chuỗi được in mà không có dấu ngoặc kép JSON, trong khi các đối tượng và mảng vẫn giữ nguyên giá trị JSON. Các tệp dấu chấm chủ đề và bố cục không còn được trình trợ giúp chiếu viết nữa; `config.json` vẫn là nguồn liên tục duy nhất cho các tùy chọn đó. Nếu trình chiếu không thành công, quá trình tải lại phiên sẽ dừng trước khi khởi động lại các bề mặt màn hình để một phần chủ đề cũ hơn không được hiển thị như hiện tại.

## Xuất và nhập

Mô hình logic tương tự được sử dụng để trao đổi hồ sơ:

```sh
argvus config export --scope appearance --output appearance.json
argvus config import --scope appearance appearance.json
```

Sử dụng `--scope desktop` cho mẫu máy tính để bàn được quản lý hoàn chỉnh. Các trường bị thiếu sử dụng giá trị mặc định của ARGVUS; đầu ra được tạo ra không nên được chỉnh sửa trực tiếp.

Xuất và nhập chủ đề của Trung tâm điều khiển trao đổi một chủ đề và hồ sơ trực quan của nó: kho lưu trữ chứa thư mục chủ đề lược đồ 2 (`<id>/theme.toml`, `appearance.json` tùy chọn và `effects.conf`, `overrides/`, hình nền) cài đặt trong `$XDG_DATA_HOME/argvus/themes/`. Cấu hình chứa hình thức, hình học bố cục, hiệu ứng hình ảnh, phông chữ và bản trình bày bảng điều khiển. Âm thanh, bàn phím, mạng và các tùy chọn không trực quan khác vẫn nằm ngoài chủ đề. Sử dụng phạm vi `appearance` ở trên để di chuyển cấu hình giao diện hoàn chỉnh và phạm vi `desktop` để sao lưu cấu hình hoàn chỉnh.

Các tùy chọn nhóm lược đồ hiện tại theo `appearance`, `layout`, `effects`, `fonts`, `control_panel`, `default_apps`, `keyboard_shortcuts`, `hyprland`, `displays`, `power`, `session`, `calendar` và `removable_devices`. `default_apps` là nguồn liên tục duy nhất cho các ứng dụng ưa thích và `keyboard_shortcuts` lưu trữ các khóa tiếng Anh ổn định, độc lập với ngôn ngữ với các giá trị chuỗi hoặc `null` cho các liên kết bị vô hiệu hóa.

Trung tâm điều khiển đọc các giá trị giao diện chuẩn khi mở trang Giao diện và ghi các thay đổi thông qua `argvus-config`. Các tệp trạng thái gốc hiện có vẫn là các bản chiếu cho các tập lệnh xuất hiện và các bản cài đặt cũ hơn.

Bảng điều khiển sử dụng `control_panel.cards` để sắp xếp và hiển thị thẻ cũng như `control_panel.enabled` cho công tắc chính của bảng điều khiển. Các tệp `control-panel/cards.json` và `state/control-panel` của nó được lưu giữ dưới dạng các phép chiếu tương thích. Do đó, những thay đổi được thực hiện trong Bảng điều khiển là một phần của phạm vi xuất `desktop`.

Tùy chọn phông chữ sử dụng `fonts.targets` cho dòng, kiểu và kích thước của thanh tác vụ, đo từ xa, Bảng điều khiển, hệ thống, ứng dụng, thiết bị đầu cuối và mục tiêu trình duyệt. `fonts.rendering` lưu trữ các tùy chọn khử răng cưa, gợi ý, thứ tự pixel phụ và tùy chọn DPI. `fonts.conf`, quy tắc cấu hình phông chữ, cài đặt màn hình và các khối phông chữ được quản lý bên trong biểu định kiểu Waybar là các phép chiếu: Trung tâm điều khiển vẫn duy trì `/fonts/targets/*` đến `argvus-config` và máy chiếu chỉ ghi lại khối được phân tách của nó.

Tùy chọn Hyprland sử dụng `keyboard_shortcuts` cho các phím tắt, `hyprland.input` cho cài đặt chuột và bàn di chuột, và `hyprland.keyboard` cho bố cục, biến thể và tùy chọn XKB. ID liên kết tệp kê khai ánh xạ tới `config_key` ổn định (ví dụ: `window.close` → `close_window` và `window.drag_mouse` → `drag_window__floating_window_only`). Tùy chọn hiển thị sử dụng `displays.monitors`, `displays.workspaces` và `displays.primary_monitor`. Các tệp TOML gốc và các tệp Lua được tạo trong `data/generated/` vẫn là các dự đoán được Hyprland sử dụng.Mọi công tắc `effects` đều được bật theo mặc định ở chế độ bật (hoạt ảnh, làm mờ, đo từ xa tiện ích và từng bề mặt trong suốt), trên các máy ảo và cả các phiên sử dụng năng lượng thấp; bóng cửa sổ (`shadow_enabled`) và lắc lư (`wobble_enabled`) mặc định tắt. Tùy chọn hiệu ứng sử dụng `effects` cho công tắc chính Ảnh động (`animations`, được áp dụng cho `hyprland.lua` từ `hypr/state.lua` được chiếu, độc lập với công tắc Hiệu ứng, tức là`blur_global_enabled` cộng với `transparency_<surface>_enabled` của `taskbar`, `control-panel`, `widget-telemetry`, `terminal` và `launchers` và đọc là được kích hoạt khi bất kỳ trong số chúng được kích hoạt; `argvus effects-toggle animations|effects` chuyển đổi chúng), công tắc làm mờ toàn cầu (`blur_global_enabled`), thông số làm mờ Hyprland (`blur_size`, `blur_passes`, `blur_brightness`, `blur_noise`, `blur_contrast`, `blur_vibrancy`, `blur_vibrancy_darkness`), bóng cửa sổ(`shadow_enabled`,`shadow_sharp`, `shadow_color`, `shadow_range`, `shadow_render_power`, `shadow_offset_x`, `shadow_offset_y`, `shadow_scale`), các giá trị Ảnh động (`animation_<leaf>_speed`, `_curve`, `_style`, `_popin` và `_enabled` cho mỗi hoạt ảnh, cùng với `animation_bezier_x1` đến `animation_bezier_y2` và `animation_spring_mass`, `animation_spring_stiffness` và `animation_spring_dampening` cho các đường cong tùy chỉnh) và các giá trị độ trong suốt trên mỗi bề mặt. Các hiệu ứng thuộc sở hữu của chủ đề, không thuộc sở hữu của biến thể: Sticky và Spaced sử dụng cùng một phép chiếu hiệu ứng `<theme>.conf` trong khi các phép chiếu bố cục của chúng khác nhau. Các bề mặt hiện bao gồm `taskbar`, `control-panel` và `widget-telemetry`; giá trị trong suốt là tỷ lệ phần trăm từ `0` đến `100` (Trung tâm điều khiển và các bề mặt trình khởi chạy mặc định là `15`, các giao diện khác là `50`), tham số độ mờ là các số trong phạm vikhai báo ở`schema.json` và các trạng thái được bật là boolean. Các giá trị hoạt ảnh cũng được xác thực bởi `argvus-config`: tốc độ là các số từ `1` đến `10` (cho phép số thập phân), tỷ lệ phần trăm bật vào là số nguyên từ `1` đến `100`, các đường cong phải là tên từ trang Hoạt ảnh, các kiểu phải là tên từ trang Hoạt ảnh và được lá riêng của hoạt ảnh chấp nhận (ví dụ: `popin` bị từ chối đối với `animation_workspace_in_style`, `gnomed` chỉ được chấp nhận đối với việc mở và đóng cửa sổ và các biến thể trang trình bày bị từ chối trên các lá cửa sổ và lớp), các điểm Bézier Y nằm trong khoảng từ `-1` đến `2`, phạm vi phân tích Hyprland 0.56.2 và các tham số đường cong khác nằm trong phạm vi được ghi lại của chúng. Các lá `layers` và `switching` không còn tồn tại nên các cấu hình được lưu trước khi xóa sẽ mất các khóa `animation_layers_enabled`, `animation_layers_speed`, `animation_layers_curve` và `animation_switching_*` khi chúng tải. Các kiểu bên ngoài tập hợp hiện tại sẽ bị loại bỏ và các điểm Bézier Y nằm ngoài `-1` đến `2` được kẹp lại, do đó, các giá trị mặc định được đóng gói sẽ áp dụng lại cho bất kỳ kiểu nào đã bị xóa.Tùy chọn nguồn điện sử dụng `power.screen_off_minutes`, `power.lock_minutes`, `power.keep_awake` và các chính sách về nắp/nút nguồn. Các dạng được chiếu là `data/generated/hypr/hypridle.conf` và điểm đánh dấu `data/power/keep-awake`; `.keep-awake` chỉ là đầu vào di chuyển. `session.language` đã ngừng hoạt động: các giá trị hiện có bị bỏ qua, các lần ghi mới bị từ chối và không có tệp tương thích ngôn ngữ nào được chiếu. Thay vào đó, hãy đặt `LANG` trong người dùng `locale.conf`. Tùy chọn lịch sử dụng `calendar` và được chiếu tới TOML của người dùng lịch trên thanh tác vụ. Tùy chọn thiết bị di động sử dụng `removable_devices`; ứng dụng thiết bị di động coi phần này là phần ghi đè của người dùng đối với các giá trị mặc định được đóng gói.`migrate` nhập các tệp và phần gốc cũ một lần, duy trì giá trị chuẩn hiện có. `defaults.json` kế thừa, `/defaults`, `/hyprland/keybindings`, `keybindings.toml`, tệp chấm, `generated/` và các thư mục thành phần được di chuyển xuống dưới `data/`; tải lại nhiều lần không tạo lại bố cục gốc cũ. Việc chọn lại chủ đề hiện tại là bình thường: nếu trạng thái chuẩn và các dự đoán đã đồng ý, kế hoạch chiếu sẽ báo cáo không có phần nào bị thay đổi và phiên bỏ qua phần giới thiệu và khởi động lại dịch vụ.

Nguồn điện tuân theo cùng một hợp đồng chiếu: Trung tâm điều khiển ghi `/power` vào `config.json`, `argvus config project` hiển thị nguyên tử `data/hypr/hypridle.conf` và chỉ khi đó `argvus sessionctl` mới có thể khởi động lại `argvus-hypridle.service`. Việc xóa tệp đó có thể phục hồi được bằng `argvus config project`; giá trị `0` bỏ qua thời gian chờ tương ứng và có nghĩa là Không bao giờ.

Giao diện màn hình khóa được Rust chiếu từ chủ đề, điểm nhấn,
hiệu ứng, hình nền và trạng thái phông chữ vào `data/hypr/hyprlock.conf`. Lịch sử
Điểm vào `hyprlock-theme.sh` chỉ dành cho khả năng tương thích và ủy quyền cho
`argvus config lock apply`; `--invalidate` chỉ vô hiệu hóa tính năng làm mờ dẫn xuất
bộ nhớ đệm hình nền.

Thay đổi thời gian chờ nguồn sử dụng `/power/lock_minutes` và
`/power/screen_off_minutes` là nguồn sự thật của họ. `argvus config project`
sử dụng lại trình kết xuất Hypridle hiện có và điểm vào `idle-timeout.sh` là
chỉ có một bộ chuyển đổi tương thích; khởi động lại dịch vụ vẫn được ủy quyền cho
`argvus sessionctl`.

Giao diện thông báo bắt nguồn từ chủ đề, giọng điệu và
hiệu ứng vào `data/notifications/dunstrc`. Thông báo lịch sử
Tệp `theme.sh` chỉ hiển thị các chức năng tương thích gọi
`argvus config notifications`; nó không phân tích cú pháp hoặc thay đổi cấu hình Dunst.

Các bộ điều hợp hướng phần cứng như phát hiện màn hình và điều khiển Bluetooth
hiện được nhập bởi miền hiển thị Rust: `nwg-displays` là nhà nhập khẩu bên ngoài,
trong khi `/displays` và `data/generated/hypr/monitors.lua` vẫn hợp quy và
trạng thái dẫn xuất tương ứng. Các công cụ hiển thị thay đổi bố cục với
`argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` và `monitor-remove`; các lệnh hiển thị là `argvus config display status|list-modes|set|apply|apply-nwg|migrate-nwg|nwg-import|nwg-apply`. Tệp Hyprland được chiếu từ `/displays`; không có tập lệnh shell nào viết nó nữa. Lựa chọn hình nền, chế độ nhóm tiện ích trên thanh tác vụ,
và tra cứu ứng dụng mặc định cũng sử dụng API `argvus-config`; Vỏ lịch sử của họ
điểm vào là giao diện người dùng hoặc trình bao bọc tương thích. Keep Awake là một ứng dụng bền bỉ
Tùy chọn `/power/keep_awake` được chiếu tới điểm đánh dấu `data/power/keep-awake`,
trong khi lock-DPMS là trạng thái thời gian chạy phiên trong
`$XDG_CACHE_HOME/argvus/lock-dpms`. Bố cục bàn phím hiện thuộc sở hữu của Rust
Trình trợ giúp `argvus-keyboard-layout`: thứ tự được định cấu hình đến từ trình trợ giúp được tạo
Phép chiếu đầu vào Hyprland hoặc dự phòng ngôn ngữ, trong khi bố cục hoạt động vẫn được giữ nguyên
trạng thái thời gian chạy phiên được phát hiện từ Hyprland. Điểm vào chu kỳ và
daemon bố cục bàn phím chỉ là các trình bao bọc tương thích; dịch vụ người dùng chạy
daemon Rust trực tiếp và `argvus sessionctl keyboard-layout cycle` là
lệnh ổn định cho người gọi.

Trạng thái thời gian chạy Bluetooth tuân theo ranh giới bộ điều hợp bên ngoài nhưng không phải là một phần
của `argvus-config`: `argvus bluetooth` sở hữu khả năng phân tích cú pháp và
xác thực xung quanh `bluetoothctl`, số lần thử lại có giới hạn, kiểm tra dịch vụ hệ thống và
Trạng thái RFKill. `argvus bluetoothctl` là điểm vào. Bộ chuyển đổi nguồn, thiết bị được ghép nối và thiết bị được kết nối
duy trì trạng thái thời gian chạy BlueZ thay vì trạng thái JSON chuẩn; phần cứng trực tiếp
Xác thực Bluetooth phụ thuộc vào môi trường.
## Nhận dạng và đối chiếu thời gian chạy

Sử dụng `argvus config status --verbose` để kiểm tra nhị phân chính xác, xây dựng
danh tính, gốc cấu hình, gốc được tạo, gốc nội dung, nguồn kê khai và
thế hệ năng động. Nó báo cáo `synchronized`, `stale`, `incomplete`, `legacy`
và trạng thái chiếu `pending-transaction`.

`argvus config --version` báo cáo phiên bản gói, bản sửa đổi bản dựng khi được cung cấpbởi môi trường xây dựng và hồ sơ. Các thử nghiệm phát triển phải tiêm
nhị phân và tài sản một cách rõ ràng; họ không được âm thầm sử dụng `argvus config`
hoặc `~/.config/argvus`. `apply` bình thường hóa một cách an toàn các số nhận dạng kết hợp cũ
chẳng hạn như `argvus-dark-spaced` thành các giá trị chủ đề và biến thể riêng biệt trước
xây dựng lại các dự báo. Danh tính tài sản đóng gói không được ghi lại trong bảng kê khai trong bản phát hành này.

## Ánh sáng ban đêm

Mở **Máy tính để bàn & Đầu vào > Hiển thị > Đèn ngủ** hoặc chạy `argvus control-center displays night-light`. Định cấu hình nhiệt độ, kích hoạt thủ công và lịch trình cục bộ cố định thông qua `argvus config display night-light`. Xem [Ánh sáng ban đêm](/docs/user-guide/night-light/) để biết các yêu cầu về lệnh, trạng thái và thời gian chạy.

### Cài đặt lắc lư (Hyprland 0.57+)

`effects.wobble_enabled`, `wobble_stiffness`, `wobble_damping`, `wobble_mass`,
Dự án `wobble_intensity` và `wobble_mesh` trực tiếp vào `hypr/state.lua`.
Sử dụng `argvus effects-toggle wobble-settings get` để đọc chúng, hoặc
`argvus effects-toggle wobble-settings set enabled=enabled stiffness=100 damping=10 mass=1 intensity=1 mesh=8`
để lưu và yêu cầu tải lại. `enabled=disabled` tắt hiệu ứng.

Trung tâm điều khiển cũng chia sẻ bản nháp Ảnh động: `animations-settings get`
trả về các mục nhập `wobble_*` và `animations-settings set` chấp nhận các tên đầy đủ này
cùng với các đối số hoạt ảnh hiện có của nó. Điều này duy trì toàn bộ dự thảo trong một
bản vá được xác thực và yêu cầu tải lại một lần. Ví dụ:

```sh
argvus effects-toggle animations-settings set enabled=enabled wobble_enabled=true wobble_intensity=0 wobble_mesh=8
```

`enabled` toàn cầu điều khiển hoạt ảnh; `wobble_enabled` độc lập
ưu tiên. Việc tắt hoạt ảnh toàn cục sẽ ngăn chặn tình trạng lắc lư trong Lua mà không xóa
sở thích đó. Giá trị không hợp lệ sẽ từ chối toàn bộ bản vá. Nhận chìa khóa bị thiếu
ARGVUS mặc định mà không thay thế các lựa chọn hiện có. Xem [Diện mạo](/docs/user-guide/appearance/#window-wobble)
cho giới hạn và mặc định.


### Phím tắt do người dùng tạo

`/keyboard_shortcuts` giữ lại các giá trị chuỗi hoặc null hiện có cho các hành động kê khai chính thức. Các phím tắt chung tùy chỉnh sử dụng một đối tượng `/hyprland/custom_shortcuts` riêng biệt, được lưu trữ trong `config/hyprland.json`. ID `custom_` ổn định của nó độc lập với mô tả và ngôn ngữ. Mỗi bản ghi có `description`, `keys`, `enabled` và `action` được đánh máy:

```json
{
  "custom_editor": {
    "description": "Open my editor",
    "keys": "SUPER + CTRL + E",
    "enabled": true,
    "action": { "type": "command", "command": "editor --new-window" }
  },
  "custom_float": {
    "description": "Toggle floating",
    "keys": "SUPER + CTRL + F",
    "enabled": false,
    "action": {
      "type": "dispatcher",
      "dispatcher": "window.float",
      "arguments": { "action": "toggle" }
    }
  }
}
```

Ví dụ là giá trị của `/hyprland/custom_shortcuts`, không phải là tài liệu cấu hình hoàn chỉnh. Tên người điều phối đến từ các nhà máy điều phối Lua được hỗ trợ; đối số là một đối tượng JSON tùy chọn hoặc `null`, mã không bao giờ được đánh giá. Các phím tắt tùy chỉnh không có sơ đồ con, ràng buộc chuột hoặc cờ nâng cao. Phép chiếu ghi `data/generated/hypr/custom-shortcuts.lua` dưới dạng bảng dữ liệu thoát, bao gồm một bảng trống khi bản ghi cuối cùng bị xóa. Cả phép chiếu đầy đủ và `project --only keyboard_shortcuts` đều tạo ra nó. Hyprland đăng ký các mục được kích hoạt bằng `hl.bind`; tập tin tùy chọn và các lỗi đăng ký riêng lẻ được chứa và ghi lại.

Trung tâm điều khiển tạo/chỉnh sửa bản ghi và vô hiệu hóa các xung đột đã được xác nhận bằng cách sử dụng một giao dịch `argvus config patch`. Nó lưu trước khi áp dụng thông qua dịch vụ cấu hình hiện có. Việc áp dụng không thành công sẽ không hoàn tác việc lưu thành công. Việc khôi phục các giá trị mặc định chính thức sẽ giữ nguyên các bản ghi tùy chỉnh và bị từ chối nếu nó xung đột với một phím tắt tùy chỉnh đang hoạt động. Mô tả của người dùng vẫn là văn bản bằng chữ ở tất cả các ngôn ngữ và bảng mã được tạo hiệu quả.

## Chế độ cửa sổ toàn cầu

Mở **Trung tâm điều khiển → Màn hình nền & Đầu vào → Hyprland → Windows → Chế độ** và chọn **Cửa sổ xếp chồng (Mặc định)** hoặc **Cửa sổ nổi**. Bộ chọn tương tự có sẵn từ biểu tượng chế độ cửa sổ trong nhóm tiện ích trên thanh tác vụ.

Tùy chọn này áp dụng cho mọi màn hình và không gian làm việc và được ghi nhớ sau khi đăng xuất. Nổi chuyển đổi các cửa sổ ứng dụng hiện có và các cửa sổ mới mở. Ngói chỉ khôi phục các cửa sổ mà chính sách này đã chuyển đổi; các hộp thoại và các cửa sổ khác vốn đã nổi vẫn nổi. Một cửa sổ được ghim rõ ràng trong khi Nổi đang hoạt động cũng vẫn nổi để bảo toàn mã pin của nó. Bố cục lát gạch được tính toán lại; cây phân chia chính xác trước đó của nó không được khôi phục.`SUPER + SHIFT + Space` giữ trạng thái bình thường trên mỗi cửa sổ trong Ngói. Trong chế độ Nổi, nó không chuyển cửa sổ về chế độ xếp lát. Các bảng, thanh tác vụ và thông báo là các bề mặt lớp vỏ và không bị ảnh hưởng. Toàn màn hình và tối đa hóa tiếp tục sử dụng hoạt động bình thường của Hyprland.

Tùy chọn chuẩn là `/hyprland/window_mode` trong `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. Phép chiếu được tạo ra của nó là `data/generated/hypr/window-mode.lua`; không chỉnh sửa nó. Sổ đăng ký khôi phục trên mỗi cửa sổ nằm trong `$XDG_RUNTIME_DIR/argvus`, nằm trong phạm vi phiên bản bộ tổng hợp và vẫn tồn tại khi tải lại cấu hình. Nó không được sử dụng lại trong một phiên mới.

Nếu ứng dụng đang chờ xử lý hoặc bị lỗi, giao diện sẽ phân biệt ứng dụng này với tùy chọn đã lưu. Chọn lại tùy chọn để thử lại. Nếu không có bộ tổng hợp, tùy chọn sẽ được lưu cho lần đăng nhập tiếp theo.

```sh
argvus config window-mode get
argvus config window-mode status --json
argvus config window-mode set floating
argvus config window-mode set tile
argvus config window-mode toggle
```

## Nút cửa sổ và thanh ứng dụng

Chế độ nổi thêm thanh tiêu đề ARGVUS với **Thu nhỏ**, **Tối đa hóa / Khôi phục** và **Đóng** đối với các cửa sổ ứng dụng thông thường. Bấm đúp vào thanh tiêu đề sẽ chuyển đổi tối đa hóa. Trang trí bổ sung được ẩn trong Ngói và toàn màn hình. Các ứng dụng tự vẽ các điều khiển có thể hiển thị cả hai thanh; Các điều khiển ARGVUS thống nhất là có chủ ý.

Thanh Waybar thứ hai xuất hiện trên mọi màn hình, ở cạnh đối diện với thanh tác vụ chính. Nó chỉ liệt kê các cửa sổ thuộc không gian làm việc hiện tại của màn hình đó, bao gồm các cửa sổ được thu nhỏ từ không gian làm việc đó. Một không gian làm việc đặc biệt mở được ưu tiên. Các cửa sổ được ghim vẫn hiển thị; các tab nhóm ẩn không tạo thêm mục nhập và nhóm thu nhỏ có một mục khôi phục. Các tab bên trong ứng dụng không được liệt kê riêng biệt. Các mục được thu nhỏ vẫn giữ lại liên kết màn hình/không gian làm việc ban đầu của chúng. Nhấp vào cửa sổ không hoạt động để kích hoạt, cửa sổ đang hoạt động để thu nhỏ cửa sổ hoặc cửa sổ thu nhỏ để khôi phục cửa sổ đó; menu ngữ cảnh của nó cung cấp khả năng khôi phục, thu nhỏ, tối đa hóa/khôi phục và đóng. Các biểu tượng ứng dụng không xác định sẽ quay trở lại biểu tượng cửa sổ. Kích hoạt hoặc khôi phục một mục sẽ giữ nguyên vị trí con trỏ. Thanh phụ sử dụng CSS chính hiệu quả (màu sắc, phông chữ, đường viền và làm tròn), với chiều cao và lề phản chiếu ở cạnh đối diện.

`SUPER + Alt + Up` đặt thanh chính ở trên cùng và thanh cửa sổ ở dưới cùng. `SUPER + Alt + Down` thì ngược lại. Việc lặp lại một hướng sẽ khiến các vị trí không thay đổi. Ngói chỉ có thanh chính. Việc quay lại Ngói sẽ khôi phục tất cả các cửa sổ được thu nhỏ trước khi xóa thanh cửa sổ, sau đó giữ nguyên chính sách xếp kề/nổi ban đầu.

Giảm thiểu sử dụng không gian làm việc `special:argvus-minimized` dành riêng. Không sử dụng không gian làm việc đó một cách thủ công. Quy trình `argvus windows service` ghi lại danh tính, không gian làm việc/màn hình ban đầu, hình học, toàn màn hình/phóng to và trạng thái pin trong sổ đăng ký thời gian chạy dành riêng cho phiên bản tổng hợp trong `$XDG_RUNTIME_DIR/argvus`. Các bản ghi vẫn tồn tại sau khi khởi động lại dịch vụ hoặc tải lại cấu hình nhưng không vượt qua các phiên tổng hợp. Đóng một cửa sổ sẽ xóa bản ghi của nó. Nếu màn hình biến mất, quá trình khôi phục sẽ sử dụng đầu ra có sẵn. Thanh khôi phục bị thiếu hoặc bị lỗi sẽ ngăn cản việc giảm thiểu mới; việc khôi phục không thành công sẽ giữ lại bản ghi và thanh khôi phục.

Phiên sở hữu `argvus-windows.service` và `argvus-window-bar.service`. Cái sau giữ con Waybar của nó không hoạt động trong Ngói. Plugin hyprbars được đóng gói bởi `argvus-hyprland`, được xây dựng cho Hyprland 0.57.0 ABI trên x86_64; nó không được biên dịch trong thư mục chính của người dùng. Xây dựng lại plugin bằng bản cập nhật ABI tổng hợp.

Các lệnh: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` và `minimize|restore|activate|maximize|close ADDRESS`. Các địa chỉ cửa sổ đến từ danh sách. `position top --persist` lưu trữ/dự án tùy chọn mà không cần khởi động lại thanh chính. Chi tiết lỗi về cấu hình và thời gian chạy được ghi vào nhật ký; lỗi hành động hiển thị cũng hiển thị thông báo đã dịch.Trong **Dải cách + Nổi**, cửa sổ tôn trọng các khoảng trống bên ngoài và các khu vực được cả hai thanh dành riêng. Các giới hạn bao gồm đường viền và tiêu đề, nằm bên trong cùng một đường viền cửa sổ. Tối đa hóa thông qua các điều khiển ARGVUS sẽ lấp đầy khu vực này bằng lề; khôi phục sẽ khôi phục hình dạng trước đó, kể cả sau khi khởi động lại dịch vụ. Toàn màn hình vẫn sử dụng toàn bộ đầu ra. Cửa sổ nổi có thể chồng lên nhau; ốp lát những khoảng trống bên trong không ngăn cách chúng.

Lề bên trong dành riêng cho các thanh không được tính hai lần ở các khoảng trống bên ngoài cửa sổ. Với khoảng cách đối xứng, khoảng cách từ cửa sổ đến thanh hiển thị bằng nhau ở trên và dưới, kể cả sau khi hoán đổi các thanh.

`SUPER + Alt + Space` chuyển đổi chế độ Ngói/Nổi chung, sử dụng cùng tùy chọn với thanh tác vụ và Trung tâm điều khiển. Lệnh tương đương là `argvus config window-mode toggle`. Các lời gọi đồng thời được tuần tự hóa; chuyển đổi sử dụng tùy chọn đã lưu ngay cả khi ứng dụng đang chờ xử lý. Phím tắt có thể được thay đổi hoặc vô hiệu hóa trong Phím tắt; `SUPER + Shift + Space` giữ hành động trên từng cửa sổ.