---
title: ARGVUS Uluslararasılaştırma
description: Dil katalogları, seçimi ve Unicode desteği.
---

# ARGVUS'un Uluslararasılaşması

ARGVUS Gemiler 18 arayüz dilleri: Arapça (İngilizce)`ar`), Bengali (`bn-BD`Alman (İngilizce).`de-DE`)
İngilizce (İngilizce)`en-US`İspanyolca (İngilizce).`es-ES`Fransızca (İngilizce).`fr-FR`)`hi-IN`), Endonezya
(`id-ID`İtalya (İngilizce).`it-IT`Japonya (İngilizce).`ja-JP`Kore (İngilizce)`ko-KR`), Brezilyalı Portekiz
(`pt-BR`Avrupa Portekiz (İngilizce)`pt-PT`Rusya (İngilizce).`ru-RU`Türkçe (İngilizce).`tr-TR`), Urdu
(`ur-PK`Vietnam (İngilizce).`vi-VN`) ve Basitleştirilmiş Çin (İngilizce).`zh-CN`). İngilizce İngilizcedir
Canonical katalog ve fallback. Çeviriler ile üretildi AI Yardım ve yardım ve yardım
Teknik düzeltmeler; bağımsız yerli-speaker incelemesi onaylanmadı.

## Bir dil seçin

Open Control Center > Tercihler > Dil & Bölge > Dil. Sayfa keşfeder
Oluşturulan kataloglar ve yerli isimlerini gösterir. Tasarruf eder `ARGVUS_LANGUAGE` in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in
`$XDG_CONFIG_HOME/locale.conf` (default `~/.config/locale.conf`), bağımsız olarak
`LANG`; Bir sistemi yerelleştirmek gerekli değildir. Brezilyalı Portekiz (İngilizce).`pt-BR`)
Avrupa Portekiz (İngilizce)`pt-PT`) ayrı seçimlerdir.
Tercih değişmez `/etc/locale.conf` Ya da bölgesel `LANG` ayarlayın.
Yorumlar ve bölgesel kategori ayarları korunmuştur. Kontrol Panel yeniden başlatılıyor
Tasarrufdan sonra; oturum bileşenleri boyunca dili uygulamak için yeni bir oturum başlat.
Yapılanmamış kullanıcılar İngilizce kullanır. Ayrı Bölgesel Yerel tercih kontrolleri
`LANG`, bölgesel formatlama ve üçüncü taraf uygulamaları. Giriş selamlayıcı çalışır
Kullanıcının tercihi mevcut değildir ve kendi hizmet-accountlarını korur
Dil, İngilizce varsayılan olarak.
Bölgesel varyantlar, tam olarak belirlenmiş katalogu dil için kullanırlar
katalog yok; örneğin `es_MX.UTF-8` kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım kullanım `es-ES`. `zh-Hans` Seçiciler
Basitleştirilmiş Çin. Geleneksel Çin (İngilizce)`zh-Hant`, `zh-TW`, `zh-HK`, `zh-MO`)
Basitleştirilmiş Çin'e sessizce haritalanmadı ve İngilizce'ye geri döndü.

## Kataloglar ve katkılar

Kaynak, argvus deposundaki `resources/locales/<locale>/<domain>.json`'dur.
Yüklü kataloglar  `argvus` `/usr/share/argvus/i18n/`'da yayınlanmaktadır. Ustayla gönderiyorlar
`argvus` paketi. `locale.json` tanımlayıcıyı ve yerel/görünen adları sağlar.
Ayrı bir PO kataloğu veya gerekli çevrimiçi çeviri platformu yoktur.

Anlamsal anahtarları kullanın ve yer tutucu adlarını, komutları, seçenekleri, yolları ve bilgileri koruyun.
klavye kısayolları. Cümleleri birleştirmek yerine mesajların tamamını çevirin
çevrilmiş parçalardan. Yeni kanonik anahtarların onda da çeviriye ihtiyacı var
diller. Her alan adında terminolojiyi tutarlı tutun ve çevirileri inceleyin
bağlamda, özellikle kimlik doğrulama, izinler ve yıkıcı eylemler.

```sh
argvus i18n list
argvus i18n locale
argvus i18n get common welcome name=Alice
argvus i18n validate
```

`ARGVUS_I18N_DIR`, geliştirme ve aşamalandırma için alternatif bir katalog kökü seçer
paket kontrolleri. Doğrulama, etki alanlarını, anahtarları, dize değerlerini, yer tutucuları,
boş çeviriler ve yerel ayara özgü tamsayı çoğul formları. Doğrulama kanıtlıyor
Dilsel doğruluk değil, yapısal bütünlük.

## Unicode ve yazı tipleri

`argvus-fonts`, Latince, sembol ve emojiye ek olarak Noto CJK yazı tiplerini gerektirir
aileler. Fontconfig, Çince, Japonca ve Hangul için geri dönüş sağlar.
kullanıcının seçtiği birincil aileyi değiştiriyor. Uygulamalar ve terminaller hala
yazı tipi geri dönüşünün etkinleştirilmesi gerekiyor.

Paylaşılan Ratatui metin yardımcıları terminal hücrelerini ölçer, genişletilmiş grafiği korur
kesme ve geniş metni sarma sırasında kümeler. Metin editörleri taşır ve siler
bütün grafikler. Son derece dar alanlar iki hücreli bir glifi tek bir alanda görüntüleyemez
hücre; bu durumda sert sarma yardımcısı bir değiştirme karakteri kullanır. Emoji ve
belirsiz genişlikteki karakterler, terminal/yazı tipi uygulamaları arasında hala farklılık gösterebilir.
Terminalin genişlik politikası Ratatui'nin normal Unicode genişliğine uygun olmalıdır.

Dil seçimi bir giriş yöntemi yüklemez veya klavye düzenini değiştirmez.Çince, Japonca veya Korece girerken Wayland uyumlu bir giriş yöntemi kullanın.
