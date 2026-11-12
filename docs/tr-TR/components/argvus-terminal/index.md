---
title: Terminali
description: ARGVUS terminalini ve terminal uygulama profillerini açın.
---

Yapılandırılmış terminali şununla açın:

```sh
argvus --terminal
```

`argvus terminal` (`argvus`'nun bir alt komutu), terminal entegrasyonunu ve Kitty yapılandırmasını sağlar. `argvus-app-profiles`, Superfile ve Yazi gibi terminal uygulamaları için profiller sağlar; `argvus` dağıtıcısını sağlamaz.

`argvus-config`, temayı `data/generated/terminal/`'ya yansıtır ve `argvus --yazi` ve `argvus --spf`, yapılandırmalarını ana dizininizde gerçekleşen kopyalar yerine yansıtılan `data/generated/yazi/` ve `data/generated/superfile/` ağaçlarından okur. Uygulamanın kendi yapılandırma dizinindeki tam yerel geçersiz kılma yine de kazanır; böylece yansıtılan ağacı kendi yapılandırmanızla değiştirebilirsiniz.

Tema değişiklikleri, Kitty'nin türetilmiş dosyalarını `$XDG_CACHE_HOME/argvus/argvus-terminal` altında yeniden oluşturur. Oluşturucu güncellemeleri serileştirir ve her dosyayı atomik olarak değiştirir. Terminal hala açılmıyorsa aktif temayı inceleyin ve önbelleği şununla yeniden oluşturun:

```sh
argvus config get /appearance/theme --raw
argvus terminal --apply
argvus terminal --print-config
```

Terminal, yönetilen temayı ve aksanı şuradan okur:
İlk önce `$XDG_CONFIG_HOME/argvus/config.json`. Eski `.active-theme` ve
`.accent-color` dosyaları eski kurulumlar için yedek olarak kalır.

## Şeffaflık ve bulanıklık

**Denetim Merkezi → Görünüm → Terminal** altında **Şeffaflık**'ı yapılandırın
ve **Bulanıklaştırma**, `0%`'dan `100%`'ya kadar bağımsız olarak. Değişiklikler şu şekilde gerçekleştirilir:
**Uygula** düğmesini tıklayın ve altında Kitty profilini yeniden oluşturun.
`$XDG_CACHE_HOME/argvus/argvus-terminal`; mevcut Kitty pencereleri
yeniden yükleme yoluyla güncellenen yapılandırma.

`0%` şeffaflığı arka planı opak tutar ve `100%` onu tamamen şeffaf hale getirir
şeffaf. Bulanıklaştırma, Kitty'nin desteklediği arka plan bulanıklaştırma yarıçapını kullanır ve yalnızca
pencere şeffaf olduğunda görünür etki.

## Uzun süren komut bildirimi

30 saniye veya daha uzun süre çalışan komutlar bittiğinde bildirim gönderir,
terminal penceresi odaklanmadığı sürece. Bildirim Kitty'ninkini kullanıyor
Zaten etkin olan kabuk entegrasyonuyla `notify_on_cmd_finish`
`~/.zshrc` ve `~/.bashrc`'nun değişikliğe ihtiyacı yoktur. 30 saniye eşiği sabittir
Kitty konfigürasyonunda.

## TUI Uygulama Modu

ARGVUS Kontrol Merkezi ve görev çubuğu TUI açılır pencereleri, `argvus tui-terminal`'yu bir
özel Kitty uygulama profili. Aktif ARGVUS temasını ve yazı tipini devralır,
kararlı bir pencere sınıfı atar ve bir pencere açmak yerine çocuğuyla birlikte çıkar.
kabuk. Kontrol Merkezi sekmeleri gizler ve bağımsız Şeffaflık ve Bulanıklığa sahiptir
değerler; Görev çubuğu monitörü ve sıcaklık açılır pencereleri, görevleri için alt sekme çubuğunu kullanır.
yerelleştirilmiş `Press q to close` başlığı. Başlatıcı, ayarlamak için `--title TITLE`'yu kabul eder
sabit bir Kitty penceresi ve sekme başlığı. Normal terminal sekmelerini tutar ve
ayarlar.

Başlatıcı ortamında LANG olmadığında, Kitty klavye kompozisyonunu başlatmadan önce `argvus terminal`, `$XDG_CONFIG_HOME/locale.conf`'dan (varsayılan olarak İngilizce) kişisel yerel ayarı sağlar. Mevcut yerel ortam geçersiz kılmaları korunur. Güncellemeden sonra Kitty'yi yeniden başlatın: LANG'ı mevcut bir kabuğun içine aktarmak Kitty'nin oluşturma tablosunu yeniden başlatmaz.

Oturumun başlatılmasından devralınan boş LC_ALL ve LC_CTYPE, Kitty alt ortamından kaldırılır. Aksi takdirde Kitty boş bir yerel ayarı açık olarak değerlendirir ve geçerli bir LANG ile bile oluşturma tablosunu yükleyemez. Boş olmayan geçersiz kılmalar korunur. Bu aynı zamanda yeni profiller için de geçerlidir ve başlatıcıyı güncelledikten sonra Kitty'nin yeniden açılmasını gerektirir.

Özel Kontrol Merkezi penceresi (`argvus-control-center`), başlatma başına Kitty geçersiz kılma `confirm_os_window_close=0`'yu alır, böylece besteci kapatma eylemi onay istemez. Geçersiz kılma, paylaşılan terminal profiline yazılmaz; diğer TUI'ler ve normal terminaller mevcut politikalarını korur.
