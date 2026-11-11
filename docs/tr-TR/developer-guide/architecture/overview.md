---
title: Mimariye genel bakış
description: ARGVUS nasıl entegre bir masaüstü ortamı oluşturur?
---

ARGVUS'un beş çalışma zamanı katmanı vardır:

1. `argvus`'da giriş noktaları ve gönderim;
2. `argvus-session`'daki oturum yaşam döngüsü;
3. `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar` ve `argvus-control-panel`'daki kompozitör ve kabuk yüzeyleri;
4. Kontrol Merkezi, takvim ve sistem monitörü gibi kullanıcı uygulamaları;
5. görüntüleme, ağ, güç, görünüm, hesaplar ve bildirimlere ilişkin sağlayıcılar ve varlıklar.

Paylaşılan durum bu katmanları birbirine bağlar. `argvus-config` standart belgeye sahiptir ve `data/generated/`'nun tek yazarıdır; dolayısıyla bir tema veya efekt değişikliği, birkaç paket tarafından tüketilen dosyaları tek bir işlemde yeniden oluşturur; oturum hizmetleri, her bileşenin uygulamasının sahipliğini üstlenmeden yaşam döngüsü sahipliği sağlar.

## Sorumluluk sınırları

| Katman | Birincil sorumluluk | Örnekler |
| --- | --- | --- |
| Giriş noktaları ve sevkıyat | Masaüstü uygulamalarını başlatın ve ARGVUS profillerini çözümleyin. | `argvus`, `argvus-app-profiles` |
| Oturum yaşam döngüsü | Ortamı hazırlayın, Hyprland'ı başlatın, kullanıcı hizmet ortamını senkronize edin ve kendi oturumunuzu kapatın. | `argvus-session`, `argvus sessionctl` |
| Besteci ve kabuk | Pencere davranışını ve görünür masaüstü yüzeylerini tanımlayın. | `argvus-hyprland`, `argvus-waybar`, `argvus-taskbar`, `argvus-control-panel` |
| Yapılandırma | `config.json`'ya, yol modeline ve oluşturulan her tüketici dosyasına sahip olun. | `argvus-config` |
| Uygulamalar | Ayarlar ve odaklı masaüstü iş akışları sağlayın. | Kontrol Merkezi, takvim, başlatıcı, sistem monitörü ve çıkarılabilir cihazlar |
| Sağlayıcılar ve varlıklar | Etki alanı işlemlerini ve paylaşılan görsel kaynakları uygulayın. | ekran, ağ, güç, bildirimler, görünüm, yazı tipleri ve duvar kağıtları |

`argvus` paketi kurulum hedefi ve koordinatörüdür ancak monolitik bir çalışma zamanı değildir. Odaklanmış paketler ikili dosyalarının, yapılandırmalarının ve hizmetlerinin sahipliğini korur. Oturum yöneticisi, uygulamaları özümsemeden bunları grafiksel oturumun uygun noktasında başlatır.

Bu ayrım hata ayıklama sırasında önemlidir: bir oturum hatası ilk olarak `argvus-session`'ya aittir; `argvus-taskbar`/`argvus-waybar`'ya ait bir görev çubuğu hatası; ve oluşturulan bir tema veya duvar kağıdı hatası, görünüm ve oturum yapılandırma yoluna aittir.