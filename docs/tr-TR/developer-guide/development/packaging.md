---
title: Paketleme
description: ARGVUS Arch paketlerini oluşturun ve inceleyin.
---

Her bileşen, deposu altında Arch ambalajına sahiptir. Bir veriyi belgelemeden önce `packaging/archlinux/`'yu, paket işlevlerini ve kurulu dosya listesini inceleyin. `/usr/share/argvus` kapsamındaki paket varsayılanları ve `/etc/argvus` kapsamındaki yönetici yapılandırması birbirinin yerine kullanılamaz.

Çalışma zamanı doğrulaması için paket içeriğini `pacman -Ql <package>` ile karşılaştırın ve etkin kullanıcı geçersiz kılmalarını ayrı ayrı inceleyin.

Yerel yapılar, Cargo nesnelerini `build/cargo-target`'da, paket kaynak ağacının dışında tutar, böylece tekrarlanan bir `make build` yalnızca değişenleri yeniden derler. `make clean` bu dizini kaldırır ve sonraki derleme soğuk olarak başlar.

Çalışma zamanı yükünü yalnızca depo adından çıkarmayın. Bir paket; ikili dosyalara, kullanıcı hizmetlerine, `/usr/share/argvus` varsayılanlarına, `/etc/argvus` yönetici yapılandırmasına veya varlıklarına sahip olabilir. Kurulumdan sonra deponun `PKGBUILD` ve `pacman -Ql` değerleri ile gerçek yükü doğrulayın.