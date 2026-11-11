---
title: Tema ve duvar kağıdı sorunlarını giderme
description: Tema yayılımını ve oluşturulan görünüm dosyalarını tanılayın.
---

$XDG_CONFIG_HOME/argvus/config.json'daki standart `appearance` bölümünü kontrol edin, ardından  `$XDG_CONFIG_HOME/argvus/config.json` `$XDG_CONFIG_HOME/argvus/data/generated` altında oluşturulan dosyaları inceleyin. Eski nokta dosyaları yalnızca geçiş girişleridir.`argvus-config`, `data/generated/`'nun altındaki tek yazardır. Kırılanı kurtarmak için
tema veya duvar kağıdı, tüm nesli kanonik belgeden yeniden oluşturun ve
Oluşturulan tüketicileri doğrudan düzenlemek yerine sonucu uygulayın:

```sh
argvus config validate
argvus config apply
argvus sessionctl reload
```

`argvus theme-switch` (`argvus config theme-apply`'yu çalıştırır) ve `argvus config accent`orkestratörlerdir: doğrularlar, kilitlerler, sıçramayı gösterirler, taahhüt ederler
`argvus-config` ve ardından okuyamayan harici tüketicileri uzlaştırın

. They never write 

, and the theme transaction does
not write the GTK, storage or 

 files through separate scripts.
Use 

 to re-run only the accent reconciliation.

Resmî tema değişiklikleri, kaydedilen görünüm değerlerini uygular ve etkilenen çalışma zamanı tüketicilerini günceller. Özel bir tema uygulamak, görev çubuğunu, Hyprland yeniden yüklemesini, duvar kağıdını ve Denetim Masası hizmetlerini yeniden başlatır. Sticky veya Spaced modu değiştirmek, o modun aralığını, kenarlıklarını ve görev çubuğu marjlarını tekrar uygular, ardından görev çubuğu, telemetri ve Hyprland yeniden yükleme hizmetlerini yeniden başlatır. Dunst'un bildirim renkleri, Rahatsız Etmeyin etkin olduğunda istisnadır: yeniden yükleme, DND kapatılana kadar ertelenir. Eğer başka bir yüzey eski renkleri tutuyorsa, temanın derlemesinin aktif olduğundan ve seçtiğiniz tema olduğundan emin olun, ardından yeniden yüklemeleri tekrar çalıştırın:

```sh
cat "${XDG_CONFIG_HOME:-$HOME/.config}/argvus/data/generated/theme/current/build.toml"
argvus config theme-runtime consumers
```

Her temalı dosya derlemeden gelir: yüklü olmayan bir tema ( theme.toml  argvus config accent --apply  mode.css  data/generated/  `config.json`  `data/generated/`  `mode.css`  `argvus config accent --apply`  `theme.toml` `/usr/share/argvus/themes/` veya `~/.local/share/argvus/themes/` altında theme.toml yok) veya render edilmeyen bir tema reddedilir ve önceki tema kalır. `argvus theme check DIRECTORY`, her render hatasının şablonunu ve satırını gösterir. Bir Waybar çubuğu renklerini yalnızca JSONC içindeki `reload_style_on_change` ile yeniden yükler. Bir anahtar takılı gibi görünüyorsa, hâlâ çalışıp çalışmadığını kontrol edin; kendisini kilidi yanında, `theme-switch.active` içinde kaydeder:

```sh
cat "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/theme-switch.active"
ps -ef | grep -E 'theme-switch|theme-apply'
```

İşaretleyicide hala canlı olan bir pid, geçişin tamamlanmadığı anlamına gelir. Bir anahtar, tema kilidi ve projeksiyon için en fazla 60 saniye, yeniden yükleme komutları için ise 5 saniye bekler; sürecinden çıkan bir işaretleyici eskidir ve bir sonraki anahtar onu yok sayar. Sorunu bildirirken yukarıdaki komutların çıktısını ekleyin.Sadece ön-oturum açma karşılama ekranı beyazsa veya yedek temayı kullanıyorsa, özel kaynağı genel projeksiyonu ile karşılaştırın:

```sh
uid=$(id -u)
argvus config get /appearance/theme --raw
sudo cat "/var/lib/argvus/greeter/themes/$uid"
sudo cat "/var/lib/argvus/greeter/themes/$uid.accent" 2>/dev/null || true
```

greetd yapılandırmasını değiştirmeden projeksiyonu onarın:

```sh
sudo argvus greeter-setup --sync-themes
```

Bu dosya, görünüm orkestratörü tarafından ön-kimlik doğrulama için yayınlanmıştır ve
`data/generated/` dışında bulunur, bu yüzden sade bir `argvus config apply`
refresh it.

Eğer karşılama görevlisi bir Kitty önbelleğinde `Invalid color name: 'null'` bildirirse,
`argvus terminal` paketini güncelleyin ve karşılama görevini tekrar açarak önbelleği yeniden oluşturun.
Eksik bir kanonik vurgu, bir Kitty rengi değil, varsayılan bir durumdur; mevcut terminal
başlatıcı, yazmadan önce JSON `null`'yu ve geçersiz eski değerleri reddediyor
`active_tab_foreground`.

Birkaç tema değişikliğinden sonra varsayılan terminal açılmayı durdurursa, türetilmiş Kitty önbelleğini manuel olarak düzenlemek yerine inceleyin:

```sh
argvus config get /appearance/theme --raw
ls -l "${XDG_CACHE_HOME:-$HOME/.cache}/argvus/argvus-terminal"
argvus terminal --apply
argvus terminal --print-config
```

Yukarıdaki komut kanonik tercihi okur. `.active-theme` yalnızca biruyumluluk projeksiyonu. Önbellek, `argvus terminal` ve
tarafından tema uygulamasıyla senkronize edilerek yeniden oluşturulur. Eksik veya eski bir
`/usr/lib/argvus/loading-theme/splash` eksik bir paket güncellemesini gösterir;
canlı oturumu değerlendirmeden önce koordine edilen paketleri yeniden derleyip yükleyin
result.

Eğer pencere kenarlıkları önceki tema rengini koruyorsa, Hyprland'ın Lua
dosyasını manuel olarak düzenlemeyin. Tema ve vurgu değişiklikleri `config.json` üzerinden yansıtılır, sonra
`argvus sessionctl reload`, Hyprland'ı yeniden yükler ve etkin, etkin olmayan,
ve oturum yaşam döngüsü boyunca kenarlık renklerini gruplandırın:

```text
config.json → projection plan → Hyprland reload → hyprctl reapply of active/inactive/group borders
```

Kenarlık *tercihleri* `argvus config borders` (veya onun
`borders-switch.sh` delegesi) Kontrol Merkezinde ve Kontrol Panelinde; yeniden yüklemek
oturum yaşam döngüsü boyunca taahhüt edilen renkleri yeniden uygular ve
bunları yeniden türetin.

Planı ve etkili birleştirici değerlerini inceleyin:

```sh
jq . "${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json"
hyprctl -j getoption general:col.active_border
hyprctl -j getoption general:col.inactive_border
argvus sessionctl reload
```

`hyprctl` mevcut Hyprland soketine ulaşamazsa kenarlık projeksiyonu
başarısız olarak rapor edildi ve bağımlı bileşenler başarılı bir şekilde değerlendirilmedi
yeniden yüklendi. Yeniden denemeden önce kullanıcı oturumu günlüğünü kontrol edin.