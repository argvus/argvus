---
title: Katkıda Bulunmak
description: ARGVUS projeleri ve belgeleri için katkı sınırları.
---

Bir özelliği değiştirmeden önce tüm tüketen depolarda izleyin. Paylaşılan durum sözleşmelerini, paket sahipliğini, hizmet sahipliğini ve her iki yerelleştirme kataloğunu koruyun.

`argvus-config`, `data/generated/`'nun tek yazarıdır. Bir kabuk yardımcısına değil, `crates/argvus-config/src/project.rs`'ya ve `project/` (`project/hyprland_projection.rs`'daki Hyprland dosyaları) altındaki alt modüllerine bir tüketici dosyası ekleyin veya değiştirin. Bir tüketici uygulaması `config.json`'yu okuyamıyorsa, sahip olunan pakete bir bağdaştırıcı ekleyin ve kurallı işleme sonrasında onu orkestratörden çağırın. `data/generated/`'yu silin, `argvus config apply`'yu çalıştırın ve ağacın `config.json`'dan bayt bayt yeniden oluşturulduğunu doğrulayın. Kaynağı, paket yükünü ve canlı oturum davranışını ayrı ayrı doğrulayın; tek başına bir kaynak yapısı, yüklü bir Wayland oturumunun yeni dosyaları kullandığını kanıtlamaz.