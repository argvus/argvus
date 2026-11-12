---
title: Gece Lambası
description: Hyprland'da renk sıcaklığını ve sabit yerel programları ayarlayın.
---

**Denetim Merkezi > Masaüstü ve Giriş > Ekranlar > Gece Işığı**'nı açın, **Gece Işığı**'nı arayın veya `argvus control-center displays night-light`'yu çalıştırın.

Gece Işığı ve programı başlangıçta kapalıdır. **1000 ile 20000 Kelvin** arasında bir sıcaklık seçin (varsayılan **4500 K**). Daha düşük değerler daha sıcak renkler verir. Sayfada kayıtlı manuel aktivasyon, etkin durum ve bir sonraki geçiş gösterilir. Enter bir değeri düzenler; Esc bunu iptal eder veya Ekranlara geri döner. Geçersiz değerler kaydedilmeden önce reddedilir. Değişiklikler kaydedilir ve hemen uygulanır. Kaydetme sonrasında uygulama başarısız olursa, kaydedilen tercih, fiili etkin durum ve hatanın yanında görünür durumda kalır.

## Programlama ve hızlı eylemler

Sabit program geçerli yerel saat dilimini kullanır. Başlangıç ​​dahildir ve bitiş hariçtir; **20:00–07:00** gece yarısını geçiyor. Başlangıç ​​ve bitiş farklı olmalı ve `HH:MM` kullanılmalıdır. Hiçbir konum veya güneş enerjisi hesaplaması kullanılmaz.

Hızlı geçişler için Kontrol Panelinin **Ekran** kartını veya **SUPER + Shift + B**'yi kullanın. Parlaklık seçici, tüm parlaklık eylemlerini korur ve parlaklık arka ucu olmayan bir ekran da dahil olmak üzere arka uç mevcut olduğunda Gece Işığını ekler. Kalıcı bir görev çubuğu modülü yoktur.

Planlama kapalıyken manuel aktivasyon kalıcıdır. Zamanlama açıkken, hızlı bir geçiş, bir sonraki zamanlanmış geçişe kadar çalışma zamanının geçersiz kılınmasını sağlar. Programın veya sıcaklığın değiştirilmesi bu geçersiz kılmayı temizler. Oturumun kapatılması, yeni bir birleştirici örneği veya değiştirilmiş bir sonraki geçiş (saat dilimi değişikliği dahil) de bunu geçersiz kılar.

## Komutlar ve durum

Tüm arayüzler aynı genel komutu kullanır:

```sh
argvus config display night-light status
argvus config display night-light enable
argvus config display night-light disable
argvus config display night-light toggle
argvus config display night-light temperature 4500
argvus config display night-light schedule on 20:00 07:00
argvus config display night-light schedule off 20:00 07:00
argvus config display night-light apply
argvus config display night-light apply-schedule
```

`status`, JSON şeması sürüm 1'i döndürür: `available`, `unavailable_reason`, `settings` (etkin, sıcaklık, program_enabled, başlangıç, bitiş), `active`, `effective_temperature`, `mode` (`manual` veya `scheduled`), `manual_override`, `next_transition` (ofset ile ISO 8601), `minimum_temperature` ve `maximum_temperature`. Etkili alanlar kullanılamadığında boştur; Kimlik aktif olduğunda etkin sıcaklık sıfırdır. Kullanılabilirlik hataları durum olarak raporlanır; eylem başarısızlıkları bir teşhisle birlikte sıfırdan farklı bir değer döndürür.

Kanonik tercih `/displays/night_light`'dur ve `argvus config` tarafından `$XDG_CONFIG_HOME/argvus/config/displays.json`'da atomik olarak depolanır. Geçici geçersiz kılmalar ve eylem kilidi `$XDG_RUNTIME_DIR/argvus/night-light/` altında yayında. Oluşturulan monitör dosyaları bağımsızdır. Bu özelliği yapılandırmak için oluşturulan dosyaları düzenlemeyin.

## Çalışma zamanı

Hyprland ve **hyprsunset >= 0.4.0** gerektirir. Oturum, yukarı akış `hyprsunset.service`'yu bir kez başlatır, `argvus-night-light-init.service` aracılığıyla kayıtlı durumu uygular ve `argvus-night-light.timer` ve `argvus-night-light.service` aracılığıyla programları her dakika değerlendirir. Saat ve zaman dilimi değişiklikleri de değerlendirmeyi tetikler. Programlama kapalıyken, periyodik işlem manuel durumu yalnız bırakır. Tekrarlanan uygulamalar etkin kimlik/sıcaklığı sorgular ve değişmeyen yazma işlemlerini atlar. Birleştiricinin yeniden yüklenmesine gerek yoktur.

ARGVUS, yukarı akış hizmeti için bir systemd drop-in'i gönderir: `--identity` ile başlar ve yapılandırma keşfini `/usr/share/argvus/session/config/night-light`'ya yalıtır. Bu, yerel profillerin rekabet etmesini önler ve kişisel `hyprsunset.conf` dosyalarına dokunulmaz. ARGVUS dışında başlatmalar da dahil olmak üzere, ARGVUS oturum paketi kuruluyken, bu giriş hizmeti için geçerlidir. Mevcut kullanıcı sistem geçersiz kılmaları öncelikli olabilir; Bu durumu teşhis ederken `systemctl --user cat hyprsunset.service`'yu inceleyin. Hyprsunset 0.4.0, `--config`'nun ayrı yol bağımsız değişkenini reddeder, dolayısıyla yapılandırma keşfi bunun yerine yerel hizmet XDG değişkenleriyle yalıtılır.Arka uç, sınırlı G/Ç ve değişikliklerden sonra geri okuma ile resmi Hyprsunset IPC'yi (`temperature`, `identity`, `identity get`) kullanır. Gölgelendirici, alternatif birleştirici arka ucu, sistem yapılandırması yazma veya ayrıcalık yükseltme kullanılmaz. Kullanılamayan kontroller Kontrol Merkezi'nde devre dışı bırakılır ve Ekran kartında gizlenir.
