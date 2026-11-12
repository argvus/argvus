---
title: Temaları içe ve dışa aktarma
description: Etkin temayı kendi temanız olarak dışa aktarın, bir tema arşivini içe aktarın, uygulayın ve silin.
---

Dışa ve içe aktarma, **temalarla**, resmi temalarla aynı formatta çalışır. Dışa aktarma, vurgu renginiz ve duvar kağıdınızla birlikte etkin temayı kendinize ait yeni bir tema olarak kaydeder. İçe aktarma, temalarınız arasına bir tema arşivi yükler. Her ikisi de **Kontrol Merkezi → Görünüm → Temalar**'dadır ve temalarınız **Özel Temalar**'da listelenir.

## Dışa aktarılan bir temanın içeriği

`argvus-theme-<id>.tar.gz` arşivinde resmi bir tema gibi düzenlenmiş bir tema dizini bulunur:

```text
<id>/theme.toml        the manifest (schema 2): name, category, palette, wallpaper
<id>/appearance.json  your visual settings, including layout, effects, fonts and panel presentation
<id>/effects.conf     per-surface effect values for the active theme, when present
<id>/overrides/...     the overrides of the theme it was made from, if any
<id>/wallpaper.<ext>   the wallpaper on screen when you exported
```

Bildiri, adınız, yeni kimliğiniz, palet vurgusu olarak vurgu renginiz ve yanındaki duvar kağıdıyla birlikte etkin temayı temel alır. Görünüm profili, pencere aralığı, kenarlıklar, görev çubuğu kenar boşlukları, panel sunumu, renkler, efektler ve yazı tipleri dahil olmak üzere ARGVUS'ta geçerli olarak uygulanan görsel ayarları korur. Özel duvar kağıdı ve vurgu rengi temayla birlikte saklanır. Herhangi bir temanın uygulanması, son uygulandığından bu yana yapılan bağımsız görsel değişikliklerin yerini alarak tüm profilini (veya profil olmadığında ARGVUS varsayılanlarını) yeniden uygular. Yapışkan veya Aralıklı arasında geçiş yapmak her zaman o modun pencere ve görev çubuğu geometrisini yeniden uygular; Dışa aktarılan özel bir tema uygulandığında kullanılabilir kalmasını sağlamak için kişisel geometriyi bu temaya kaydedin. Ses, klavye ve ağ tercihleri ​​gibi görsel olmayan ayarlar bir temanın parçası değildir.

Uygulanan görünüm, oturumun yeniden başlatılması ve yeniden yüklenmesi sırasında korunur. Oturum hazırlığı, orijinal temanın geometrisini geri yüklemeden gerekli dosyaları geçerli yapılandırmadan yeniden oluşturur.

Bir tema uygulandıktan sonra değiştirilen ayarlar, bir sonraki tema veya mod değişikliğinde geçicidir. Mevcut görsel yapılandırmayı korumak ve daha sonra dışa aktarılan temayı uygulayarak geri yüklemek için temayı **Denetim Merkezi → Görünüm → Temalar → Temayı dışa aktar** seçeneğinden dışa aktarın.

Temalarınız `~/.local/share/argvus/themes/<id>/`'ya yüklenir. Orada bir arşivin çıkarılması temayı da yükler.

## Etkin temayı dışa aktar

1. **Kontrol Merkezi → Görünüm → Temalar**'ı açın.
2. **Temayı dışa aktar** seçeneğini seçin veya Temalar sayfasında `e` tuşuna basın.
3. Tema için bir ad yazın. Alan yazılacak dosyayı gösterir.
4. `Enter` tuşuna basın.

Kimlik, addan oluşur: küçük harfler, rakamlar ve kısa çizgiler. Kimlik, yüklü bir tema veya ana dizininizdeki bir arşiv tarafından zaten kullanıldığında, `-2`, `-3` vb. eklenir. Arşiv ana dizininize yazılır, örneğin `~/argvus-theme-my-theme.tar.gz`. ARGVUS, temayı yazmadan önce kontrol eder: manifest yüklenmeli ve her bileşen her iki modda da oluşturulmalıdır.

## Bir temayı içe aktar

