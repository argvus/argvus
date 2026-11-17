---
title: ARGVUS Hesap Makinesi
description: Temel · Bilimsel · Programlama · Finansal · Birim dönüştürme
---

# ARGVUS Hesap Makinesi

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Ayarlar → Profil bölümünde ↑/↓ ve Enter ile veya satıra tıklayarak seçim yapın. Profil ve seçenekler hemen kaydedilir ve sonraki açılışta geri yüklenir. Her profil oturum boyunca kendi ifadesini korur. Enter veya = ifadeyi seçmeden sonuçla değiştirir ve imleci sona koyar.

Oranlar dönem başına yüzdedir; oran, sayı ve ödeme için aynı dönemi kullanın. Ödemeler dönem sonunda yapılır ve sıfır oran desteklenir. Basit ve bileşik faiz birikmiş toplamı verir. Dönüştürme çevrimdışıdır, para birimi içermez. Geçmiş profil, seçenek ve finansal girdilerle 100 kayıt tutar; sonuç seçimi bağlamı geri yükler. Temizleme kayıtlı dosyayı da boşaltır.

## Ayarlar

- `s Ayarlar` → **Profil** → Temel / Bilimsel / Programlama / Finansal / Birim dönüştürme.
- ↑ / ↓ · Enter · Escape / ←.
- **Sonucu kopyala** · **Geçmiş** · **Geçmişi temizle**.
- **Temel**: 370 × 420; **Bilimsel**: 440 × 640.
- **Programlama**: 520 × 640; **Finansal**: 440 × 640; **Birim dönüştürme**: 440 × 520.

## Temel / Bilimsel

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Açı birimi**: Derece / Radyan. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 anlamlı basamak · 4096 bayt · 64 ayrıştırıcı düzeyi.

## Programlama

Varsayılan: ondalık, işaretli 64 bit. Aritmetik sözcük genişliğinde sarar; işaretli sağa kaydırma işareti genişletir. Tamsayı bölmesi sıfıra doğru kesilir. Sıfıra bölme ve [0, bits) dışı kaydırmalar hatadır. Öneksiz sayılar seçili tabanı kullanır. Taban değişimi geçerli ifadeyi dönüştürür; genişlik veya işaret aynı bitleri yeniden yorumlar.

- **Taban**: 2 / 8 / 10 / 16; **Sözcük boyutu (bit)**: 8 / 16 / 32 / 64.
- **İşaretli** / **İşaretsiz** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (İşaretli); `-7/2 = -3`.

## Finansal

Basit faiz · Bileşik faiz · Sabit taksit · Bugünkü değer · Gelecekteki değer · Yüzde değişimi.

- `P`: Başlangıç anaparası; `FV`: Gelecekteki tutar; `A`: Dönem başına ödeme.
- `r`: Dönem başına oran (%) / 100; `n`: Dönem sayısı.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Basit faiz: `P*(1+r*n)`; Bileşik faiz: `P*G`.
- Sabit taksit: `P*G/H`; Bugünkü değer: `(FV+A*H)/G`; Gelecekteki değer: `P*G+A*H`.
- Yüzde değişimi: `(Son tutar - İlk tutar) / İlk tutar * 100`.

## Birim dönüştürme

Uzunluk · Kütle · Sıcaklık · Alan · Hacim · Zaman · Hız · Veri depolama.

**Kaynak** / **Hedef**: Ayarlar · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Geçmiş

100 · Profil + `context` · `Geçmişi temizle`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Uygulama metni Control Center → Appearance → Desktop Style → Fonts içindeki Applications Font ayarını izler. GTK gtk-font-name değerini devralır; sayı ve başlık boyutları korunur. Başlatıcı üretilen apps_family değerini okur ve değişiklikleri izler.

Üst bilgi vurgu renkli ARGVUS rozetiyle uygulama adını birleştirir. Tıklanabilir s Settings alt bilgisi, Control Center stilinde farklı tema arka planına sahip tam genişlikte bir şeritte yer alır.
