---
title: Oturum başlatma sorunlarını giderme
description: Karşılayıcı, Hyprland ve sistemd kullanıcı oturumu hatalarını teşhis edin.
---

Çalıştır:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```

Karşılayıcıyı, `argvus start`, Hyprland'ı ve ilgili kullanıcı birimini ayrı ayrı kontrol edin. Başarılı bir paket yapısı, kurulu yükün veya etkin kullanıcı geçersiz kılmanın güncel olduğunu kanıtlamaz.

Kontrolleri şu sırayla kullanın:

1. Karşılayıcının yüklü ARGVUS karşılama kurulumunu kullandığını doğrulayın.
2. Oturum günlüğünü okuyun ve eksik veya geçersiz Hyprland Lua yapılandırmasını arayın.
3. `systemctl --user status argvus-session.target` ile hazırlığı ve hedef durumu kontrol edin.
4. Her hizmeti aynı anda yeniden başlatmak yerine arızalı bileşeni inceleyin.

Yükleme katmanı etkinken `argvus-session.target` etkin değilse, inceleyin
hizmetleri yeniden başlatmadan önce projeksiyon hatası:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```

`argvus-config`, `data/generated/`'nun altındaki tek yazar olduğundan `project`,
Teşhis koyarken bir nesli yeniden üretmenin güvenilir yolu. Oturum başlangıcı
henüz taahhüt edilmiş bir plan olmadığı için çalıştırıyor; yazdığı plan
`${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json` ne demek
daha sonra tekrar projeksiyon yapmadan `argvus sessionctl reload` uygulanır.

Oturum, ölümcül hazırlık veya hedef başlatma sırasında yükleme katmanını durdurur
başarısız olur. Görsel tüketici arızası ya belgelenmiş geri dönüşünü kullanmalı ya da
açıkça bildirildi; başarılı bir oturum olarak gizlenmemelidir.

## Yavaş giriş

`greeter.handoff-takeover-failed`, `handoff.master-acquired` olmadan görünüyorsa,
satın alma için `handoff.master-unavailable card=<name> errno=<n>`'yu inceleyin
hata. Başka bir dizgici master iken açılan bir gözlemci errno döndürebilir
13 (`EACCES`) bu besteci master'ı serbest bıraktıktan sonra bile. Yardımcı cihazı yeniden açar
örtülü olarak master elde etmek için aynı GPU; `handoff.master-reopened` bunu kaydediyor
kurtarma ve bu tanımlayıcı üzerinde yeni bir çerçeve arabelleği hazırlanır. Karşılayan kişi başarısız olan yardımcıyı (`greeter.handoff-cancelled`) iptal eder, dolayısıyla
Kullanıcı bestecisi başlamadan önce başka bir bekleme eklenemez. Bir oturum gönderimi
`release` satın alma beklemedeyken `handoff.release-before-master` alır:
yardımcı, onaylamadan önce DRM tanımlayıcılarını kapatır. Bir karşılama bestecisi
Hyprland/Aquamarine'deki kaza ayrı bir bulgudur; yardımcının iptal edilmesi
sürücü veya dizgici sorununun çözüldüğünü kanıtlamaz.

`greeter.vt-switch-returned active=<tty>` oturum açıldığında aktif VT'yi kaydeder
çağrı dönüşlerini değiştir. Çizim zaman aşımı aynı zamanda `active-vt=<tty>`'yu da kaydeder.
hata. Bunlar anlık görüntülerdir, tamamlanmış bir DRM aktarımının kanıtı değildir.

Öncesi/sonrası karşılaştırması için çalışmayı kaydedin ve sürüm başına beş oturum açma bilgilerini kullanın.
aynı makine, tema ve monitör kurulumu. Sonraki her oturum kapatmadan önce, kopyalayın
`/run/argvus-greeter/timing.log`, oturum günlüğünü ve önyükleme günlüğünü ayrı bir
`/tmp/argvus/` altındaki örnek dizin; bir sonraki karşılama koşusu zamanlamasının yerini alır
günlük. Her örnekle önyükleme kimliğini kaydedin ve zaman damgalarını yalnızca bu örnekle karşılaştırın
önyükleme. İlk çerçeveyi, oturum sarmalayıcı girişini devretmek için kimlik doğrulamayı ölçün,
Hyprland'ın son fırlatılması ve yükleme-döndürücü durağı. Medyan ve en kötüyü karşılaştırın
kimlik doğrulamadan döndürücüyü durdurmaya kadar geçen süre ve siyah çerçevelerin görsel olarak kontrol edilmesi,
yanıp sönüyor veya sıkışmış bir döndürücü var. Masaüstü yüzeylerinin süresi tek başına
uçtan uca giriş süresi. Arıza ve kurtarma testlerini tek kullanımlık bir oturumda tekrarlayın,
kaydedilmemiş çalışmayı içeren bir oturumu sonlandırmadan.

