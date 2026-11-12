---
title: Ağ ve Bluetooth
description: NetworkManager'ı ve Bluetooth'u ARGVUS aracılığıyla yönetin.
---

`argvus network` alt komutu NetworkManager'ı rapor eder ve kontrol eder, `argvus bluetooth` ise Bluetooth adaptörünün durumunu ve kullanılabilirliğini rapor eder ve kontrol eder. Her ikisi de `argvus` içinde işlem halinde çalışır; `networkctl` ve `bluetoothctl` takma ad olarak kalır. Denetim Masası ve Yol Çubuğu bu komutları çağırır.

Kurulu alt komutlar için her iki komutta da `--help`'yu kullanın. Ağ ve Bluetooth kullanıcı arayüzü entegrasyonu, Blueman gibi isteğe bağlı gelişmiş yöneticilerden farklıdır.

Kontrol Merkezi, ağ durumunu yapılandırma sayfalarından ayırır: **Ağ → Durum** mevcut durumu özetler; arayüzler, Wi-Fi, Ethernet, VPN, DNS, proxy ve güvenlik duvarı ise sağlayıcıları mevcut olduğunda ayrı yollardır. **Bluetooth → Durum**, **Cihazlar** ve **Eşleştirme** adaptör durumunu ve cihaz işlemlerini kapsar. Bu sayfalar sağlayıcı tarafından desteklenen sistem işlemlerini gerçekleştirir; görev çubuğu ve Denetim Masası hızlı durum/eylem yüzeyini ortaya çıkarır.

Ağ ve Bluetooth değişiklikleri yetkilendirme gerektirebilir ve NetworkManager veya Bluetooth hizmeti aracılığıyla hemen geçerli olabilir. ARGVUS görev çubuğu göstergesini değiştirmekle aynı şey değildir.

Kontrol Merkezi, `--ask` ile standart giriş yoluyla Wi-Fi şifrelerini nmcli'ye gönderir; parolalar işlem bağımsız değişkenlerinden çıkarılır ve döndürülen komut hatalarından çıkarılır.

## Komut satırı ağ yönetimi

Grafik arayüzü olmadan ağı sorgulamak ve kontrol etmek için `argvus networkctl`'yu kullanın:

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```

## Komut satırı Bluetooth yönetimi

Bluetooth adaptörü için `argvus bluetoothctl` kullanın:

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```

Çekirdek, `/sys/class/bluetooth`'da denetleyici listelediğinde, `status`, bluetoothctl'yu sormadan hemen `status=no-controller`'ya yanıt verir (aksi takdirde hizmet durdurulduğunda BlueZ'yi bekler). Görev çubuğu modülü ve Kontrol Paneli kartları aynı durumu okuyor.

Yüklü sürümünüze özel ek seçenekleri ve alt komutları görmek için her iki komutta da  `bluetoothctl` `--help` kullanın.

Wi-Fi parolası iletişim kutusu açıkken, tarama güncellemeleri seçilen SSID'yi ve güvenlik türünü korur. Bu ağ kaybolursa veya güvenliği değişirse, doğrulama bir hatayla durur ve yeni bir seçim gerektirir.

Manuel DNS, çözümleyici dosyasını güncellemeden önce IPv4 ve IPv6 sunucularını ilgili NetworkManager ayarlarına gönderir. Hizmet tarafından yönetilen çözümleyici sembolik bağlantıları, dosya değiştirmenin reddedilmesiyle korunur. Otomatik DNS'ye başarılı bir dönüş, yedeklemeyi geri yükler ve kaldırır; böylece daha sonraki bir manuel döngü, mevcut çözümleyici durumunu kaydeder.

Ağ profili düzenleyicisinin kapatılması, mevcut işinin tamamlanmasını bekler, dolayısıyla bir profilin kaydedilmesi veya silinmesi, düzenleyicinin atılmasıyla kesintiye uğratılamaz. Arka plan yenilemeleri iptal edilebilir durumda kalır.

Wi-Fi bağlantı istekleri ağ eylem korumasını paylaşır: çalışan bir eylem veya profil düzenleme işi, başka bir mutasyon başlamadan önce tamamlanır. İlerleme ve tamamlanma mesajları seçilen dili takip eder.

Bluetooth kaldırma onayı, cihaz listesi yenilendiğinde bile orijinal cihaz adresini korur. Bluetoothctl'den gelen pil yüzdeleri tanınır ve cihaz ayrıntısı sorguları sınırlı eşzamanlılık ve bireysel zaman aşımlarıyla çalıştırılır.

Wi-Fi şifresi, manuel DNS ve ağ arama alanları köşeli ayraçlı yapıştırmayı kabul eder. Yapıştırılan denetimler ve satır sonları atlanır. Desteklenmeyen Yalnızca Ctrl veya Yalnızca Alt karakter kısayolları harf eklemez. Editörler, Ctrl+Alt olarak kodlanmış yazdırılabilir AltGr olaylarını kabul eder; Ctrl+S yalnızca Alt olmadan kaydeder. Paylaşılan metin düzenleyici, görsel sütunlarda dikey gezinmeyi korur ve geniş karakterler için yatay kaydırmayı hizalar.Görev çubuğu ağ göstergesi, ilk anlık görüntü ve otomatik monitör yeniden bağlantısıyla `argvus networkctl waybar --follow` kullanarak NetworkManager olaylarını takip eder. Artık 10 saniyelik yoklama aralığını beklemiyor. Bağlı, sınırlı İnternet erişimi ve bağlantısız olmak ayrı durumlar olmaya devam ediyor. Wi-Fi durumu, tarama isteğinde bulunmadan önbelleğe alınmış erişim noktalarını okur. `argvus networkctl waybar` tek bir Waybar JSON anlık görüntüsünü yayar.
