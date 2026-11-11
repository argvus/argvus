---
title: Karşılayan
description: Selamlamayı başlatan oturum açma selamlayıcısı, argvus paketinde gönderilir.
---

`argvus-greeter`, greetd için giriş selamlayıcısıdır. Klavye öncelikli bir terminaldir
Kurulu Wayland oturumlarını keşfeden ön uç, hesapları kendileriyle birlikte gösterir.
avatarları seçer ve seçilen hesap için aktif ARGVUS temasını uygular.

Masaüstünün tek paketi olan `argvus` paketinin bir parçasıdır. Kaynağı yaşıyor
argvus deposundaki `crates/argvus-greeter`'da, kök Cargo'nun bir üyesi olarak
çalışma alanıdır ve  `argvus` `argvus`'nun geri kalanıyla birlikte oluşturulmuştur.

Karşılayan kişi kullanıcıların kimliğini doğrulamaz. greetd kimlik doğrulamayı şu şekilde yönetir:
PAM. Başlatıcı, karşılayıcıyı `argvus tui-terminal` ile açar, böylece karşılayıcı
ayrı bir terminal programına ihtiyacınız var.

Karşılayıcı yapılandırma dosyaları `/etc/argvus` altına yüklenir. Kullanım
Paketin sağladığı kurulum seçeneklerini görmek için `argvus greeter-setup --help`.
Oturum açma akışı için bkz. [Oturumlar](/docs/user-guide/sessions/).

Oturum açma aktarım soketi, her istemciye çekirdek eş kimlik bilgileriyle yetki verir. Yalnızca root, yardımcı sahip ve greetd tarafından kimliği doğrulanan hesap kontrol komutları gönderebilir. Soket kullanıcılar arası bağlantılara izin vermesine rağmen diğer yerel hesaplar reddedilir. Hazırlanan bir DRM çerçevesi orijinal tanımlayıcısına bağlı kalır; yardımcı, ekranı devralmadan önce bu tanımlayıcıyı bekler.

Kimlik doğrulama alanına odaklanıldığında, yazdırılabilir karakterler (`j` ve `k` dahil) yanıta girer. Sekme ve Shift-Tab, odağı taşır; Geri tuşu son karakteri kaldırır. Bu, gizli ve gizli olmayan istemler için geçerlidir.

Hesap listesi, siz terminalde gezinirken veya terminali yeniden boyutlandırırken seçilen hesabı görünür tutmak için kaydırılır.