---
title: Sanal makineler
description: ARGVUS'u sanal grafik donanımıyla çalıştırın.
---

`argvus start` sanallaştırmayı algılar ve algılanan ortam gerektirdiğinde yazılım oluşturma ve imleç/değiştirici geri dönüşleri de dahil olmak üzere etkilenen oturumlar için uyumluluk ayarlarını dışa aktarır.

Bir sanal makine hâlâ oluşturulamıyorsa geçersiz kılma eklemeden önce oturum ortamını ve günlükleri inceleyin. Bkz. [grafik sorunlarını giderme](/docs/user-guide/troubleshooting/graphics-and-virtual-machines/).Sanallaştırma algılandığında `argvus start`, yazılım oluşturma, yazılım imleci ve değiştirici geri dönüşleri dahil olmak üzere oturuma yönelik uyumluluk değişkenlerini dışa aktarır. Bu, normal oturum açma yolunu denenecek ilk şey haline getirir; Masaüstü bir VM'de çalışıyor diye kalıcı bir geçersiz kılma eklemeyin.

Algılama yeterli değilse tanılama geri dönüşü olarak kullanıcı Hyprland geçersiz kılma özelliğini kullanın:

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

Bunu yalnızca VM grafik yığını gerektirdiğinde ekleyin:

```lua
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

Oturumu değiştirdikten sonra çıkış yapın ve tekrar giriş yapın. Aktif değeri `echo $LIBGL_ALWAYS_SOFTWARE` ile doğrulayın; yazılım ile rendering masaüstünü kullanılabilir hale getirebilir ancak performansı düşürebilir.
