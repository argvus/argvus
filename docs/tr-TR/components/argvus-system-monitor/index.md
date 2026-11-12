---
title: Sistem monitörü
description: ARGVUS sistem monitörünü açın.
---

Yapılandırılmış monitörü şununla açın:

```sh
argvus --system-monitor
```

`argvus system-monitor` alt komutu yerel giriş noktası ve btop entegrasyonunu sağlar. Görev çubuğunda gösterilen telemetri, `argvus widget-telemetry` tarafından sağlanan ayrı bir özelliktir. CPU, Bellek ve GPU modüllerine tıklamak, markalı bir üst bilgi ve `Esc Close` alt bilgi içeren bir ARGVUS TUI panelinin içinde btop'u açar. GPU modu altı adede kadar GPU kutusu gösterir ve CPU, bellek, ağ veya işlem kutusu göstermez. Paneli kapatmak için `q` veya `Esc` tuşuna basın. Alttaki Kitty sekmesi yerelleştirilmiş `Press q to close` ipucunu gösterir. Bağımsız `argvus system-monitor --cpu`, `--mem` ve `--gpu` çağrıları seçilen kutularla doğrudan btop'u başlatır.

GPU paneli, kurulu btop yapısının ve sistemin cihaz sağlayıcılarının yeteneklerine bağlıdır. NVIDIA izleme, btop'ta GPU desteğine, yüklü NVIDIA sürücüsünden NVIDIA Yönetim Kütüphanesine ve cihaza erişime ihtiyaç duyar. AMD izleme, isteğe bağlı olan ve kurulmayabilen ROCm SMI'yı kullanır. Intel GPU verileri sysfs'den okunur ve kullanıcının bu arayüzlere erişimine bağlıdır; btop 1.4.7 Intel kullanımını, gücünü ve saatini bildirir ancak Intel sıcaklığını sağlamaz. NVIDIA nouveau ve diğer sürücüler, cihaza bağlı hwmon aracılığıyla görev çubuğuna GPU sıcaklığı verebilirken, btop sağlayıcı desteği sürücüye ve mevcut kitaplıklara bağlıdır. ARGVUS, cihaz izinlerini değiştirmez veya btop'u yükseltilmiş ayrıcalıklarla çalıştırmaz. Hiçbir GPU açığa çıkmazsa veya destek veya sağlayıcının mevcut olmaması nedeniyle btop üç saniye içinde bir GPU kutusu oluşturmazsa panelde yerelleştirilmiş bir kullanılamıyor mesajı görüntülenir.

Monitör seçilen temayı ve aksanı okur.
İlk önce `$XDG_CONFIG_HOME/argvus/config.json`. Oluşturulan btop dosyaları kalır
türetilmiş önbellek durumu; `.active-theme` ve `.accent-color` uyumludur
geri dönüşler. Btop kendi tema dosyasını sakladığından monitör
kurallı değişiklikten sonra harici bağdaştırıcı olarak tema işlemi (`argvus theme-switch`)
kararlı; oluşturulan ağacın bir parçası değildir.

## Çalışma zamanı sözleşmesi

- Tema ve vurgu kanonik konfigürasyondan gelir (`argvus config get
/appearance/theme` and `/appearance/accent`). The files `~/.config/argvus/data/.active-theme`
  ve `.accent-color` geri dönüşlerdir.
- Oluşturulan btop konfigürasyonu ve temalar şuraya yazılır:
  `$XDG_CACHE_HOME/argvus/argvus-system-monitor`. Bunlar türetilmiş hallerdir; bunları düzenlemeyin.
- Varsayılanlar `/usr/share/argvus-system-monitor/btop` altına yüklenir.
- Modül kısayolları (`--mem`, `--cpu`, `--net`, `--proc`) yalnızca ilk bağımsız değişken olarak okunur.
  Btop'un kendi `-c`/`--config` veya `--themes-dir`'sunu geçirmek, btop'u bu yollarla çalıştırır ve hayır yazar
  konfigürasyon.
- `--panel [--cpu|--mem|--gpu]`, btop'u ARGVUS TUI kabuğuna yerleştirir. Bu mod görev çubuğu tarafından kullanılır;
  `--panel` olmadan monitör kendini btop ile değiştirmeye devam ediyor.
