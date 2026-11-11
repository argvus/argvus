---
title: Depo rolleri
description: Çalışma alanı birleştirme sonrasında ARGVUS kaynak havuzlarının sorumlulukları.
---

Depolar, gezinme sayfasından ziyade sorumluluğa göre gruplandırılmıştır:

- çalışma alanı: `argvus` (gönderici, süreç içi komutlar, GTK 4 yardımcıları, paylaşılan kasalar, `crates/argvus-greeter`'daki karşılayıcı ve komutların verileri);
- oturum ve entegrasyon: `argvus-session`, `argvus-hyprland`, `argvus-portal`, `argvus-lock`, `argvus-boot-splash` ve karşılayıcı (`argvus-greeter`, `argvus/crates/argvus-greeter`);
- Çalışma alanının dışındaki kullanıcı arayüzü: Denetim Masası, görev çubuğu (`argvus-taskbar`), başlatıcı ve Yol Çubuğu;
- sağlayıcılar: ekran, güç, güvenlik duvarı, bildirimler;
- varlıklar ve paketleme: görünüm, duvar kağıtları, yazı tipleri, simgeler, profiller.

## Çalışma alanına birleştirildi

Bu depolar `de/argvus` ile birleştirildi. Belgeleri artık çalışma alanının `docs/` altında bulunuyor ve depoları yalnızca geçmiş için tutuluyor:

| Eski depo | Şimdi | Dokümantasyon |
| --- | --- | --- |
| `argvus-config` | `argvus config` | [komut satırı](/docs/reference/command-line/), [yapılandırma dosyaları](/docs/reference/configuration-files/) |
| `argvus-i18n` | `argvus i18n` | [Uluslararasılaştırma bileşeni](/docs/components/argvus-i18n/) |
| `argvus-keys`, `argvus-projects` | `argvus keys`, `argvus projects` | komut satırı |
| `argvus-terminal` | `argvus terminal` | [Terminal bileşeni](/docs/components/argvus-terminal/) |
| `argvus-system-monitor` | `argvus system-monitor` | [Sistem monitörü bileşeni](/docs/components/argvus-system-monitor/) |
| `argvus-network` | `argvus network`, `argvus bluetooth` | [Ağ bileşeni](/docs/components/argvus-network/) |
| `argvus-widget-telemetry` | `argvus widget-telemetry` | komut satırı |
| `argvus-control-center` | `argvus control-center` (terminal uygulaması) | [Kontrol Merkezi bileşeni](/docs/components/argvus-control-center/) |
| `argvus-taskbar-calendar` | `argvus taskbar-calendar` (GTK 4 yardımcısı) | [Görev çubuğu takvim bileşeni](/docs/components/argvus-taskbar-calendar/) |
| `argvus-removable-devices` | `argvus removable-devices` (GTK 4 yardımcısı) | [Çıkarılabilir aygıt bileşeni](/docs/components/argvus-removable-devices/) |
| `argvus-loading-theme` | `argvus loading-theme` (`splash` yardımcısı) | komut satırı |
| `argvus-tui-terminal` | `argvus tui-terminal` (çalışma alanındaki sandık) | komut satırı |
| `argvus-tui`, `argvus-theme` | Paylaşılan kasalar `crates/argvus-tui` ve `crates/argvus-theme` | geliştirici notları |
| `argvus-greeter` | `crates/argvus-greeter` formatındaki kaynak, `argvus` paketinde gönderilir | [karşılama bileşeni](/docs/components/argvus-greeter/) |
| `argvus-themes` | Emekli. Her resmi tema kendi `argvus-theme-*` paketidir. | [temalar yükleniyor](/docs/user-guide/appearance/) |

Birleştirilmiş bileşenlerin geliştirici notları `docs/internal/<repository>/`'dadır.

Dil, sahiplik sınırı değildir: her bileşen, çalışma zamanı yardımcılarını paketinde tutar. Paketleri kesen tek katı kural: data/generated/'nun altına yalnızca `argvus-config` yazar.  `data/generated/` `config.json`'yu okuyamayan bir uygulamaya ulaşması gereken bir bileşen, kendi yardımcısına bir bağdaştırıcı ekler ve orkestratörün kurallı işleme sonrasında onu çağırmasına olanak tanır; asla oluşturulan bir dosyayı yazmaz. Genel CLI'ler `/usr/bin`'ya aittir; dahili yardımcılar, çalışma zamanında ihtiyaç duyulduğunda `/usr/lib/argvus/<component>` altına veya komutlar dizinine aittir; mimariden bağımsız veriler `/usr/share/argvus/<component>` altında kalır.

Kurulu paket sınırı için [paket matrisine](/docs/reference/package-matrix/) bakın.