---
title: Ortam değişkenleri
description: ARGVUS oturum bileşenleri tarafından kullanılan ortam sözleşmeleri.
---

Oturum, XDG ve Wayland ortam entegrasyonunu kurar ve algılandığında sanallaştırma uyumluluk değişkenlerini dışa aktarabilir. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG` ve `ARGVUS_BOOTSTRAP`, oturum ve dağıtıcı yardımcıları tarafından yapılandırma köklerini çözümlemek için kullanılır.

Ortam değişkenlerini çalışma zamanı sözleşmeleri olarak ele alın: Kalıcı bir geçersiz kılma eklemeden önce yüklü yardımcının `--help` veya kaynağını doğrulayın.

Türetilmiş durum, konfigürasyon kökü yerine durum kökü altında bulunur.
`XDG_STATE_HOME` (normalde `~/.local/state`) `argvus/config-projection.json`'yu alır,
projeksiyon bildirimi `argvus-config` yeniden oluşturulur ve `argvus sessionctl reload`
geçerlidir ve ayrıca `argvus/session.log`. İkisi de gerçeğin kaynağı değil.

## Dil ortamı

Tek genel dil tercihi $XDG_CONFIG_HOME/locale.conf'daki `LANG`'dur (varsayılan  `$XDG_CONFIG_HOME/locale.conf` `~/.config/locale.conf`). Geçerli bir değer olmadan, /etc/locale.conf, PAM veya oturum açma yöneticisi Portekizce sağlasa bile ARGVUS İngilizce'yi (`en-US`) seçer.  `/etc/locale.conf` `pt_BR.UTF-8` gibi değerler kataloglar için `pt-BR`'ya normalleştirilir.

Oturum, masaüstü bileşenlerini başlatmadan ve sistem kullanıcısını ve D-Bus etkinleştirme ortamlarını içe aktarmadan önce bu tercihi uygular. Devralınan `LANG`, `LC_MESSAGES`, `LANGUAGE`'nun yerini alır ve `LC_ALL`'yu temizler; aynı dosyadaki bölgesel `LC_*` ayarları korunur. İngilizce, oluşturulduğunda `en_US.UTF-8`'yu kullanır, aksi takdirde `C.UTF-8` kullanılır. Hiçbir tercih dosyası otomatik olarak oluşturulmaz. Dili değiştirdikten sonra oturumu kapatıp tekrar açın.

`session.language`, `argvus/data/language` ve `argvus/data/internal/language` eski veriler yoksayılır. Bunlar ne ithal edilir ne de yenilenir; mevcut dosyalar korunur. `argvus config set` ve `patch`, `session.language`'ya yapılan yeni yazma işlemlerini reddeder ve arayanları `locale.conf`'ya yönlendirir.

Teşhis için `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` ve `systemctl --user show-environment`'yu karşılaştırın. `localectl status`, kullanıcının ARGVUS dilini değil sistemi açıklar. `ARGVUS_I18N_DIR`, geliştirme için dili değil, bir katalog dizini seçer.