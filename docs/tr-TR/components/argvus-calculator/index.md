---
title: ARGVUS Hesap Makinesi
description: ARGVUS Hesap Makinesi — GTK4 / ARGVUS
---

# ARGVUS Hesap Makinesi

`argvus calculator` · launcher: **ARGVUS Hesap Makinesi**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **Bilimsel**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **Radyan**: İşaretli değil: derece.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **Temizle**; Backspace: **Karakter sil**; Tab / Shift+Tab.
- **Sonucu kopyala**; **Geçmiş**: 100; **Geçmişi temizle**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## Geçmiş

Geçmiş kapatıldıktan sonra saklanır (100 sonuç). Temizleme, kayıtlı verileri de siler. Ayarlar: gezinmek için ↑/↓, etkinleştirmek için Enter veya tıklama, dönmek için Escape.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

Her zaman ortalanmış küçük bir kayan pencere (370 × 420) olarak açılır; bilimsel mod büyütebilir.

Ana ekranda yalnızca ifade, sonuç ve tuş takımı bulunur. Menü bilimsel mod, açı birimi, kopyalama ve ayrı geçmiş ekranını sunar. Geri veya Escape hesaplamaya döner; geçmiş sonucu seçilince yeniden kullanılır. Temel mod ifadeyi koruyarak küçük pencereyi geri getirir.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

Pencere büyütülünce tuşlar büyür. Her yeni açılış varsayılan boyutu kullanır. İkincil ekranların üstünde geri oku vardır.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

Seçilen temel veya bilimsel mod hemen kaydedilir ve sonraki açılışta geri yüklenir.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
