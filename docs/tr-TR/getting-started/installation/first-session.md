---
title: İlk oturum
description: Kurulumdan sonra ARGVUS'u başlatın.
---

Grafiksel oturum açma için, `argvus greeter-setup --help` ile paketlenmiş greetd entegrasyonunu yapılandırın ve [Oturumlar](/docs/user-guide/sessions/) bölümünde açıklanan ekran yöneticisi akışını kullanın.

TTY oturumu için sanal terminalde oturum açın ve şunu çalıştırın:

```sh
argvus --start-desktop
```

Oturum açtıktan sonra ARGVUS kullanıcı hedefini incelemek için `argvus sessionctl status` kullanın. Besteci veya kabuk başlamazsa [oturum sorunlarını gidermeyi](/docs/user-guide/troubleshooting/session-startup/) uygulayın.