Her başlangıç ​​adımı, süresini oturum günlüğüne kaydeder. Son oturum açma adımlarını görmek için önce en yavaş olan:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```

Karşılayıcıdan masaüstüne geçişi zamanlamak için, karşılayanı ve teslim işaretlerini günlükle birlikte okuyun. Hepsi `CLOCK_MONOTONIC` kullanıyor ve şu şekilde sıralanıyorlar:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log` yalnızca son oturum açma bilgisini tutar: karşılayan kişi, DRM aktarım yüzeyi (`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) ve `greeter.*` işaretleri. `handoff.crtc` hatları, aktarım DRM ana birimini beklerken her CRTC değişikliğini kaydeder; `fb=0` veya `mode_valid=false` veya `handoff.output-found ... framebuffer=0` içeren bir satır, karşılama düzenleyicisinin, aktarım alınmadan önce çıkışı kapattığı ve dolayısıyla monitörün yeniden senkronize edilmesi gerektiği anlamına gelir. `greeter.watchdog-stop-requested`, karşılama oturumunun birleştiricisini beklemeden sona erdiği anlamına gelir ve `greeter.compositor-exit-deferred`, aktarım yüzeyi gittikten sonra dizgiciden çıkmasının istendiği anlamına gelir; `greeter.compositor-exit-requested` tek başına, hemen çıkmasının istendiği anlamına gelir. Günlükteki selamlama ve oturum açma satırları, karşılama oturumunun kapanması ile kullanıcı oturumunun açılması arasında yaklaşık yarım saniyelik bir boşluk gösteriyorsa, karşılama derleyicisinin sökme işlemi, oturum açma sırasında çalıştırıldı. İşaret `compositor-in-foreground=true`'yu gösterdiğinde bu beklenen bir durumdur: Karşılayıcının terminalinin kapatılması, ertelenmiş çıkıştan önce dizgiyi sonlandırır. Bu, karşılayıcının dizgicisi çıkmadan önce VT'sinden ayrılamadığı durumlarda gerçekleşir: `greeter.vt-switch-requested`'yu ve ardından `greeter.handoff-drawing`'yu arayın; ikincisi eksik olduğunda `greeter.handoff-takeover-failed error=<reason>` nedenini söylüyor. `handoff.page-flip`, geçişin, karşılayanın mod seti olmadan son karesinin yerini aldığı anlamına gelir; `handoff.page-flip-failed errno=<n>` ve ardından `handoff.modeset` gelmesi, çekirdeğin sayfa çevirmeyi reddettiği anlamına gelir. `handoff.overlay plane=<id>` ve ardından `handoff.retired`, oturum döndürücü ekranda görünene kadar kaplamanın Hyprland'ın ilk karelerini sakladığı anlamına gelir; `handoff.overlay-unavailable`, CRTC'nin boş katman düzlemi olmadığı anlamına gelir ve `handoff.retire-timeout`, oturum döndürücünün hiçbir zaman `retire` göndermediği anlamına gelir. Oturum döndürücü (`argvus-greeter-handoff`) günlüğe `first layer-shell frame callback` ve `stop requested` yazar.

Bir görünüm değişikliği, değişen tema dosyaları, yeni bir aksan veya eski bir başlangıç ​​damgasının taşınması sonrasında `project.theme-switch` veya sıfırdan farklı bir `prepare.startup-theme` girişi bekleniyor. Aynı dosyaların yeniden yüklenmesi, içerik tabanlı başlangıç ​​önbelleğini korur. Her oturum açma işleminde görünüyorsa, gerçekleştirilmiş tema dosyaları eksiktir; `argvus theme-switch <theme>`'yu bir kez çalıştırın.

Oturum günlüğü `$XDG_STATE_HOME/argvus/session.log` (normalde `~/.local/state/argvus/session.log`) altında tutulur. Kullanıcının Hyprland geçersiz kılma işlemini değiştirdikten sonra oturumu kapatıp açın, böylece oturum giriş noktası onu doğrular ve tekrar seçer. Normal başlatma akışı için bkz. [Grafik oturum](/docs/argvus-session/graphical-session/).
