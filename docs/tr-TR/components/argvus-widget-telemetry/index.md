---
title: Widget telemetrisi
description: Waybar telemetri blokları ve argvus widget-telemetry aracılığıyla bunların arasında geçiş yapılması.
---

Telemetri widget'ı, görev çubuğunda gösterilen sistem bilgileridir (başlık, tuşlar, sistem, CPU)
ve GPU, bellek, depolama, işlemler, ağ ve geliştirici kontrol paneli). Şu anda yapım aşamasındadır
`argvus` paketi: kabuk toplayıcı yok.

## Komutlar

```text
argvus widget-telemetry block BLOCK ...   prints the text of one block (used by the bar)
argvus widget-telemetry toggle ...        switches the blocks on and off, and their order
```

`block`, çubuğun sıcak yoludur: her yenilemede çağrılır. `toggle` hangi engellemeleri değiştirir
çubuk kaydırma penceresini gösterir ve hareket ettirir.

Bloklar şunlardır: `header`, `keys`, `system`, `cpu_gpu`, `memory`, `storage`, `processes`,
`network` ve `dev_dashboard`. Görev çubuğu aynı zamanda `cpu-temp` ve `gpu` problarını da kullanır. GPU probu `--present` (desteklenen bir GPU DRM/sysfs'de görünür), `--available` (panel için bir GPU ve btop komutu mevcuttur), `--available-temp` (atfedilebilir bir sıcaklık okunabilir) ve `--temp` (geçerli en yüksek sıcaklık veya `N/A`).

GPU sıcaklıkları GPU'nun PCI cihazıyla eşleştirilir. NVIDIA'nın tescilli okumaları `nvidia-smi`/NVML veri yolu kimliklerini kullanır; NVIDIA nouveau, AMD ve Intel okumaları hwmon'u yalnızca sysfs cihaz bağlantısı bu GPU ile benzersiz bir şekilde eşleştiğinde kullanır. Görev çubuğu sıcaklığı, algılanan GPU'lar arasında geçerli maksimum değerdir. Eksik, geçersiz veya belirsiz okumalar kullanılamaz. NVIDIA'nın tescilli sıcaklığı, sürücü sağlayıcısını gerektirir; AMD ve Intel sıcaklığı, bağlantılı bir hwmon sensörünü açığa çıkaran çekirdeğe bağlıdır. Intel sıcaklığı btop 1.4.7'nin GPU paneli tarafından sağlanmıyor.

## Oturumla başlıyoruz

`argvus-widget-telemetry.service` kullanıcı birimi, yalnızca widget etkinleştirildiğinde oturumla başlar.
etkinleştirildi. Ana anahtar konfigürasyonda `effects.widget_telemetry_enabled`'dur;
varsayılan olarak etkin olduğundan yeni bir hesap, giriş sırasında widget'ı başlatır.

Geçiş `$XDG_CACHE_HOME/argvus/waybar/widget-telemetry-state` yazdıktan sonra bu dosya
onun yerine karar verir. Widget'ı `enabled`'nun yalnızca ilk satırı başlatır. Daha yaşlı
Geçerli dosya okunduğunda `desktop-telemetry-state` ve `sysinfo-state` dosyaları hala okunuyor
kayıp.

Telemetri etiketleri desteklenen on dilin tamamında kompakttır. CPU, GPU, IP, DE, WM ve Kernel gibi teknik tanımlayıcılar teknik anlamlarını korur. Çalışma süresi `/proc/uptime`'dan okunur ve kısaltılmış birimlerle gösterilir (örneğin `1d 12h 49m`); artık sistem yerel ayarından tam sözcüklere dönüşmüyor. Donanım modeli adları ve diğer sağlayıcı değerleri widget genişliğini hâlâ etkileyebilir.