1. `.tar.gz` arşivini ana dizininize (`~`) kopyalayın. ARGVUS, orada bir temayı içeren arşivleri listeler.
2. **Denetim Merkezi → Görünüm → Temalar → Temayı içe aktar**'ı açın veya Temalar sayfasında `i` tuşuna basın.
3. Arşivi seçin ve `Enter` tuşuna basın.

ARGVUS, arşivi kurmadan önce kontrol eder. `theme.toml` içeren, yalnızca göreli yollara sahip (`..` yok, bağlantı yok), en fazla 128 giriş ve dosya başına boyut sınırı olan tek bir dizini barındırması gerekir; kimliğin dizinle eşleşmesi, duvar kağıdının arşivde olması ve her bileşenin her iki modda da oluşturulması gerekir. Dahil edilen bir görünüm profilinin desteklenen bölümleri ve geçerli JSON'u kullanması gerekir. Herhangi bir kontrol başarısız olursa hiçbir şey yüklenmez.

Tema **Özel Temalar**'a eklenir. İçe aktarma bunu uygulamaz: ne zaman olacağını siz seçersiniz.

### Ad çarpışmaları

- Temalarınızdan biri aynı kimliğe sahipse ARGVUS, **Temanız aynı kimlikle değiştirilsin mi?** diye sorar. Değiştirmek için **Değiştir**'i seçin veya kendinizinkini korumak için `Esc` tuşuna basın. Değiştirilen tema, yenisi yüklenene kadar yedek olarak saklanır.- Resmi bir temanın kimliğine sahip bir arşiv reddedilir. Başka bir adla tekrar dışa aktarın.

## Özel bir tema uygula

1. **Kontrol Merkezi → Görünüm → Temalar → Özel Temalar**'ı açın.
2. Temayı seçin ve `Enter` tuşuna basın.

Duvar kağıdı, aralık, kenarlıklar, vurgu rengi, efektler, yazı tipleri ve panel sunumu dahil olmak üzere temanın dışa aktarılan yapılandırmasının tamamı uygulanır. Görsel değerleri mevcut ayarların yerini alır. ARGVUS daha sonra görev çubuğunu, Hyprland yeniden yükleme hizmetini, duvar kağıdını ve Denetim Masasını yeniden başlatarak her yüzeyin özel temayı kullanmasını sağlar.

## Özel bir temayı silin

1. **Kontrol Merkezi → Görünüm → Temalar → Özel Temalar**'ı açın.
2. Temayı seçin ve `d` tuşuna basın.
3. `y` ile onaylayın veya iptal etmek için `n` veya `Esc` tuşuna basın.

Tema dizini `~/.local/share/argvus/themes`'dan kaldırıldı. Etkin temaysa ARGVUS önce **ARGVUS Dark**'a geçer.

## Önceki sürümlerin profilleri

Önceki sürümler, görünüm ayarlarının bir kopyası olan *tema profillerini* (`profile.tar.gz`) dışa aktarıyordu. Bir Temalar sayfası ilk açıldığında `~/.local/share/argvus/themes/<name>/`'daki her profil, vurgu rengi ve duvar kağıdıyla birlikte temel temasından oluşturulan bir temaya dönüştürülür. Orijinal profil ve eski `custom-themes.json` listesi `~/.config/argvus/data/backups/theme-profiles-<seconds>/`'ya taşınır ve durum satırı dönüşümü bildirir. Temel teması kurulmamış profil yalnızca yedeğe taşınır.

## Bir temayı paylaşın

Dışa aktarılan bir tema normal bir arşivdir. ARGVUS çalıştıran başka bir makineye kopyalayın ve oraya aktarın.

## Sorun Giderme

- **Arşiv listelenmemiş.** Doğrudan ana dizininizin içinde olmalı, `.tar.gz` ile bitmeli ve `<id>/theme.toml` şeklinde olmalıdır.
- **İçe aktarma reddedildi.** Durum satırında bunun nedeni belirtiliyor: aynı kimliğe sahip resmi bir tema, eksik bir duvar kağıdı, yüklenmeyen bir bildirim veya oluşturulamayan bir bileşen.
- **Dışa aktarma reddedildi.** Adın en az bir harf veya rakama ihtiyacı vardır ve etkin temanın yüklenmesi gerekir.
