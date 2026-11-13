---
title: Kontrol Merkezi
description: ARGVUS masaüstünüzün kalıcı bölümlerini yapılandırın.
---

Kontrol Merkezi, ARGVUS'un klavye öncelikli ayarlar uygulamasıdır. Tek seferlik bir işlem gerçekleştirmek yerine masaüstünü yapılandırmak istediğinizde bunu kullanın.

## Kontrol Merkezi ve Kontrol Paneli

| Kullan… | İçin… |
| --- | --- |
| **Kontrol Merkezi** | Temalar, duvar kağıtları, efektler, düzen, görev çubuğu davranışı, yazı tipleri, varsayılan uygulamalar, klavye ve giriş ayarları, dil, bölge ve sistem yapılandırması. |
| **Kontrol Paneli** | Mevcut durum ve ses seviyesi, parlaklık, ağ, bildirimler, güç ve oturum kontrolleri gibi sık yapılan işlemler. |

Kontrol Paneli, Kontrol Merkezi'nde kalıcı olarak yapılandırılan bir ayar için hızlı kontrolleri ortaya çıkarabilir. Örneğin, hızlı bir görünüm veya görüntüleme eylemi için paneli, arkasındaki masaüstü yapılandırmasını değiştirmek istediğinizde ise Kontrol Merkezi'ni kullanın. Bkz. Denetim Masası ve [İşlemlerin nerede yapılandırılacağı](/docs/user-guide/where-to-configure/).

## Ana alanlar

Mevcut Kontrol Merkezi ana sayfası şu alanları sağlar:

Ev, altı kartta gruplandırılmış varış noktalarının bir kataloğudur. Her hedef kendi sayfasını doğrudan açar ve arama, onu herhangi bir kartın altında bulur.
- **Görünüm** — masaüstünü özelleştirin: Masaüstü Stili (tema, görünüm modu, vurgu rengi, duvar kağıtları, simgeler ve yazı tipleri), Uygulama Stili (terminal ve başlatıcı şeffaflığı), Görev Çubuğu, Denetim Masası ve Widget Telemetrisi.
- **Masaüstü ve Giriş** — çalışma alanını yapılandırın: Hyprland (Windows: Kurallar, Boşluklar ve Mod, animasyonlar, bulanıklaştırma, kenarlıklar, gölge ve yüklü sürüm), Ekranlar, Klavye ve Kısayollar ve Fare ve Dokunmatik Yüzey.
- **Tercihler** — kişisel ve bölgesel ayarlar: Dil ve Bölge (dil, saat dilimi, formatlar ve klavye düzeni merkezi), Tarih ve Saat, Varsayılan Uygulamalar, Güç ve Oturum.
- **Donanım ve Bağlantı** — cihazlar ve bağlantılar: Donanıma Genel Bakış, Ağ ve Wi-Fi, Bluetooth, Ses ve Depolama.
- **Sistem Yönetimi** — işletim sistemini yönetin: Paketler, Hizmetler, Önyükleme, Ana Bilgisayar Adı, Kullanıcılar ve Gruplar, Güvenlik Duvarı ve Bildirimler (Rahatsız Etmeyin).
- **Ayarlar** — Kontrol Merkezinin kendisi, kartlardan sonra listelenir: Dil (kullanıcı dili) ve Stil (dekoratif simgeler ve şeffaflık). Açmak için `s` tuşuna basın.
- **Araçlar ve Tanılama** — geliştirici ve bakım araçları: ARGVUS Hoş Geldiniz, Projeler, Parçacıklar (`argvus snippets` için kaydedilen metin, `SUPER + ALT + N` ile yazılan metin), SSH anahtarları (`~/.ssh`'daki `argvus keys` tarafından yönetilen anahtar çiftleri), GPG anahtarları (tarafından yönetilen anahtarlık) `argvus gpg`; gpg-agent pinentry-gtk), Dev Services'i, System Doctor'u ve ARGVUS Hakkında'yı kullanır.

Rahatsız Etmeyin etkinleştirildiğinde bildirim açılır pencereleri kapatılır ve yeni bildirimler daha sonra kullanılmak üzere saklanır. Kontrol Paneli, Rahatsız Etmeyin devre dışı bırakılana kadar bildirim geçmişini gizler.

## ARGVUS Hoş Geldiniz

**Araçlar ve Teşhis** altında **ARGVUS Hoş Geldiniz'i açın**'ı seçin, `welcome` arayın veya `argvus welcome`'yu çalıştırın. Hoş Geldiniz, Kontrol Merkezine entegre edilmiştir ve tek penceresini kullanır. Enter seçilen ayarlar sayfasını açar; İşiniz bittiğinde Esc tuşuyla kılavuza dönün. F6 Sonraki, geçerli adımı işaretler ve aynı pencerede bir sonraki adımı açar. Esc yalnızca geri gider ve kontrol listesindeki Enter her zaman seçilen sayfayı açar. O adımı tekrar gözden geçirir, S onu işaretlemeden atlar ve F, isteğe bağlı adımlar beklemede olsa bile kılavuzu tamamlar. İptal etmek veya iade etmek bir adımı tamamlamaz. Mevcut ilerleme korunur.

Kılavuz, dil ve bölgeyi, ekranları, donanımı ve sürücüleri, ağı, sesi, Bluetooth'u, görünümü, gücü, varsayılan uygulamaları ve oturum başlatmayı kapsar. Hiçbir zaman sürücü veya paket yüklemez ve sistem değişikliklerini tek başına uygulamaz. İlerleme `$XDG_STATE_HOME/argvus/welcome/progress.json` (veya `~/.local/state/argvus/welcome/progress.json`) dosyasında saklanır.Masaüstü hazır olduktan sonra oturum TTY, ARGVUS Greeter ve diğer oturum açma yöneticileri için `argvus welcome --if-needed`'yu çalıştırır. Kullanıcı başına `opened` ilerleme kaydı, tamamlama veya işten çıkarma kaydedilmediği sürece Hoş Geldiniz açılır. İlk başarılı oluşturma, `opened`'yu `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`'ya kaydederek daha sonraki oturum açmalarda yeniden açılmasını önler. `/etc/skel`'ya bağlı değildir. Bu kayda sahip olmayan mevcut hesaplar da bir kez otomatik olarak açılır. Oluşturmadan önceki hatalar, bir sonraki oturum açmada başka bir denemeye izin verir. `argvus welcome` kılavuzu mevcut olmaya devam ediyor.

Tam sağlayıcı sayfaları, kurulu paketlere ve izinlere bağlı olabilir. Eksik bir isteğe bağlı etki alanının bozuk bir Kontrol Merkezi olduğunu varsaymayın.

Izgarada ekrana sığmayacak kadar fazla giriş olduğunda, aşağıda daha fazlasının olduğunu göstermek için sağ kenarda bir kaydırma çubuğu görünür.

## Bir ayar bulma

Ana ekran, ayarları başlığa, kategoriye, tanımlayıcıya ve anahtar sözcüklere göre kaydeder. Arama, vurguya duyarlı değildir ve `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network` gibi pratik terimleri kabul eder. `user` ve `DPI`. Tam başlık eşleşmeleri ilk olarak sıralanır, ardından bir başlık öneki, yazılan metni içeren başlık veya kategorideki herhangi bir kelime ve son olarak anahtar kelime eşleşmeleri gelir; böylece bir kelimenin herhangi bir bölümünü (yalnızca başlangıcını değil) yazarak yine de hedefi bulabilirsiniz.

Görünür menü yeteneklere duyarlıdır. Sağlayıcısı kurulu olmadığında, donanım mevcut olmadığında veya mevcut oturum gerekli yeteneği sağlayamadığında bir rota eksik olabilir. Bu nedenle arama, bu kurulumun neyi yapılandırabileceğini keşfetmenin de yararlı bir yoludur.

Ana ayarlar ağacı şu anda şunları içerir:

| Alan | Güncel varış noktaları |
| --- | --- |
| Görünüm | Masaüstü Stili: tema, görünüm modu, vurgu rengi, duvar kağıtları, simgeler ve yazı tipleri; Uygulama Stili: terminal ve başlatıcı şeffaflığı; Görev Çubuğu; Kontrol Paneli; Widget Telemetrisi |
| Masaüstü ve Giriş | Hyprland (Windows: Kurallar, Boşluklar ve Mod, animasyonlar, bulanıklaştırma, kenarlıklar ve gölge), Ekranlar, Klavye ve Kısayollar, Fare ve Dokunmatik Yüzey |
| Tercihler | Dil ve Bölge (dil, saat dilimi, formatlar, sistem yerel ayarları ve klavye düzeni), Tarih ve Saat, Varsayılan Uygulamalar, Güç, Oturum |
| Donanım ve Bağlantı | Donanıma Genel Bakış (özet, CPU, GPU, bellek, cihazlar), Ağ ve Wi-Fi, Bluetooth, Ses, Depolama |
| Sistem Yönetimi | Paketler, Hizmetler, Önyükleme, Ana Bilgisayar Adı, Kullanıcılar ve Gruplar, Güvenlik Duvarı, Bildirimler |
| Ayarlar | Dil (kullanıcı dili), Stil (simgeler ve şeffaflık) |
| Araçlar ve Teşhis | Projeler, Snippet'ler, SSH anahtarları, GPG anahtarları, Dev Hizmetleri, Sistem Doktoru, ARGVUS Hakkında |

Bunlardan bazıları tam sayfalar, bazıları ise arama veya alan adı özeti yoluyla ulaşılan alt sayfalardır. Tam liste, kurulu ARGVUS sağlayıcıları tarafından belirlenir.

Her sayfanın yolu başlıkta gösterilir; örneğin `Control Center > Hardware & Connectivity > Audio`. Yol, sayfayı listeleyen Ana Sayfa kartıyla başlar; dolayısıyla aynı sayfa Ana Sayfadan, aramadan ve CLI'den aynı şekilde okunur. Kök ve kart adları çevrilir: Portekizce arayüzde `Central de Controle > Hardware e conectividade > Áudio` gösterilir.

## Her alan ne işe yarar?

### GörünümGörünüm ana özelleştirme alanıdır. Masaüstü grubu temayı (Yapışkan veya Aralıklı moduyla), görünüm modunu, vurgu rengini, duvar kağıtlarını, simge temasını ve Yazı Tiplerini içerir ve Uygulama Stili grubu terminal ve başlatıcı şeffaflığını tutar. Görev Çubuğu, Denetim Masası ve Widget Telemetrisi doğrudan evdeki Görünüm altında listelenir ve Görünümün paylaşılan şeffaflık sayfasını kendi yüzeyleri için yeniden kullanır. Bulanıklaştırma, Hyprland > Bulanıklaştırma altındaki genel bir ayardır ve bulanıklaştırmanın etkinleştirildiği her ARGVUS yüzeyi için geçerlidir. Bu sayfa, Etkinleştir anahtarını ve Hyprland bulanıklık değerlerini (Boyut, Geçişler, Parlaklık, Gürültü, Kontrast, Canlılık ve Canlılık koyuluğu) taslak olarak düzenler ve **Uygula** seçilene kadar hiçbir şey değişmez. Animasyonlar, Hyprland > Animasyonlar altında aynı şekilde çalışır: Etkinleştirme anahtarı, Özel eğriler (Bézier, Yay, Doğrusal ve Kolaylaştırma) ve beş animasyon kategorisi, **Uygula** ile birlikte uygulanan bir taslak oluşturur. Bézier ve Spring'in düzenlenebilir parametreleri vardır (Bézier noktaları ve yay kütlesi, sertlik ve sönümleme); Doğrusal ve Hareket Hızı sabit eğrilerdir, dolayısıyla sayfaları yalnızca kontrol noktalarını listeler ve düzenlenemez. Stil seçimleri Hyprland 0.56.2'nin çizimlerini takip ediyor: Açılış ve Kapanış slayt, popin ve gnomed sunar, katman girişi ve çıkışı slayt, popin ve fade sunar ve çalışma alanı yaprakları slayt, fade, Slidefade, Slidevert ve Slidefadevert sunar. Bu kontroller, yalnızca Kontrol Merkezi penceresini değiştirmek yerine görünüm ve oturum bileşenleriyle koordineli çalışır.

### Görev Çubuğu

Görev çubuğu, panelin konumunu, etrafındaki boşluğu ve pencerelerini, yüksekliğini ve kenarlığını, yardımcı simge grubunu, tarih/saat bloklarını ve ayrıca kendi şeffaflığını yapılandırır. Evdeki **Görünüm** altında listelenmiştir. Yapışkan modda, **Boşluklar > Alt** varsayılan olarak `2` şeklindedir; Spaced, `18` piksel taban çizgisini korur.

**Yükseklik** 24, 28 (varsayılan), 32 veya 36 pikseli seçer. **Kenarlık**, **Etkinleştir** anahtarına ve varsayılan olarak 1 olmak üzere 1 ila 3 piksellik **Kalınlık**'a sahiptir. Kenarlık kapalıyken görev çubuğu penceresi, sağ grup ve MPRIS oynatıcısında kenarlık yoktur. **Uygula** seçilene kadar bu değişiklikler taslak niteliğindedir. Tema değiştiğinde uygulanan yükseklik ve kenarlık korunur; uygulanana kadar tema belirler.

**Simgeler** sayfası **İsteğe bağlı modüller**, **Çalışma alanı sunumu**, **Başlatıcı** ve **Yardımcı Programlar**'ı açar. İsteğe bağlı modüller ayrı ayrı gösterilebilir veya gizlenebilir: ses oynatıcı, Odak, başlatıcı, ağ, güç profili, klavye düzeni, bellek, CPU ve sıcaklık okumaları, tarih, saat, pil, sistem tepsisi, bildirimler, çıkarılabilir cihazlar, kayıt, Bluetooth, ses girişi/çıkışı ve uyanık tutma. Çalışma alanları kullanılabilir durumda kalır ve adlarını (varsayılan) veya uygulama simgelerini gösterebilir. Odaklanılan pencere başlığı, Ayarlar ve Güç eylemleri her zaman kullanılabilir. Başlatıcı görünürlüğü İsteğe bağlı modüllerde kontrol edilir; alt menüsü özel simgeyi değiştirir. Değişiklikler, görev çubuğunu yeniden başlatan **Uygula** işleminden sonra etkili olur.

### Kontrol Paneli

Denetim Masası, hızlı eylem kartlarının görünürlüğünü ve sırasını, ayrıca kendi oturumlarını ve şeffaflığını yapılandırır.

Kart ayarları okunduğu anda sayfa açılır ve giriş kabul edilir. Donanıma (Bluetooth ve Parlaklık) bağlı kartlar arka planda kontrol edilir: bu kontrol bitene kadar bu pencerede en son görülen kullanılabilirliği korurlar ve sonuç geldiğinde liste kendi kendine güncellenir.

### Widget Telemetrisi

Widget Telemetrisi, sistem bilgisi widget'ının oturumlarını ve şeffaflığını yapılandırır.

### Yazı tipleriYazı tiplerine **Görünüm > Masaüstü Stili > Yazı Tipleri**'nden erişilir. Görev çubuğu, telemetri/sistem bilgileri, Kontrol Paneli, ARGVUS sistem arayüzü, uygulamalar, terminal ve tarayıcı için bir aile ve boyut seçmenizi sağlar. Ayrıca kenar yumuşatma, ipucu, alt piksel modu ve DPI gibi yazı tipi oluşturma ayarlarını da ortaya çıkarır. Varsayılan birincil aile IBM Plex Mono'dur; bireysel hedefler bağımsız olarak değiştirilebilir ve Yazı Tipleri sayfasından geri yüklenebilir.

### Varsayılan uygulamalar

Varsayılan uygulamalar, terminal, dosya yöneticisi, metin düzenleyici, terminal düzenleyici, tarayıcı, resim görüntüleyici, PDF görüntüleyici, video oynatıcı, ses oynatıcı, arşiv aracı ve başlatıcı gibi roller için yüklü programları seçer. Seçim ARGVUS tarafından kaydedilir ve standart XDG ilişkilendirmelerini güncelleyebilir. TUI dosya yöneticileri, metin dosyalarını açarken seçilen terminal düzenleyiciyi kullanır; grafiksel metin düzenleyicileri masaüstü dosya ilişkilendirmeleri için varsayılan olarak kalır.

### Parçacıklar

Parçacıklar, `argvus snippets`'nun `SUPER + ALT + N` ile yazdığı metnin sayfasıdır. **Araçlar ve Teşhis → Parçacıklar**'dadır (`snippets`'yu arayın veya `argvus control-center snippets`'yu çalıştırın).
- **Yeni kod parçası** — **Ad** ve **İçerik** yazın, ardından **Parçacığı kaydet**. Zaten var olan bir ad, söz konusu parçanın içeriğinin yerine geçer.
- **Kayıtlı snippet'ler** — konumları (`1  email`) ve içerikleri tek satırda listelenir. İçeriğe girin, onu düzenleyin. **Type** pasajı yazar ve **Kaldır** onay ister.
- **Seçiciyi aç** — `SUPER + ALT + N` ile aynı seçiciyi açar.

Yazma, Enter'a bastıktan 3 saniye sonra başlar çünkü o anda odak noktası Kontrol Merkezinin kendisidir. Bu süre içinde metnin alınması gereken alanı tıklayın. Sayfa yalnızca tek satırlık içeriği düzenler; `argvus snippets add` içeren bir terminalde çok satırlı bir kod parçası oluşturulabilir.

Sayfa, `argvus-config`'daki girişleri okur ve bunları `argvus snippets` aracılığıyla yazar, böylece başlatıcı doğrulamayı korur. `argvus snippets` kurulu olmadığında sayfada öyle yazıyor.

### SSH anahtarları

SSH anahtarları, `argvus keys`'nun yönettiği anahtar çiftlerinin sayfasıdır. **Araçlar ve Tanılama → SSH anahtarlarında** bulunur (`ssh keys`'yu arayın veya `argvus control-center ssh-keys`'yu çalıştırın).

Sayfanın yukarıdan aşağıya doğru dört bölümü vardır:

```
Search        filters the key pairs by name or comment
Generate      name, type and the Generate key pair action
Keys          one block per key pair; a scrollbar appears when they do not fit
Actions       Refresh and Unload all keys from agent
```

#### Nasıl kullanılır?

1. **Kontrol Merkezi → Araçlar ve Teşhis → SSH anahtarlarını** açın. Ana Sayfadan `ssh keys`'yu da arayabilirsiniz.
2. **Bir anahtar bulun:** `/` tuşuna basın, ardından yalnızca adı veya yorumu eşleşen çiftleri göstermek için yazın. `Enter` aramayı sonlandırır ve diğer listelerde olduğu gibi vurgulanan satırı çalıştırır; bu nedenle basmadan önce satırı kontrol edin; `Esc` aramayı temizler.
3. **Bir çift oluşturun:** **Oluştur**'da **Ad**'ı seçin, `Enter` tuşuna basın, adı yazın ve `Enter` tuşuna tekrar basın. **Tip** seçeneğini seçin ve `Enter` tuşuna basarak `ed25519` ve `rsa` arasında geçiş yapın. **Anahtar çifti oluştur** seçeneğini seçin ve `Enter` tuşuna basın. Yeni çift, parola olmadan aynı anda **Anahtarlar** altında görünür.
4. **Genel anahtarı kopyalayın:** çiftin bloğunda **Genel anahtarı kopyala** seçeneğini seçin ve `Enter` tuşuna basın. `.pub` dosyasının içeriği, sunucunun `authorized_keys` veya Git ana bilgisayarına yapıştırılmaya hazır şekilde Wayland panosuna gider. Özel anahtar asla kopyalanmaz. Bunun için `wl-clipboard` (`wl-copy`) gerekir; onsuz, sayfa anahtarın kopyalanmadığını söylüyor.
5. **Bir çifti aracıya yükleyin:** çiftin bloğunda **Aracıya yükle** seçeneğini seçin ve `Enter` tuşuna basın. Parolası olmayan bir anahtar, pencere olmadan tek seferde yüklenir. Parola içeren bir anahtar, OpenSSH'nin bunu istediği bir ARGVUS terminalini açar; anahtarın yüklendiğini söylediğinde terminali `Enter` ile kapatın. Sayfa, açıkken aracıyı birkaç saniyede bir kontrol eder, böylece satır kendiliğinden **Ajandan kaldır** olarak değişir. Tek seferde güncellemek için **İşlemler** bölümünde **Yenile** seçeneğini kullanın.6. **Anahtarı temsilciden kaldırın:** çiftin bloğunda **Temsilciden kaldır** seçeneğini seçin. Her anahtarı kaldırmak için **İşlemler** bölümünde **Tüm anahtarları aracıdan kaldır** seçeneğini kullanın. Anahtar dosyalar diskte kalır.
7. **Bir çifti silin:** çiftin bloğunda **Sil**'i seçin ve `Enter` tuşuna basın. Onay **İptal**'e odaklanır, dolayısıyla başıboş bir `Enter` hiçbir şey yapmaz. Onaylamak için `y` tuşuna veya iptal etmek için `n`/`Esc` tuşuna basın. Aracıya yüklenen bir anahtar silinemez; önce onu boşaltın.

Dolaşma: `↑`/`↓` veya `j`/`k` listede gezinin, `Enter` bir satırı etkinleştirir, `Esc` geri döner ve `?` yardımı gösterir. `Tab` odağı **Eylemler**'e taşır; burada `←`/`→` bir düğme seçer ve `Enter` onu çalıştırır. `↑`/`↓` odağı listeye geri getirir.

Parolalar hiçbir zaman Kontrol Merkezine yazılmaz ve hiçbir zaman komut bağımsız değişkenleri olarak iletilmez. Sayfa, listeyi `argvus keys list` ile okur ve yalnızca `argvus keys` aracılığıyla yazar. `argvus-keys` kurulu olmadığında sayfada öyle yazıyor.

#### Sorun giderme

| Belirti | Nedeni ve çözümü |
| --- | --- |
| `Could not open a connection to your authentication agent` **Acenteye yükleme** sonrasında | Terminal aracı soketini alamadı. Hyprland başladığı ortamı korur ve aracı daha sonra başlamış olabilir. $XDG_RUNTIME_DIR/ssh-agent.socket'ya geri dönen `argvus-launcher`'yu güncelleyin ve  `$XDG_RUNTIME_DIR/ssh-agent.socket` `systemctl --user status argvus-ssh-agent.service`'yu kontrol edin. |
| **Ajan** `Agent unavailable`'yu gösterir | Başlatıcı aracıya ulaşamıyor. Hizmeti yukarıdaki gibi kontrol edin. |
| Sayfada `argvus-keys is not installed` | `argvus` paketini güncelleyin: `argvus keys` bunun bir alt komutudur. |
| **Sil** bir hata gösteriyor | Anahtar aracıya yüklenir ve saklanır. Önce kaldırın, sonra tekrar silin. |

### GPG anahtarları

GPG anahtarları, `argvus gpg`'nun yönettiği anahtarlığın sayfasıdır. **Araçlar ve Tanılama → GPG anahtarları**'ndadır (`gpg keys`'yu arayın veya `argvus control-center gpg-keys`'yu çalıştırın).

Sayfada yukarıdan aşağıya şu bölümler bulunur:

```
Pinentry         the pinentry gpg-agent uses, for information only: pinentry-gtk
Generate key ▾   full name, e-mail, comment, algorithm, key strength, expiration date,
                 passphrase, and the Generate key and Cancel actions (expanded at first)
Import ▾         Import public key… and Import private key… (expanded at first)
Keys ›           the keyring, on its own page, with the search at the top
```

`▾` genişletilmiş bir bölümü, `▸` ise daraltılmış bir bölümü işaret eder. `Enter` veya `Space` onu genişletir veya daraltır.

#### Nasıl kullanılır?

1. **Kontrol Merkezi → Araçlar ve Teşhis → GPG anahtarlarını** açın. Ana Sayfadan `gpg keys`'yu da arayabilirsiniz.
2. **Pinentry** `pinentry-gtk`'ya sabitlenmiştir. Sayfa sadece onu gösteriyor. Bir anahtar ilk kez üretildiğinde, ARGVUS bu pin girişini `gpg-agent.conf`'ya kendi işaret çizgileri arasına yazar ve diğer satırlarınızı saklar. Zaten bu işaretleyicilerin dışında bir `pinentry-program` satırı yazdıysanız, o satırı kaldırabilmeniz için oluşturma bir hatayla durur.
3. **Bir anahtar oluşturun:** **Anahtar oluştur ▾** alanına **Tam ad** ve **E-posta** yazın. Her ikisi de gereklidir ve **Anahtar oluştur** her ikisinde de metin bulunana kadar devre dışı kalır. **Yorum** isteğe bağlıdır. Kullanıcı kimliği `Name (comment) <e-mail>` olur.
- **Algoritma** **RSA** ve **DSA ElGamal** arasında geçiş yapar. Bir DSA anahtarı, bir ElGamal şifreleme alt anahtarını alır; bir RSA anahtarı bir RSA anahtarı alır.
- **Anahtar gücü** algoritmanın boyutları arasında geçiş yapar: RSA 1024, 2048 ve 4096; DSA 1024, 2048 ve 3072. gpg, 3072 bitin üzerinde DSA anahtarları oluşturmaz, dolayısıyla 4096, DSA için sunulmaz.
- **Son kullanma tarihi** kendi sayfasını açar. Tarihi dilinizin düzenine yazın: Portekizce, İspanyolca, Fransızca, İtalyanca, Almanca ve Rusça olarak `DD/MM/YYYY`; `YYYY/MM/DD` İngilizce, Çince, Japonca ve Korece. Tarih bugünden sonra olmalıdır. Anahtarın kalıcı olması için **Hiçbir zaman sona ermez** seçeneğini açın; tarih alanı daha sonra devre dışı bırakılır ve kullanılmaz.
- **Parola** kendi sayfasını açar. **Parolayla koru** varsayılan olarak açıktır; **Parolayı** yazın ve ardından **Parolayı onaylayın**. İkisi eşleşmelidir. Korumanın kapatılması her iki alanı da temizler ve sayfa, korumasız bir anahtar oluşturmadan önce onay ister.
4. **Anahtar oluştur**'u seçin ve `Enter` tuşuna basın. Anahtar, bir imzalama anahtarı ve bir şifreleme alt anahtarı alır. Parola, başlatıcıya hiçbir zaman komut argümanı olarak değil, standart girişinde gönderilir. Oluşturma başarısız olursa diğer alanlar kalır ancak parolalar silinir ve yeniden yazılması gerekir.
5. **İptal** formun tüm alanlarını temizler ve tarihleri ​​ve boyutları varsayılan değerlerine döndürür.
6. **İçe aktar:** **İçe aktar ▾** bölümünde **Genel anahtarı içe aktar…** veya **Özel anahtarı içe aktar…** seçeneğini seçin. Dosyayı seçebilmeniz için dosya yöneticisi açılır. Özel anahtar önce onay ister ve anahtar kullanıldığında gpg tarafından parolası sorulur. Dosya yöneticisi grafiksel ise dosyayı sayfaya bildiremez; sayfa daha sonra yolu yazacağınız bir alan açar.
7. **Anahtarlar ›** anahtarlığı açar. Her anahtarın kimliğini, parmak izini, türünü, son kullanma tarihini ve özel anahtarının anahtarlıkta olup olmadığını gösterir. Her tuşta şu eylemler bulunur:
   - **Anahtarı düzenle ›** anahtarın kendi sayfasını açar (aşağıda açıklanmıştır).
- **Genel anahtarı dışa aktar…**, `~/<fingerprint>.asc` olarak önerilen bir dosya yolu ister. Dosya `600` moduyla oluşturulmuştur. Mevcut bir dosya asla değiştirilmez.
- **Özel anahtarı dışarı aktar…** yalnızca özel kısmı olan anahtarlar için görünür. Onay ister, ardından `~/<fingerprint>-private.asc` olarak önerilen dosya yolunu ister. Dosya `600` moduyla oluşturulmuştur.
- **Genel anahtarı kopyalayın** ve **parmak izini kopyalayın** `wl-copy` ile Wayland panosuna kopyalayın.
- **Sil tuşu…**, İptal'e odaklanarak onay ister. Anahtarı ve özel kısmını kaldırır.
8. **Bir anahtar bulun:** **Anahtarlar** sayfasında, üstteki arama çubuğu tuşları filtreler. Listeyi daraltmak için yazın; arama, kimlik (yorumu içeren) ve parmak iziyle eşleşir. `Esc` aramayı temizler.
9. **Anahtarı düzenle ›** anahtarın ayrıntılarını ve ardından şu eylemleri gösterir:
   - **Son kullanma tarihi ›** anahtarın ve alt anahtarlarının her birinin son kullanma tarihini değiştirir. Son kullanma tarihini kaldırmak için **Hiçbir zaman sona ermez** seçeneğini kullanın. **Uygula** değişikliği kaydeder.
- **Parolayı değiştir** pinentry penceresinde eski ve yeni parolayı sorar.
- **Anahtarı ›** ile imzala, bu anahtarı özel kısmı olan diğer anahtarlarınızdan biriyle onaylar. İmzalayanın parolası sorulur. Sayfa önce onay ister.
- **Anahtarı iptal et ›** nedenleri listeler: neden yok, anahtar ele geçirildi, anahtar değiştirildi, anahtar artık kullanılmıyor. Her biri onay istiyor. İptal edilen anahtar bir daha kullanılamaz.
- **Güven** anahtara ne kadar güveneceğinizi belirler: ayarlanmamış, marjinal, tam veya nihai.
- **Anahtar sunucusuna yayınla…** onaylandıktan sonra genel anahtarı `gpg.conf`'da ayarlanan anahtar sunucusuna gönderir. **Anahtar sunucusundan getir** anahtarı söz konusu anahtar sunucusundan indirir.

Dolaşmak: `↑`/`↓` veya `j`/`k` listede gezinin, `Enter` bir satırı etkinleştirir, `Esc` geri döner ve `?` yardımı gösterir.

Parolalar yalnızca parola sayfasının maskelenmiş alanlarına veya pinentry penceresine yazılır. Sayfa, oluşturma parolasını standart girişte `argvus gpg`'ya gönderir ve başlatıcı bunu bir dosya tanımlayıcıda gpg'ye verir, böylece işlem listesinde, komut bağımsız değişkeninde veya günlüklerde hiçbir zaman görünmez. Önemli değişiklikler arka planda çalışır; biri koşarken diğerleri bekler. Sayfa, `argvus gpg list` ile anahtarlığı okur ve yalnızca `argvus gpg` aracılığıyla yazar. `argvus gpg` kurulu olmadığında sayfada öyle yazıyor.

Henüz sayfada değil: anahtar sunucusunda ada veya e-postaya göre anahtar aranıyor. Bilinen bir parmak iziyle **Anahtar sunucusundan getir** seçeneğini kullanın.#### Sorun giderme

| Belirti | Nedeni ve çözümü |
| --- | --- |
| Sayfada `argvus gpg is not available` | Yüklenen `argvus-launcher`, GPG komutundan öncedir. `argvus gpg list`'yu bir terminalde çalıştırın: `unknown command`, paketin yeniden oluşturulması ve kurulması gerektiği anlamına gelir. Anahtarların ayrıca `gnupg`'ya ihtiyacı vardır. |
| **Anahtar oluştur** devre dışı kalır | Hem **Tam adı** hem de **E-posta**'yı yazın. |
| Durum satırında `argvus-gpg: pinentry-gtk is not installed` | pinentry-gtk sağlayan `pinentry` paketini yükleyin. |
| Oluşturma  `pinentry-gtk` `pinentry-program` hatasıyla duruyor | `gpg-agent.conf`, ARGVUS bloğunun dışına bir pin girişi ayarlar. Bu satırı kaldırın; ARGVUS daha sonra kendi yazar. |
| **Pinentry** `Not set`'yu gösteriyor | gpg-agent, sayfadan ilk anahtar oluşturulana kadar yerleşik varsayılanını kullanır. |
| Sayfa, yüklü `argvus-launcher`'nun Kontrol Merkezinden daha eski olduğunu söylüyor | `argvus-launcher`'yu yeniden yükleyin: deposunda `make build`'yu ve ardından `make install`'yu çalıştırın ve sayfayı yeniden açın. Sürümler eşleşene kadar temel işlemler kapalı kalır. |
| Dosya yöneticisi açılıyor ancak sayfa bir yol istiyor | Dosya yöneticisi grafikseldir ve seçilen dosyayı raporlayamaz. Alana yolu yazın. |
| **Genel anahtarı kopyala** kopyalanamayacağını söylüyor | `wl-copy` yüklü değil veya Wayland oturumu kullanılamıyor. `wl-clipboard`'yu yükleyin. |
| **Anahtarı düzenle ›** anahtarın artık anahtarlıkta olmadığını söylüyor | Sayfa açıldığından beri anahtarlık değişti. **Tuşlar ›**'ya geri dönün ve yenileyin. |
| Durum satırında `argvus-gpg: ...` | Başlatıcı eylemi reddetti ve metinde geçersiz süre sonu, zaten var olan bir dosya veya eksik `--yes` gibi nedenler belirtiliyor. |
| Bir eylem bekliyor ve hiçbir şey olmuyor | Pinentry penceresi gizlendi veya açılmadı. Oturumun `WAYLAND_DISPLAY` veya `DISPLAY`'yu gösterdiğini kontrol edin. |
| İçe aktarma reddedildi | Dosya seçilen türden bir anahtar içermiyor. Genel anahtarlar **Genel anahtarı içe aktar…** üzerinden geçer; **Özel anahtarı içe aktar…** aracılığıyla özel anahtarlar. |

### Dil ve Bölge

Bu kartta Dil (bölgesel yerel ayar), Saat Dilimi, Tarih ve Saat, Sistem Yerel Ayarları ve Klavye Düzeni listelenir ve her biri doğrudan evden açılır. Klavye sayfaları aynı zamanda varyant ve konsol tuş haritasını da içerir. Ana sayfa araması ayrıca klavye kısayollarını ve fare/dokunmatik yüzey ayarlarını doğrudan ayar yolları olarak ortaya çıkarır.

Bu kart dili listeler Dil (the This card list Language (the This card ARGVUS arayüz dili), Bölgesel Locale, Time Zone, Date & Time, System Locales ve Klavye Layout. Dil 18 dilli seçici açılır; Bölgesel Locale bölgesel formatlar ve üçüncü taraf uygulamalar için kullanılan yerel ayarlar.

**Preferences > Dil & Bölge > Dil** kurtarır `ARGVUS_LANGUAGE` in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in in `$XDG_CONFIG_HOME/locale.conf` (normal olarak `~/.config/locale.conf`) `ARGVUS_LANGUAGE=pt-PT`.Değişir ARGVUS Uygulamaları ve yüzeyleri korurken `LANG` Bölgesel formatlama ve üçüncü taraf uygulamaları için. Kontrol Panel hemen yeniden başlar; oturum bileşenleri boyunca dili uygulamak için yeni bir seans başlatır. Bir arayüz dili seçmek yönetici izni gerektirmez veya `locale-gen`.

**Regional Locale** kurtarır `LANG` Aynı dosyada ve bölgesel formatları ve üçüncü taraf uygulamaları tarafından kullanılan dili kontrol eder. Seçilmiş onun seçilmiş UTF-8 Sistem yereli zaten oluşturulmalıdır.

**Sistem Locales ** değişiklikler `/etc/locale.gen` Ve çalışır ve çalışır `locale-gen`Ancak bunu uygulamak yönetici izni gerektirir.

**Regional Locale** aynı dosyada ve bölgesel formatları ve üçüncü taraf uygulamaları tarafından kullanılan dili kurtarır. Seçilen UTF-8 sistemi yerele zaten oluşturulmalıdır.

**Sistem Yerel Ayarları**, `/etc/locale.gen`'yu değiştirir ve `locale-gen`'yu çalıştırır; dolayısıyla bunu uygulamak için yönetici yetkilendirmesi gerekir. Altbilginin üzerindeki kalıcı **[WARN]** bildirimi, yönetim ayarlarına sahip sayfaları tanımlar. Aynı bildirim, saat dilimi, tarih/saat, ana bilgisayar adı, konsol anahtar haritası, hesap ve grup düzenleme, güvenlik duvarı, sistem hizmetleri, CPU yöneticisi, güç profilleri ve politikaları, önyükleme yapılandırması, paket işlemleri, anlık görüntüler ve korumalı ağ değişikliklerini kapsar. Kişisel tercihler, kullanıcı hizmetleri ve salt okunur sayfalar bu bildirimi almaz. Bildirim bilgilendirme amaçlıdır; Yetkilendirme yalnızca bir eylemin gerektirdiği durumlarda istenir.

### Donanım ve ekranlarİlgili yetenekler yüklendiğinde Kontrol Merkezi CPU, GPU, bellek, güç ve cihazlar gibi donanım bilgilerini gösterebilir. Giriş sayfası fare ve dokunmatik yüzey davranışını yapılandırır; ekran alanı, çalışan oturum tarafından desteklendiğinde monitör çözünürlüğünü, yenileme hızını, ölçeği, konumu, yönlendirmeyi, birincil ekranı, VRR ve HDR'yi yönetir. Ekran değişiklikleri hemen uygulanır. Ekranı kullanılamaz hale getirebilecek bir değişiklikten sonra (çözünürlük veya yenileme hızı, konum, yansıtma, monitörün devre dışı bırakılması, renk derinliği) veya **Uygula** sonrasında, Kontrol Merkezi odak **Geri Döndür**'e odaklanarak bu değişikliğin devam edip etmeyeceğini sorar: korumak için `y` tuşuna basın; `n`, `Esc` veya `Enter` bunu geri alır ve 15 saniye içinde yanıt alınmazsa önceki yapılandırma otomatik olarak geri gelir. Bağlantısı kesilmiş bir monitörün kayıtlı yapılandırmasını kaldırmak ve bir profili silmek için onay isteyin; profiller **Profili uygula**, **Yeniden adlandır** ve **Profili sil** ile kendi sayfalarını açar. Düzen, `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` ve `monitor-remove` aracılığıyla kaydedilir; dolayısıyla config.json'nun kanonik `/displays` bölümü monitör düzeninin tek kaynağıdır. Yalnızca profil kataloğu  `config.json` `display/config.toml`'da kalır.

### Bağlantı ve ses

Ağ alanı, mevcut olduğunda Wi-Fi, Ethernet, VPN, DNS ve proxy yolları da dahil olmak üzere kurulu ağ sağlayıcısının durumunu ve bağlantı sayfalarını gösterir. Güvenlik duvarı **Sistem Yönetimi** altında listelenir. Bluetooth, Bluetooth desteğine bağlıdır. Ses alanı, isteğe bağlı özelliği etkinleştirildiğinde ve kurulu ses hizmetiyle entegre olduğunda sağlanır.

**Ağ ve Wi-Fi** ana sayfası ve **Durum** sayfası, en yavaş olandan sonra değil, her NetworkManager sorgusu yanıtlandıkça doldurulur: önce cihaz listesi, ardından radyo, bağlı arayüzlerin adresleri ve DNS, kayıtlı bağlantılar ve NetworkManager'ın zaten bildiği Wi-Fi ağları (yeni bir tarama yalnızca `r` içeren Wi-Fi sayfasından çalıştırılır). Cihaz listesi gelene kadar satırlarda *Yükleniyor...* yazıyor ve Wi-Fi anahtarı yalnızca bilgisayarda bir Wi-Fi kartı olduğunda sunuluyor. Bir sayfa yenilenirken bir eylemin başlatılması (Wi-Fi anahtarı, bağlanma, DNS), yenilemenin yerini alır ve eylem sona erdiğinde sayfa yeniden yenilenir.

#### Bağlantılar, nmtui'den ilham aldı

**Ağ ve Wi-Fi > Bağlantılar** her NetworkManager bağlantı profilini türüne, etkin olduğu cihaza (`★ Active · eno1`), bir köprünün, bağın veya ekibin bağlantı noktası olup olmadığına ve salt okunur olup olmadığına göre gruplandırılmış şekilde listeler. `Enter`, profili düzenleyicide açar, `a` profili bağlar veya bağlantısını keser, `/` ada göre filtreler ve `r` yeniler. **Bağlantıyı etkinleştir**, nmtui'da olduğu gibi, `Enter`'nun bir profili yukarı veya aşağı getirdiği listenin aynısıdır. Bir arayüzün, kayıtlı bir Wi-Fi ağının ve bir VPN'in ayrıntı sayfalarında, profillerini açan **Bağlantıyı düzenle** satırı bulunur.

**Bağlantı ekle** Ethernet, Wi-Fi, Hotspot, DSL (PPPoE), Bridge, Bond, Team, VLAN, IP tüneli, Loopback ve WireGuard profilleri ve yüklü NetworkManager VPN eklentisi başına bir VPN profili oluşturur (OpenVPN'in kendi formu vardır; diğer eklentiler ayarlarını  `nmtui` `key=value` girişleri olarak düzenler). Ayrıca bir WireGuard `.conf` veya `.ovpn` (`nmcli connection import`) gibi bir eklenti dosyasını da içe aktarır. Ekibin `libteam` paketine, DSL'nin `ppp`'ya ve bir VPN'in eklentisine ihtiyacı vardır (örneğin `networkmanager-openvpn`); onlar olmadan satır devre dışı bırakılır ve ne kurulacağını söyler. WEP, InfiniBand, MACsec ve Open vSwitch sunulmamaktadır.Düzenleyici, katlanabilir bölümlerin bir listesidir: **Genel** (ad, arayüz, otomatik bağlantı, öncelik, tüm kullanıcılar tarafından kullanılabilir, ölçülü), türün kendi ayarları, **IPv4** ve **IPv6** (yöntem, adresler, ağ geçidi, DNS sunucuları, etki alanları, rotalar, otomatik DNS veya rotaları göz ardı edin, hiçbir zaman varsayılan rota olarak kullanmayın, rota metriği, gizlilik uzantıları, adres oluşturma). Seçimler ve sayılar `←/→` ile değişir; `Enter` bir değer yazar; listeler (adresler, DNS, rotalar, izin verilen IP'ler) kendi sayfalarını açar; burada `Enter` bir girişi düzenler, onu temizleyerek onu kaldırır ve **Giriş ekle** bir giriş ekler. Seçilen yöntem için geçerli olmayan alanlar devre dışı olarak gösterilir. Bir köprü, tahvil veya ekip, Ethernet **Bağlantı Noktalarını** bir alt sayfada seçer; WireGuard, **Eşlerini** bir alt sayfada düzenler ve **anahtar çifti oluşturabilir** (`wg` ile, `wireguard-tools`'dan, `argvus` paketinin bir bağımlılığı); özel anahtarın genel anahtarı onun altında gösterilir.

Hiçbir şey değişmediğinde veya bir alan geçersiz olduğunda devre dışı kalan **Kaydet**'e kadar NetworkManager'a hiçbir şey ulaşmaz ve ardından hangi alanı ve nedenini söyler. Yalnızca değişen ayarlar yazılır. Etkin bir bağlantı kaydedilirken önce soru sorulur ve ardından yeniden bağlanır, böylece yeni ayarlar hemen uygulanır (bu, ağın kısa süreliğine kesintiye uğramasına neden olabilir). Kaydedilmemiş değişiklikleri olan `Esc` bunları atmadan önce sorar. Tehlikeli bölgede **Bağlantıyı sil**, İptal'e odaklanarak onay ister; bir köprüyü, bağı veya takımı silmek aynı zamanda bağlantı noktalarını da siler.

Parolalar, özel anahtarlar ve önceden paylaşılan anahtarlar hiçbir zaman geri okunmaz: yeni bir değer yazılana kadar satırlarında *Değişmedi* ifadesi geçer ve yeni değer maskelenmiş olarak gösterilir. `nmcli connection edit`'nun standart girişi aracılığıyla yazılırlar, hiçbir zaman komut argümanları olarak yazılmazlar. Değişiklikler oturum kullanıcısı olarak gerçekleştirilir: `wheel`'nun yerel üyelerine NetworkManager'ın polkit kuralı tarafından izin verilir ve diğer kullanıcılardan oturumun polkit aracısı tarafından yönetici şifresi istenir.

### Güç ve oturum

Güç, Uyanık Tut düğmesi de dahil olmak üzere kurulumun kullanabileceği sistem güç kontrollerini ve politika sayfalarını sağlar. Etkinleştirildiğinde Uyanık Tut, ARGVUS boşta kalma ilkesinin otomatik ekran kilidi ve ekran gücü zamanlayıcılarını başlatmasını engeller. Oturum, ARGVUS oturum durumunu, bileşenlerini, otomatik başlatmayı, tanılamayı ve günlükleri gösterir. **Güç** üzerinde, listenin altındaki **Eylemler** çubuğu **Askıya Al**, **Hazırda Beklet** ve **Yenile** öğelerini tutar (`r` da yenilenir): `Tab` veya `←/→` ona gidin, `Enter` odaklanılan düğmeyi çalıştırır, `↑`/`↓` odağı listeye geri getirir ve `Esc` geri döner. Askıya Alma ve Hazırda Bekletme, önce onay ister. Bu sayfalar sistem izinleri gerektirebilir ve Kontrol Panelindeki hızlı işlemlerden ayırt edilmelidir.

### Sistem araçları

Sistem bölümü önyükleme bilgilerini, paketleri, hizmetleri, depolamayı, tanılamayı, kullanıcı/grup yönetimini ve sistem bilgilerini açığa çıkarabilir. Bunlar yönetimsel veya teşhis araçlarıdır; işleme bağlı olarak bir sayfa salt okunur olabilir veya yetkilendirme gerektirebilir.

**Hizmetler** dört sayfadan oluşur: sistem birimleri, kullanıcı birimleri, arızalı birimler ve günlük (Günlükler). Her birim listesi, f'nun döngü yaptığı bir `Filter:` satırını ve `f` ile açılan kenarlıklı bir `Search:` kutusunu gösterir. Her ünite, Aktif ve Etkin durumunu dolu veya boş noktalar halinde gösterir. Günlükler'de,  `/` `b` geçerli ve önceki önyükleme arasında geçiş yapar, `u` üniteyi döndürür ve `p` önceliği değiştirir; `Filter:` satırı kullanımdaki değerleri gösterir. `Enter` bir üniteyi veya günlük girişini açar.**Dev Services**, geliştirme amaçlı üç kaynağı salt okunur bir sayfada bildirir: geçerli kullanıcının sistemd hizmetleri, o anda dinlemekte olan TCP bağlantı noktaları (`ss`) ve yüklü herhangi bir kapsayıcı çalışma zamanından çalışan kapsayıcılar (Podman, Docker). Yalnızca bunlardan en az biri sistemde mevcut olduğunda Ana Izgarada görünür - systemd kullanıcı oturumu olmayan, Podman'ı olmayan ve Docker'ı olmayan bir makine girişi göstermez. `r` üç kaynağı bağımsız olarak yeniler; böylece yavaş veya arızalı bir kaynak (örneğin, yanıt vermeyi durduran bir kapsayıcı çalışma zamanı) diğerlerinin verilerini göstermesini engellemez.

**Paketler** kaynağa göre gruplandırılmıştır. **Resmi Depolar** ve bir AUR yardımcısı (`paru` veya `yay`) yüklendiğinde, **AUR (Arch User Repository)** her biri **Yükle**, **Kaldır** ve **Güncelleme**'yi açar; **Önbellek**, **Geçmiş**, **Sürüm düşürme** ve **Aynalar** bunu takip eder. Üç görev aynı ekranı paylaşır: Üstte sabitlenen **Arama** kutusu yazdığınız harfleri alır ve paketler onun altında **Sonuçlar** bloğunda (Güncellemede**Güncellemeler**) listelenir; burada boş bir liste nedenini belirtir (örneğin **Sonuç yok**, **Eşleşen öğe yok** veya **Bekleyen güncelleme yok**). **Yükle**, üstteki Arama çubuğunda Enter tuşuna bastığınızda depoları (`pacman -Ss`) veya AUR'yi arar; **Kaldır** kurulu paketleri listeler (AUR listesi, depoların dışında oluşturulan paketleri gösterir,  `Enter` `pacman -Qm` ve AUR menüsü gösterilirken Resmi listeden ayrılırlar); **Güncelleme** bekleyen güncellemeleri listeler (AUR için `pacman -Qu` veya `paru -Qua`/`yay -Qua`). Bir pakette, `Enter` ekranın görevini çalıştırır ve `→`, **Yükle** (veya **Yeniden Yükle**) ve **Tehlike bölgesinde** **Kaldır** ile paket sayfasını açar; `←` veya `Esc` geri dönün. **Resmi Depolar → Kaldır** **Orphans** ile başlar; burada `Space` paketleri (`[x]`) işaretler, `Enter` bir paket açar ve **İşaretli kaldır** bunları birlikte kaldırır. `Tab`, Güncelleme sonuçları listesinden listenin altındaki **Eylemler** çubuğuna geçer (**Tümünü yükselt** ve **Veritabanını yenile** veya AUR'da **Tümünü yükselt (AUR)**), `←`/`→` düğmeleri arasında hareket eder, `Enter` odaklanılanı çalıştırır ve `↑`/`↓` (veya yazarak) odağı listeye geri getirir; **Tümünü yükselt** seçeneği soluk görünür ve herhangi bir güncelleme listelenmediğinde atlanır. İçerik haritası ve genel arama aynı yolu izler; örneğin **Paketler > Resmi Depolar > Kaldır > Yetimler**. Önbellek temizleme işlemleri, Önbellek sayfasının Tehlike bölgesindedir ve **Aynalar → Aynaları yapılandır**, **Önizleme oluştur**'dan önce yansıtıcı seçeneklerini ayarlar. Her paket işlemi, çalışmadan önce planını veya AUR için PKGBUILD uyarısını gösterir ve onay ister (`y` onaylar, `n` veya `Esc` iptal eder) ve çalışırken çıktısı gösterilir. AUR işlemleri, yardımcıyı kullanıcınız olarak çalıştırır (birini yüklemek veya güncellemek için `-S <package>`, tümünü güncellemek için `-Sua`); AUR kaldırma işlemleri, resmi paketlerle aynı `pacman -Rns`'yu kullanır. Yüklü diğer paketler kaldırdığınız pakete (resmi veya AUR) bağlı olduğunda, onay bunları adlandırır ve iki yol sunar: **Yalnızca kaldır** seçilen paket (bağımlılık kontrollerini atlayan `pacman -Rdd`, böylece ihtiyaç duyan paketler çalışmayı durdurabilir) veya **bağımlılarla birlikte kaldır** (kaldırdığı paket sayısıyla `pacman -Rcns`). Odak noktası her paketi saklayan **İptal** ile başlar; `Enter` odaklanılan satırı çalıştırır ve bu seçimde `y` kısayolu yoktur.Üstteki **Arama** çubuğu paket sorgusunun sahibidir. `/`, metnini temizlemeden ona odaklanır; yazmak aynı zamanda odağı da oraya taşır. Ok tuşlarını, `Home`, `End`, `Backspace`, `Delete` ile düzenleyin veya yapıştırın. **Yükle**'de `Enter`, önceki sonuçlar mevcut olsa bile bir depo/AUR sorgusu gönderir ve ardından odağı sonuçlara taşır; resmi aramalar boş olmayan bir sorgu gerektirir ve AUR aramaları en az iki karakter gerektirir. Bir Yükleme sorgusunu düzenlemek, bekleyen isteği iptal eder ve eski sonuçlarını temizler; yeni bir istek `Enter`'yu bekliyor. **Kaldır**, **Güncelleme** ve **Orphans**'ta düzenleme, yüklenen listeyi başka bir depo sorgusu olmadan hemen filtreler. `↓` Arama'dan sonuçlara taşınır, ilk sonuçtaki `↑` Arama'ya döner ve Arama'daki `Esc` sorguyu korurken listeye döner. Sonuç olarak `Enter` o paketin eylemini gerçekleştirir. Güncelle eylem çubuğuna `Tab` tarafından ulaşılabilir durumda kalır ve `/` buradan Arama'ya döner.

**Paketler**'deki tüm işlemler entegre etkileşimli bir terminal açar: resmi paketler, AUR, önbellek temizleme, yansıtmalar ve depo/anahtar değişiklikleri. `Enter` yazıp tuşuna basarak programın istemlerini yanıtlayın; pacman bu terminalde `--noconfirm` olmadan çalışır. `Esc` görünümü gizler ve sürecin çalışır durumda kalmasını sağlar. `F8` aynı oturumu herhangi bir Paketler sayfasından yeniden açar; **Terminali göster** menülerinde de mevcuttur. Alt bilgi işlemin çalışıp çalışmadığını, tamamlandığını veya başarısız olduğunu gösterir. Başarılı bir komutun ardından, terminalin gizlendiği durumlar da dahil olmak üzere, son çıkışından sonra terminalin içinde `Done!` belirir. `Shift+PageUp` / `Shift+PageDown` geçmişi kaydırır, `Shift+End` canlı çıktıya döner ve `Ctrl+C` alt sürece bir kesme gönderir. Terminal odaklanmışken genel kısayollar yerine `q` ve `?` girdidir. Çıktı, ANSI renkleri ve imleç korunur ve yeniden boyutlandırma alt terminali günceller. Geçmiş, bir sonraki işleme veya uygulamadan çıkılana kadar 10.000 satıra kadar saklanır. Aynı anda yalnızca bir işlem yürütülür; terminalini gizlemek, uygulamanın çalışırken kapanmasına izin vermez. Serbest kabuk açılmaz.

Yükseltme önizlemesi, mevcut senkronizasyon veritabanlarını (`pacman -Su --print`) değiştirmeden veya kök gerektirmeden okur. Onaylandıktan sonra tam yükseltme, `pacman -Syyu`'yu ayrıcalıklı yardımcı `pkexec` aracılığıyla çalıştırır; senkronizasyon ek güncellemeleri ortaya çıkarabilir. Paket yükleme, kaldırma, veritabanı yenileme, önbellek temizleme, sürüm düşürme, depo/anahtar değişiklikleri ve ayna yazma işlemleri de bu ayrıcalıklı yardımcıyı kullanır. AUR yardımcıları, kullanıcınız olarak oluşturmaya devam eder.**Resmi Depolar → Depolar** resmi olmayan pacman depolarını ekler ve yönetir. **Ekle** bir taslaktır: yapılandırmanın **Kaynağı**'nı seçin (bir `.conf` dosyasının **URL'si**, çok satırlı bir düzenleyicide yazılan veya yapıştırılan **Metin** veya Arch Wiki düzeninde yazılmış **Ad**, **Sunucu** ve **SigLevel** içeren **Form**) ve **İmzalama anahtarı** altında bir **URL** veya **Metin**'den bir **Anahtar** (veya **Yok**). Seçimler `←`/`→` veya `Enter` ile hareket eder; metin alanları, `Enter`'nun tek bir satırı kaydettiği, `Ctrl+S`'nun birkaç satırı kaydettiği, yapıştırma işlerini yaptığı ve `Esc`'nun iptal ettiği bir düzenleyici açar. İndirmeler yalnızca `https://` adreslerini kabul eder. **İnceleyin ve ekleyin**, kullanıcınız olarak her şeyi indirip kontrol eder ve ardından onay ister; depoları, sunucuları, SigLevel'i ve anahtarın kullanıcı kimliğini ve parmak izini, deponun yayınladığıyla karşılaştırmak için dörtlü gruplar halinde listeler. Onaylama, anahtarı içe aktarır ve yerel olarak imzalar (`pacman-key --add`, `--lsign-key`), `/etc/pacman.d/<name>.conf` yazar (adını ilk bölümünden alır) ve `Include = /etc/pacman.d/<name>.conf`'yu `/etc/pacman.conf`'ya ekler (bunun yerine aynı dosya için mevcut bir açıklamalı satır etkinleştirilir). Daha sonra Kontrol Merkezi `pacman -Syyu`'yu sunar, böylece yeni veritabanları kısmi yükseltme olmadan senkronize edilir. **Ekle**'yi bir taslakla bırakmak, onu atmadan önce sorulmasını ister. **Liste**, `pacman.conf`'nun kendisinde yazılan veya `/etc/pacman.d/`'da bulunan, etkin veya devre dışı bırakılmış tüm resmi olmayan depoları gösterir; resmi bölümler hiçbir zaman listelenmez. Bir depo sayfası, dosyasını, SigLevel'i, sunucuları ve kayıtlı anahtarını, bir **Etkin** geçiş düğmesini ve **Tehlikeli bölge**'de **Kaldır**'ı gösterir. `Include` satırındaki yorumları devre dışı bırakma veya satır içi bölümün her satırına `#argvus-off ` ön eki ekleme; kaldırma işlemi satırı veya bölümü ve dahil edilen dosyayı siler. Dahil edilen aynı dosyanın depoları birlikte etkinleştirilir, devre dışı bırakılır ve kaldırılır. Kontrol Merkezi anahtarı içe aktardığında **Kaldır** ayrıca anahtarın pacman'in anahtarlığından (`pacman-key --delete`) kaldırılmasını da sağlar. `/etc/pacman.conf`'da yapılan her değişiklik, yanında bir yedek tutar (`pacman.conf.argvus-backup-…`) ve kurulu paketler hiçbir zaman kaldırılmaz.

### Kullanıcılar ve gruplar

Hesap sağlayıcı mevcut olduğunda **Kontrol Merkezi → Sistem Yönetimi → Kullanıcılar ve Gruplar → Kullanıcılar**'ı açın veya **kullanıcılar**'ı arayın. Arayüz normal ve sistem hesaplarını listeleyebilir, kullanıcı oluşturabilir, hesap meta verilerini düzenleyebilir, ek ve birincil grupları yönetebilir, bir parolayı değiştirebilir veya kilitleyebilir, kilidini açabilir, bir sonraki oturum açmada parola değişikliği gerektirebilir, bir avatar ayarlayabilir veya kaldırabilir ve otomatik oturum açmayı etkinleştirebilir.

Kullanıcı oluşturma işleminde kullanıcı adı, tam ad, kabuk, ek gruplar ve isteğe bağlı şifre onayı istenir. Boş bir parola, parolası kilitli olan hesabı oluşturur; sessizce kullanılabilir bir şifre oluşturmaz. Bir kullanıcı sayfası, hesap alanlarını (tam ad, kabuk, birincil ve ek gruplar, ardından **Değişiklikleri kaydet**), **Otomatik giriş** geçişini, **Şifre** işlemlerini (değiştirme, kilitleme, kilidi açma, bir sonraki girişte değişiklik gerektirme) ve **Avatar** işlemlerini gruplandırır. Kullanıcı adı, UID / GID ve ana dizin yalnızca bilgi amaçlı gösterilir. Sondaki **Tehlikeli bölge**, ana dizini korurken hesabı silmek veya ana dizini de silmek için ayrı yıkıcı eylemleri içerir. İkincisini seçmeden önce onay metnini dikkatlice okuyun. Hesap işlemleri ayrıcalıklıdır ve sistem yetkilendirme istemini açabilir. Avatarlar, hesabın standart yüz görüntüsü entegrasyonu aracılığıyla karşılayan kişi tarafından kullanılabilir.**Otomatik oturum açma** seçilen hesabın oturumunu bir sonraki önyüklemede oturum açma ekranını göstermeden başlatır; GNOME, LightDM ve SDDM'nin bu özellik için kullandığı kuralın aynısı: hiçbir yerde parola saklanmaz. Bir hesap için etkinleştirildiğinde, önceden yapılandırılmış herhangi bir hesap için devre dışı bırakılır (aynı anda yalnızca bir hesap otomatik olarak oturum açabilir). Bir sonraki önyüklemede bir kez uygulanır; Manuel oturum kapatmanın ardından, oturumun geri kalanında normal oturum açma ekranı geri döner. Geçiş, bir onay istemiyle hemen uygulanır ve yalnızca yöneticiler tarafından kullanılabilir.

Grupların, üye yönetimi ve grup silme dahil olmak üzere kendi liste ve düzenleme sayfaları vardır. Kontrol Merkezi, sistemin hesap politikasının yerini almaz: korunan sistem hesapları ve yetkilendirme gerektiren işlemler, işletim sisteminin kısıtlamalarına tabi olmaya devam eder.

### Kontrol Merkezi ayarları

**Ayarlar** evin son girişidir. **Dil** sayfası, locale.conf aracılığıyla kullanıcı dilini ayarlar ve **Stil** sayfası dekoratif simgeleri ve Kontrol Merkezi şeffaflığını kontrol eder. Stil bulanıklığı barındırmaz: bulanıklık, Hyprland > Bulanıklaştırma altındaki genel bir Hyprland ayarıdır. Stil sayfası, ayarlar uygulamasının sunumunu değiştirir; ARGVUS hizmetlerini devre dışı bırakan veya ayar sağlayıcılarını kaldıran bir anahtar değildir. Görsel değişiklik mevcut uygulamaya uygulanırken simgelerin kaydedilmesi yetkilendirme gerektirebilir. Style'daki Esc, Ayarlar sayfasına geri döner ve oradaki Esc ana sayfaya döner.

### ARGVUS Hakkında

**ARGVUS Hakkında** **Araçlar ve Tanılama → ARGVUS Hakkında** bölümündedir (`about`, `hardware` veya `firmware`'yu arayın veya `argvus control-center about`'yu çalıştırın). `Tab`/`Shift+Tab`, `←`/`→` veya `h`/`l` ile değiştirilen beş sekmesi vardır: **Sistem ve Donanım**, **Hakkında**, **Bağış**, **Krediler** ve **Telif Hakkı**. Başlık tam yolu gösterir; örneğin `Control Center > Tools & Diagnostics > About ARGVUS > System & Hardware`.

**Sistem ve Donanım** salt okunurdur ve bu makinenin gerçeklerini, değerler tek bir sütunda hizalanacak şekilde üç bölümde gruplandırır:

| Bölüm | Alanlar |
| --- | --- |
| Önyükleme | Ürün Yazılımı (satıcı, sürüm ve tarih), Tür (UEFI veya Eski (BIOS)), Güvenli Önyükleme (yalnızca UEFI), Sanallaştırma, Önyükleyici |
| Sistem | ARGVUS, Ana Bilgisayar Adı, İşletim sistemi, Dağıtıcı, Çekirdek, Mimari, Pencere yöneticisi (Hyprland), Besteci (Wayland), GPU sürücüsü, GTK, Tema (Koyu veya Açık), Ekranlar (monitör başına bir satır), Ses (PipeWire), Kabuk (Quickshell), Oturum Açma (greetd), Init (systemd), Paketler, Çalışma Süresi, Yerel Ayar |
| Donanım | Pil (pil başına bir hat), CPU (maksimum frekansıyla), Konular, Çekirdekler, GPU, Bellek (kullanılan / toplam), Takas, Depolama, Model (yalnızca anakarttan farklı olduğunda), Anakart |

Her değer, çalışan sistemden ayrıcalıklar olmadan okunur ve sayfa herhangi bir makinede aynı şekilde çalışır. Okunamayan bir değer `N/A` olarak gösterilir ve geçerli olmayan bir satır dışarıda bırakılır:

- **Ürün yazılımı**, **Model** ve **Anakart** DMI'dan gelir (`/sys/class/dmi/id`); `To be filled by O.E.M.` gibi yer tutucu değerleri yoksayılır. **Tür** ve **Güvenli Önyükleme** `/sys/firmware/efi`'dan ve **Sanallaştırma** `systemd-detect-virt`'dan gelir.
- **Önyükleyici**, Systemd-boot, Limine ve Önyükleme Yükleyici Arayüzünü takip eden diğer önyükleyicilerin yayınladığı `LoaderInfo` EFI değişkeninden okunur. GRUB bunu yayınlamaz, dolayısıyla `/boot/grub` (veya `/efi/grub`) tarafından tanınır ve `grub` paketi sürümüyle birlikte gösterilir.
- ARGVUS, Wayland, Mesa, PipeWire, Quickshell, greetd, systemd ve GRUB sürümleri bir `pacman -Q` sorgusundan gelir ve dönem ve paket sürümü olmadan gösterilir (`1:25.2.3-1`, `25.2.3` olur). Hyprland sürümü `hyprctl version`'dan veya hiçbir oturum yanıt vermediğinde paketten gelir.
- **GPU sürücüsü**, `/sys/class/drm`'daki her DRM kartına bağlı sürücüyü adlandırır. Tescilli bir sürücü, modül sürümünü gösterir (örneğin `nvidia 580.82.07`); çekirdek içi bir sürücü Mesa sürümünü gösterir (örneğin `amdgpu (Mesa 25.2.3)`).
- **Ekranlar** `hyprctl monitors -j`'daki aktif monitörleri listeler: konektör, çözünürlük, yenileme hızı ve ölçek (ölçek 1'de bırakılmıştır).
- **Paketler** `pacman -Qq`, `-Qqe` ve `-Qqm`'dan açıkça yüklenmiş olanlar ve yabancı olanlar (AUR veya yerel olarak oluşturulmuş) olmak üzere tüm yüklü paketleri sayar.
- **Çalışma süresi** `/proc/uptime`'dan gelir ve sayfanın açıldığı andaki değerdir. **Yerel ayar** /etc/locale.conf'daki `LANG`'dan gelir.
- **Pil**  `/etc/locale.conf` `/sys/class/power_supply`'dan türe göre seçilen bilgisayarın pillerini listeler; dolayısıyla kablosuz fare gibi çevre birimleri sayılmaz. Her satır şarjı, durumu, sağlığı (tasarım kapasitesiyle karşılaştırıldığında tam kapasite) ve ürün yazılımı bunları raporladığında şarj döngülerini gösterir. Pilsiz bir makine **Algılanmadı** mesajını gösterir.
- **CPU**, hibrit işlemcilerde nominal maksimum olan tüm çekirdekler (cpufreq) arasında en yüksek maksimum frekansı ekler; çoğu sanal makinede olduğu gibi cpufreq'in bulunmadığı yerlerde bırakılır. **Konular** ve **Çekirdekler** `/proc/cpuinfo`'dan gelir.
- **Bellek** ve **Takas** `/proc/meminfo`'dan gelir (zram dahil); takasın olmadığı bir sistemde **Devre Dışı** gösterilir.
- **Depolama**, `/` kapsamındaki diski tanımlar: modeli, türü (NVMe, SSD, HDD veya eMMC/SD; sanal diskler tür göstermez), dosya sistemi ve kullanılan ve toplam alan. Disk, fiziksel aygıta kadar LUKS ve LVM bölümleri aracılığıyla bulunur.

Sayfa ilk çizildiğinde değerler arka planda okunur, dolayısıyla ne Kontrol Merkezi başlatılır ne de sayfa açılışı onları beklemez; Bu arada sekmede **Sistem bilgileri yükleniyor...** gösterilir. Raporun tamamını panoya düz metin olarak kopyalamak için (wl-copy ile), örneğin bir hata raporuna yapıştırmak için bu sekmede `c` tuşuna basın.

## Arama ve isteğe bağlı yetenekler

Ana ekranda aranabilir bir rota kaydı bulunur. Arama terimleri görünüm, temalar, duvar kağıtları, yazı tipleri, klavye kısayolları, fare, dokunmatik yüzey, ekranlar, ağ ve oturum gibi kullanıcıya yönelik kavramları içerir. İsteğe bağlı sağlayıcılar yalnızca özellikleri derlendiğinde ve çalışma zamanı yetenekleri algılandığında dahil edilir, böylece görünür menü kasıtlı olarak yeteneklere duyarlı olur.

Ana Sayfa araması etkinken sorguya  `wl-copy` `j` ve `k` dahil yazdırılabilir anahtarlar girilir; Eşleşen rotalarda ilerlemek için `↑` ve `↓` ok tuşlarını kullanın. Seçilen rotayı açmak için `Enter`'ya, sorguyu düzenlemek için `Backspace`'ya ve sorguyu temizlemek veya arama modundan çıkmak için `Esc`'ya basın.

## Görünüm iş akışı

Entegre görsel kontrolleri görmek için **Görünüm**'ü açın. Grupları şunlardır:

- **Masaüstü** — **Tema** (bir paket aileyi ve onun Yapışkan veya Aralıklı modunu seçin veya kendi temalarınızı dışa aktarın, içe aktarın, uygulayın ve silin, bkz. [Temaları içe ve dışa aktarma](/docs/components/argvus-themes/import-and-export/)), **Görünüm Modu** (Yapışkan veya Aralıklı, temadan bağımsız), **Vurgu Rengi** (birleştirilmiş vurgu veya geçerli altı basamaklı RGB rengi), **Duvar Kağıdı** (bir pakette) duvar kağıdı veya özel bir dosya) **Simgeler** (kurulu bir simge teması: listede `/` ile arama yapın, Space veya Enter ile birini seçin, ardından **Uygula**'yı seçin; Uygula'dan önce hiçbir şey değişmez) ve **Yazı Tipleri** (yazı tipi hedefleri ve yazı tipi oluşturma ayarları, Yazı Tipleri sayfasında açılır).
- **Uygulama Stili** — **Terminal** ve **Başlatıcı** şeffaflığı.Görünüm kartı şu ayarları kullanan yüzeyleri içerir: **Widget Telemetrisi** (telemetri yüzeyini etkinleştirin, mevcut blokları seçin ve odaklanmış bir blokta `Shift+Up`/`Shift+Down` ile yeniden sıralayın) ve **Denetim Masası** (panel kartlarını Odaklanmış bir kart üzerinde `Shift+Up`/`Shift+Down`, gerçek panelin sürükle ve bırak kullanımlarıyla aynı ilkel yöntemle hemen uygulanır).

Bu eylemler mantıksal ARGVUS durumunu günceller ve etkilenen çalışma zamanı yapılandırmasını uygular. Bkz. [Görünüm](/docs/user-guide/appearance/), [Temalar](/docs/user-guide/appearance/) ve [Pencereler ve düzen](/docs/argvus-hyprland/windows-and-layout/).

## Klavye kullanımıAyarlar uygulaması klavyeyle gezinmeyi destekler. Her sayfa tek bir listedir: taşımak için ok tuşlarını veya `j`/`k` tuşlarını kullanın, seçilen satırı açmak veya çalıştırmak için `Enter`, geçiş yapmak için `Space`, geri dönmek için `Esc`, Listeleri aramak için `/` ve yardım için `?`; altbilgi, seçilen satıra uygulanan tuşları gösterir. Eylemler düğmeler değil, listenin satırlarıdır ve `Tab`, listenin altında **Eylemler** çubuğu bulunan sayfalar (Varsayılan Uygulamalar, Saat Dilimi, Klavye Düzeni, Sistem Yerel Ayarları ve SSH Anahtarları ve Kısayol Oluştur/Düzenle) dışında bunlara taşınmaz. Yalnızca bilgileri gösteren satırlar atlanır. Bir metin alanı, arama veya liste filtresi etkinken, yardımdan çıkmak veya yardım açmak yerine `q` ve `?` metin olarak yazılır. **Varsayılan Uygulamalar**'da **Varsayılanları Geri Yükle**, listenin altındaki **Eylemler** çubuğundaki düğmedir: `Tab` veya `←/→` buna odaklanın ve `Enter` sıfırlamayı ister. Diğer sayfalardaki **Varsayılanları sıfırla** ve **Tüm kısayolları geri yükle**, sayfalarının sonundaki **Tehlikeli bölgede** bulunan satırlardır; `r` aynı sıfırlamayı ister ve kısayollar listesinde `r`, onayın ardından seçilen kısayolu geri yükler. **Dil ve Bölge → Klavye Düzeni** üzerinde, `/` sayfanın üst kısmındaki **Arama** kutusuna odaklanır ve `Esc` onu temizler; `●`, kullanımdaki düzeni işaretler. Düzenleri değiştiren görev çubuğunu ve klavye kısayolunu takip eder ve sayfa, yapılandırmaya hiçbir şey yazılmadan yaklaşık bir saniye içinde güncellenir. `Enter` varsayılan düzeni ayarlar; Hyprland'ın başladığı ilk düzen ve ayrıntılarında `Default` etiketli düzen; `Space` bir düzeni ekler veya kaldırır (`[x]`). Bu tuşlar yalnızca taslağı değiştirir ve listenin altındaki **Eylemler** çubuğundaki **Uygula** düğmesi, düzenleri ve varsayılanı tek adımda Hyprland'e yazar. Listeden, `Tab` veya `←/→`, **Uygula** etkinken odağı o çubuğa taşır (daha sonra altbilgide `←/→ Actions` gösterilir); `←/→` düğmeleri arasında geçiş yapar ve `↑/↓` listeye geri döner. Sayfadan uygulanmamış değişikliklerle çıkmak için öncelikle onay istenir. **Dil ve Bölge → Saat Dilimi** üzerinde düzen aynıdır: `/` üstteki **Arama** kutusuna odaklanır, `Enter` taslakta bir bölge seçer (`●` ile işaretlenmiştir) ve listenin altındaki **Eylemler** çubuğundaki **Uygula** düğmesi bunu kullanıcının `environment.d/20-timezone.conf`; `Tab`, `←/→` ve `↑/↓` Klavye Düzeninde olduğu gibi çalışır. Uygulanmamış bir bölgeyle ayrılmak için öncelikle onay istenir. **Sistem Yerel Ayarları** üzerinde düzen, Saat Dilimi ile aynıdır: `/` üstteki **Ara** kutusuna odaklanır, `Enter` veya `Space` bir taslakta bir yerel ayarı (`[x]`) işaretler veya işaretini kaldırır ve taslaktaki **Uygula** düğmesi Listenin altındaki **Eylemler** çubuğu, seçimi `/etc/locale.gen`'ya yazar ve `locale-gen`'yu çalıştırır. Bu, sistem yerel ayarlarını yeniden oluşturduğundan **Uygula** önce onay ister; `Tab`, `←/→` ve `↑/↓` diğer ikisinde olduğu gibi çalışır. **Masaüstü ve Giriş → Klavye Kısayolları → Tüm Kısayollar** üzerinde arama kutusu da üstte yer alır ve `/` buna odaklanır, ancak sayfada **Eylemler** çubuğu yoktur: listesinde kısayol bölümleri ve **Tehlikeli bölge** bulunur ve alt bilgi `r Restore default`'yu **Kısayolu düzenle**'de değil yalnızca listenin kendisinde listeler.Değişiklikleri kaydetmeden önce toplayan sayfalar (bir kullanıcı, grup, hesap oluşturma, şifre değişikliği ve güvenlik duvarı yapılandırması), bu bloğu, bir şey değişene kadar soluk kalan **Kaydet** veya **Uygula** satırıyla sonlandırır. Geri dönmek, kaydedilmemiş değişikliklerin silinmesine neden olacaksa önce Kontrol Merkezi bunu sorar. Her onay, **İptal**'e odaklı iki satır olarak **Onayla** ve **İptal**'i gösterir: `Enter` odaklanılan satırı çalıştırır, `y` onaylar ve `n` veya `Esc` iptal eder.

**Görünüm**'de her sayfa tek bir listedir: alt bilgi yalnızca seçilen satıra uygulanan tuşları gösterir ve `Space` da satırı açar veya seçer. Değişiklikleri uygulamadan önce toplayan sayfalar (Görev Çubuğu, Widget Telemetrisi, Denetim Masası ve efekt kaydırıcıları), bir şey değişene kadar soluk kalan **Uygula** satırıyla biter. **Uygula** seçildiğinde yükleme ekranı yalnızca değişiklik bir yüzeyi yeniden oluşturduğunda görüntülenir: görev çubuğu yüksekliği, yardımcı program grubu, simge veya tarih ve saat biçimi değişikliği görev çubuğunu yeniden başlatır; Widget Telemetrisini açmak veya kapatmak ya da bloklarını veya sırasını değiştirmek widget'ı yeniden başlatır; Kontrol Panelini açmak veya kapatmak veya açıkken kartlarını değiştirmek paneli yeniden başlatır. Daha sonra ekran, sayfadaki son değişiklik uygulanana kadar kalır, böylece önünde hiçbir değişiklik görünmez ve yüzey bir kez yeniden başlatılır. Kenarlıklar, şeffaflık, bulanıklık, animasyonlar, boşluklar, yazı tipleri, duvar kağıdı ve diğer ayarlar, yükleme ekranı olmadan yerinde uygulanır. Yüzde satırlarında, `←`/`→` (veya `+`/`-`) değeri 5'lik adımlarla değiştirir ve `Enter` bunu yazmanıza olanak tanır; geri dönmek için `Esc` kullanın. Geriye dönmek, uygulanmayan değişikliklerin silinmesine neden olacaksa önce Kontrol Merkezi bunu sorar. Özel bir temanın (`d`) silinmesi veya içe aktarılan bir temanın değiştirilmesi onay gerektirir; odak **İptal** ile başlar, `y` onaylanır ve `n` veya `Esc` iptal edilir.

## Kalıcılık ve sıfırlama

Ayarlar sayfaları, değişiklikleri kendi sağlayıcıları aracılığıyla uygular ve desteklenen kullanıcı durumunu kaydeder. Görünüm değişiklikleri standart yapılandırmada depolanır ve `argvus-config`, `~/.config/argvus/data/generated/` altında türetilmiş tüketici dosyalarını yazan tek bileşendir; oluşturulan dosyalar kalıcı düzenleme yapılabilecek yer değildir. Giriş ve tuş bağlama sayfalarının kendi kalıcı durumları ve sıfırlama eylemleri vardır. Diğer sistem değişiklikleri izinleri veya hizmetin yeniden yüklenmesini gerektirebilir.

Her ARGVUS ayarı için genel sıfırlama yoktur. Bir değişikliği, ona ait olan sayfadan geri yükleyin veya ilgili özelliğin belgelenmiş kurtarma prosedürünü kullanın.

Sıfırlama desteği kasıtlı olarak alan adı başına verilmektedir:

- **Görünüm**, vurgu renginin etkin tema varsayılanına sıfırlanmasını sağlar. Tek bir genel görünüm sıfırlaması sağlamaz; diğer görünüm değerlerini kendi kontrollerinden geri yükleyin.
- **Yazı Tipleri** tüm yazı tipi ayarlarını, bir hedefi veya tek bir ayarı geri yükleyebilir.
- **Varsayılan uygulamalar** tüm varsayılanları, bir kategoriyi veya tek bir seçiciyi geri yükleyebilir.
- **Klavye kısayolları** bir bağlamayı veya tüm bağlamaları geri yükleyebilir ve ardından oluşturulan oturum bağlamalarını yeniden yükleyebilir.
- **Pencere kuralları** **Masaüstü ve Giriş → Hyprland → Windows → Kurallar**'dadır (`window rules`'yu arayın veya `argvus control-center window-rules`'yu çalıştırın). Her kuralda bir **Çalışma Alanı** satırı (`Enter` veya `←/→` 1'den 10'a kadar adımlar) ve virgülle ayrılmış normal ifadeler içeren bir metin alanı açan bir **Pencere sınıfları** satırı bulunur. **Kural ekle**, çalışma alanı 1 ile ve sınıfsız `rule-N` oluşturur. Her kuralın **Kural kaldır** seçeneği **Tehlike bölgesinde** bulunur ve onay ister. Her değişiklik argvus config'da `hyprland.window_rules`'ya yazılır ve yapılandırmayı anında yeniden yükler; geçersiz sınıf kalıpları reddedilir ve hata olarak gösterilir.
- **Projeler**, **Araçlar ve Tanılama → Projeler**'dedir ( `argvus config` `projects`'yu arayın veya `argvus control-center projects`'yu çalıştırın). **Proje klasörü ekle** ve **Kök klasör ekle** sayfanın en üstünde bulunur ve yolu içeren bir metin alanı açar; başlatıcı bunun bir dizin olup olmadığını kontrol eder ve aksi takdirde, klasörün etkin proje sayısını 9'un üzerine çıkarması da dahil olmak üzere hatayı gösterir (`SUPER + CTRL + 1..9` daha fazlasını adresleyemez). Kök (alt klasörleri projelerdir) veya tek bir proje olarak işaretlenen yapılandırılmış her klasör, formun altında onay isteyen bir **Kaldır** satırıyla birlikte kendi bloğunu alır. Sayfa `argvus-projects`'yu çağırır, dolayısıyla değişiklikler başlatıcı tarafından yazılır ve `SUPER + O` veya `SUPER + CTRL + 1..9`'nun bir sonraki çalıştırılışında uygulanır. Eğer`argvus-projects` kurulu değil, sayfada öyle yazıyor.
- **Diğer sistem sayfaları** sıfırlama, uygulama, silme veya geri yükleme işlemlerini yalnızca temel sağlayıcı desteklediğinde gösterir.Bir kullanıcı tercihinin kaldırılması sağlayıcının paketlenmiş/varsayılan durumuna geri dönmesine neden olabilir, ancak dosyaların elle silinmesi genel bir kurtarma prosedürü değildir. Sayfanın sıfırlama eylemini veya belgelenmiş özelliğe özel komutu tercih edin.

## CLI giriş noktalarıYardımcı `argvus-control-center`, `argvus control-center` üzerinden çalıştırılır. Özellikle odaklanmış alanları doğrudan açabilir, örneğin:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```

Yüklü sürümde mevcut olan rotalar için `argvus control-center --help` komutunu çalıştırın. Komut satırı rotası, ayrı bir konfigürasyon sistemi değil, aynı ayarlar uygulamasına giden bir kısayoldur.Faydalı doğrudan rotalar arasında `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (`status`, `connections`, `activate` sayfalarıyla, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` ve `proxy`), `audio`, `bluetooth`, `boot`, `packages` (`official`, `search`, `installed` sayfalarıyla, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` ve `mirrors`), `storage`, `system-doctor` (`diagnostics` ve `diagnostic` takma ad olarak kalır), `about` (`system`, `about`, `donate` sekmeleriyle, `credits` ve `copyright`), `power`, `session`, `displays` ve `appearance`. Görünüm ayrıca `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces` gibi odaklanmış rotaları da kabul eder. `taskbar` ve `widget-telemetry`; ekran `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` ve vrr'yu kabul eder ve `hdr`. Kullanılabilirlik hâlâ yüklü yapıya bağlı olduğundan, komut dosyası yazarken yüklü `--help` çıkışını kullanın.

## İlgili

- [İlk yapılandırma](/docs/getting-started/)
- Kontrol Paneli
- [Görünüm](/docs/user-guide/appearance/)
- [Klavye kısayolları](/docs/argvus-hyprland/keyboard-shortcuts/)- [Fare ve dokunmatik yüzey](/docs/argvus-hyprland/input/)

### Sistem Doktoru

**Araçlar ve Tanılamalar → Sistem Doktoru**'nu açın, arayın veya `argvus control-center system-doctor`'yu çalıştırın. Önceki `diagnostics` ve `diagnostic` rotaları ve kategori seçicileri kullanılabilir durumda kalır. Yenilemek için `r` tuşuna, bir çeki incelemek için Enter tuşuna, ilgili sayfayı açmak için ayrıntılı olarak `o` tuşuna ve geri dönmek için Esc tuşuna basın. Koleksiyon arka planda çalışır ve salt okunurdur.Dosya sistemi kullanımı ve SMART sağlığı için sistemd arızalı birim sayımlarını, grafik araçlarını ve Depolama anlık görüntüsünü kullanan kontroller. Bir problem kanıt gerektirir; Kullanılamayan araçlar veya erişilemeyen SMART verileri, kullanılamaz olarak gösterilir ve başarısız veya zaman aşımına uğramış bir araştırma bir hatadır. Toplanmış delillerin bulunmadığı kategoriler, kontrolün başarılı olduğu anlamına gelmez. Paket güncellemeleri yerel pacman veritabanını kullanır ve depoları senkronize etmez; bir kilit dosyası eski olabilir ve bir işlemin çalıştığını kanıtlamaz. SMART izinler gerektirebilir ve yükseltilmiş erişim talep edilmez. Özet, son tamamlanan koleksiyonu Unix zamanı olarak kaydeder.

Storage ve System Doctor, bellek içi anlık görüntüyü 10 saniyeye kadar paylaşır. `r` tuşuna basmak açıkça yeni veriler toplar; eşzamanlı istekler yeni tamamlanmış bir koleksiyonu yeniden kullanır. Tanılama ve Depolama alt işlemi çıkışı, akış başına 1 MiB ile sınırlıdır ve büyük boyutlu çıktı, başarısız bir koleksiyon olarak değerlendirilir.

## Gece Lambası

**Masaüstü ve Giriş > Ekranlar > Gece Işığı**'nı açın veya `argvus control-center displays night-light`'yu çalıştırın. `argvus config display night-light` aracılığıyla sıcaklığı, manuel aktivasyonu ve sabit yerel programları yapılandırın. Komutlar, durum ve çalışma zamanı gereksinimleri için bkz. [Gece Işığı](/docs/user-guide/night-light/).

## Anlık Görüntü

**Kontrol Merkezi > Sistem Yönetimi > Anlık Görüntü**'yü açın ve şunu arayın:
`Snapper` veya `argvus control-center snapshot`'yu çalıştırın. `snapshot` Kargo özelliği
`full`'ya (varsayılan) dahildir ve `storage`'dan bağımsızdır.
Özellik hâlâ geliştirilme aşamasında olduğundan sayfa **Deneysel** olarak işaretlendi
ve eksik olabilir.

Bu sayfa yalnızca `SUBVOLUME` olan mevcut bir Snapper yapılandırmasını yönetir.
`/` ve `FSTYPE`'su `btrfs`'dur. `/` içeren montajı kontrol eder.
`findmnt`; `root` adlı başka bir yeri işaret eden yapılandırma kabul edilmez.
Çoklu eşleşen konfigürasyonlar bir seçim gerektirir. `snapper` Arch paketi
isteğe bağlıdır ve manuel olarak yüklenmesi gerekir. Snapper, `--jsonout`'yu desteklemeli ve
açık aktif/varsayılan sütunlar. Hiçbir konfigürasyon otomatik olarak oluşturulmaz.

Btrfs olmayan bir kökte sayfa **Bu dosya sistemi Btrfs değildir** ifadesini görüntüler ve aşağıdakileri sunar:
anlık görüntü işlemi yok. Algılama hatası, eksik Snapper, eksik kök
yapılandırma, izin reddi ve arka uç hataları ayrı durumlardır.
Normal kullanıcı sorguları reddedilirse Polkit istemek için **a — Yetkilendir**'e basın
erişim. Kısıtlı `system-settings` yardımcısında yürütülen yönetim eylemleri,
yüksek bir TUI'de değil; kimlik doğrulaması daha sonraki sorgularda tekrar istenebilir.
Kimlik doğrulama için mevcut bir grafiksel Polkit aracısı gereklidir.

Tuşlar: **↑/↓** seç, **ayrıntıları/yapılandırmayı gir**, **c** oluştur, **d** sil,
**p** saklama politikaları, **r** yenileme, **Esc** geri, **?** yardım. Saklama konusunda
görüntüleyin, **↑/↓** bir politika seçin, **e** düzenleyin, **c** döngü temizleme algoritmaları ve
**Girin** açık onayı. Yaratılış isteğe bağlı bir açıklama açar
(en fazla 256 yazdırılabilir karakter); boş bir açıklama “ARGVUS kılavuzu”nu kullanır
anlık görüntü”. Hem oluşturma hem de silme, başlangıçta odaklanılan onay gerektirir.
İptal et. Silme işlemi seçilen bir anlık görüntüye uygulanır ve anlık görüntü 0'ı reddeder;
şu anda etkin olan anlık görüntü ve varsayılan anlık görüntü. Yardımcı bunları yeniden kontrol eder
Snapper'ı çağırmadan önceki koşullar. Listeler işlemlerden sonra yenilenir; kesintiye uğramış
işlem hiçbir zaman otomatik olarak yeniden denenmez. Önce yenilenen listeyi inceleyin.

Anlık görüntüleri karşılaştırmak için tabanı seçin ve **b** tuşuna basın, ardından farklı bir taban seçin
hedefi seçin ve **v** tuşuna basın. Sayfada sıralanan kimlikler, oluşturulan sayımlar,
kaldırılan ve değiştirilen yollar ve Snapper'ın altı durum göstergesi. Bir yol seçin
ve içerik farkını istemek için **Enter** tuşuna basın; **Esc** değiştirilen yollara geri döner
ve ardından anlık görüntü listesi. Göstergeler oluşturma/kaldırma veya içerik/tür kodlarkonum 1, izinler konum 2, sahip konum 3, grup konum 4, genişletilmiş özellikler konum
5 ve ACL 6'da. Nokta, o konumda değişiklik olmadığı anlamına gelir. Durum yolları listeler;
diff metin içeriğini gösterir. İkili/UTF-8 olmayan fark çıktısı oluşturulmaz. Fark
256 KiB'in üzerindeki çıktı kesik olarak raporlanır ve görüntülenmez. Belirsiz veya
hatalı biçimlendirilmiş durum satırları reddedilir. Snapper'ın çizgi formatı yolları temsil edemez
net bir şekilde satır sonları ile; bu tür adlar desteklenmez. İzin
hatalar **a — Yetkilendir** yoluyla yeniden denenebilir; kaldırılmış bir anlık görüntü veya değiştirilmiş
kök yapılandırması listenin yenilenmesini gerektirir. Arka uç testleri sahte kullanıyor
`ProcessRunner` ve Snapper, Btrfs veya root gerektirmez.

Karşılaştırmadan bir yol seçin ve dosyayı geri yüklemeyi gözden geçirmek için **r** tuşuna basın
hedef anlık görüntüsünden. Yardımcı, her iki anlık görüntünün de hala geçerli olduğunu doğrular.
kök konfigürasyona aittir, yol hala karşılaştırmadadır ve
kaynak, Snapper'ın standart `/.snapshots/<id>/snapshot` görünümündeki normal bir dosyadır.
Sınırlı Snapper farkı metinsel olduğunda gösterilir; ikili veya büyük boyutlu farklar
tüm içerikleri kullanıcı arayüzüne yüklenmeden tanımlanır. **Enter** bir pencere açar
kaynağı ve hedefi belirten ayrı bir onay; **Esc** olmadan iptal edilir
dosyayı değiştiriyorum. Onay, önce `single` güvenlik anlık görüntüsünü oluşturur, ardından
Snapper `undochange`'yu yalnızca bu yol için kullanır ve yol ile eşleşip eşleşmediğini kontrol eder.
anlık görüntü seçildi. Güvenlik anlık görüntüsü kimliği raporlanır ve anlık görüntü tutulur.
Restorasyon, ARGVUS paket yükseltmeleri tarafından kullanılan kilidi paylaşır, dolayısıyla çakışamaz
resmi bir Kontrol Merkezi pacman işlemi.

İlk sürüm, yalnızca root'a ait üst öğe altındaki root'a ait normal dosyaları kabul eder
Grup veya başkaları tarafından yazılamayan dizinler. Dizinleri reddediyor,
sembolik bağlantılar, sabit bağlantılar, özel dosyalar, bağlama noktaları/alt hacimler, Snapper'ın dahili
dizin ve kullanıcı tarafından yazılabilen konumlar. Seçilen anlık görüntüde eksik dosyalar
asla silinmez. Snapper'ın kökü `undochange` uygulamayı garanti etmez veya
veritabanı tutarlılığı ve atomik değildir; güvenlik anlık görüntüsü bir kurtarma noktasıdır,
bir garanti değil. Snapper yalnızca yapısı tarafından desteklenen meta verileri korur. Ayrı
yapılandırılan kökün dışındaki alt birimler ve dosyalar işlemin dışında kalır.

Manuel anlık görüntüler `single`'dur, Snapper'ın varsayılan ayarıyla salt okunurdur ve temizleme yapılmaz
Bu sayfa tarafından atanan algoritma. Mevcut Snapper zamanlayıcıları ve harici işlemler
uygulamaya devam edin. Anlık görüntüler tam sistem görüntüleri değildir: ayrı alt birimler,
Ayrı bir `/home` dahil değildir. Bu sürüm **hayır sağlar
tam sistem geri alma, Snapper yapılandırmasının kurulumu veya oluşturulması**. Basın
Bilinen saklama ayarlarını incelemek için desteklenen bir kök yapılandırmasında **p**.
Eksik değerler, kurulu sistemden okunan değerler olarak değil, ARGVUS geri dönüşleri olarak etiketlenir.
Snapper oluşturma şablonu; bilinmeyen anahtarlar yalnızca tanı amaçlıdır. Desteklenen bir değeri düzenlemek için **e** tuşuna basın. Onay, eski ve
yeni değerler; kaydettikten sonra sayfa yapılandırmayı tekrar okur ve doğrular.
Temizleme algoritmalarını değiştirmek için **c** tuşuna basın ve bir tanesini gözden geçirip onaylamak için **Enter** tuşuna basın.
Temizleme, Snapper'ın resmi algoritmasını kullanır; kesin bir önizleme yoktur ve
rapor edilen kaldırılan sayım, yürütme sonrasında listelerin karşılaştırılmasıyla ölçülür. Arch Snapper 0.13.2 şunları sağlar
`snapper-timeline.timer` ve `snapper-cleanup.timer`; Zaman çizelgesi oluşturma gerektirir
zaman çizelgesi zamanlayıcısı ve yapılandırılmış temizleme ilkeleri, temizleme olmadan çalışmaz
zamanlama. Sayfa yalnızca ünite durumunu okur ve zamanlayıcıları etkinleştirmez. Alan sınırları yapılandırılmış Btrf'ler gerektirir
qgroups ve burada gösterilmiyor.

### Paket işlemleri ve anlık görüntüler

Paket kurulumu, güncellemeleri ve kaldırılması Snapper, Btrfs veya bir
anlık görüntü yapılandırması. **Tümünü yükselt** ayrıcalıklı aracılığıyla `pacman -Syyu`'yu çalıştırır
`pkexec` ile yardımcı; bireysel resmi güncellemeler `pacman -S <package>`'yu çalıştırır.ARGVUS bu işlemler için otomatik anlık görüntüler oluşturmaz. Anlık Görüntüler
sayfa bağımsız kalır ve kendi Btrfs/Snapper gereksinimlerini korur.
Pacman'de ayrı olarak yapılandırılan kancalar sistem tarafından kontrol edilmeye devam eder.

### Manuel doğrulama

- Ext4/xfs kökünde sayfayı İngilizce ve Portekizce olarak açın. Portekizce'de,
  **Este sistema de arquivos não é Btrfs** mesajını tam olarak doğrulayın ve doğrulayın
  c/d'nin bir işlem açmaması.
- Tek kullanımlık bir Btrfs VM'de Snapper'ı test edin, Snapper'ı root olmadan test edin
  yapılandırması ve başka bir alt hacme işaret eden `root` adlı bir yapılandırma.
- Mevcut bir kök yapılandırmasıyla anlık görüntüleri listeleyin, ayrıntıları açın,
  boşluklar/özel karakterler içeren anlık görüntü, silme işlemini iptal edin, ardından yalnızca silin
  numarasını ve açıklamasını onayladıktan sonra yeni oluşturulan anlık görüntüyü seçin.
- Reddedilen/iptal edilen kimlik doğrulamayı, korumalı etkin/varsayılan anlık görüntüleri kontrol edin,
  yenileme, klavyede gezinme, yeniden boyutlandırma ve her iki dil. Silme işlemini test etme
  kurtarma için gereken anlık görüntülere karşı.

Yalnızca `argvus` paketi değişir. `de/argvus`'dan `make build` ile yeniden oluşturun
`.pkg.tar.zst` paketini üretin; kurulumu ayrı bir operatör eylemidir.

Dil ve Bölge altındaki Dil, /etc/locale.conf'yu değiştirmeden yerel ayarı kullanıcı profiline (`~/.config/locale.conf`) kaydeder. Oturum, tekrar oturum açtıktan sonra bu tercihi kullanır. Hoş Geldiniz sırasında, F6 İleri veya alt bilgi İleri düğmesi incelemeyi ve ilerlemeyi kaydeder; Shift+Tab, düğmeye odaklanır ve Enter, bilgi sayfaları da dahil olmak üzere onu etkinleştirir.

Bluetooth, Hoş Geldiniz'de yalnızca Denetim Merkezi bir bağdaştırıcı algıladığında görünür. Gizli bir Bluetooth adımına ilişkin mevcut ilerleme korunur.

Dil ve oturum klavye düzeni kişisel tercihlerdir ve yönetici ayrıcalıkları gerektirmez. Yeni profiller varsayılan olarak İngilizce ve ABD klavye düzenini kullanır; mevcut kişisel tercihler korunur. Tarih/saat, NTP, sistem yerel ayarı oluşturma ve konsol (TTY) tuş haritası, sistem genelindeki yönetim davranışlarını korur.

Brezilya oturum klavyesi için  `/etc/locale.conf` `br`'yu seçin, düzeni uygulayın ve ardından Variant altında `abnt2`'yu seçin. Bu, kişisel Hyprland yapılandırmasını yönetici izni olmadan yazar; konsol tuş haritası ayrıdır. ABNT2 ölü anahtarlar kullanır: `~` ve ardından `a`, `ã` üretir ve `~` ve ardından Space, `~` üretir. Değişkenin uygulanması, yapılandırılmış varsayılan düzenini etkinleştirir; etkinleştirme hataları raporlanır.

Zaman Dilimi, TZ'yi yetkilendirme olmadan `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` altında kaydeder. Uygulamaların devralması için oturumu kapatıp tekrar açın. Tarih ve Saat, yönetici yetkisiyle sistem saatini sistem saat diliminde göstermeye ve düzenlemeye devam eder; NTP değişmedi.

Görüntü profilleri ve çalışma alanı atamaları, oluşturulan Lua yapılandırması ve Hyprland yeniden yüklemesi yoluyla uygulanır. Monitör gücü, Hyprland 0.56.2 tarafından desteklenen Lua DPMS dağıtıcısını kullanır.

Gölge oluşturma gücü, Hyprland 0.57 uyumluluğu için 1-4 (varsayılan 4) ile sınırlıdır. 4'ün üzerindeki daha eski saklanan değerler, konfigürasyon yüklenirken sabitlenir.

## Pencere sallanıyor

**Masaüstü ve Giriş → Hyprland → Animasyonlar → Windows**'u açın ve
**Sallanma** kontrolleri. `wobble`'yu arayın veya doğrudan sayfayı açın:

```sh
argvus control-center hyprland animations windows
```

Wobble varsayılan olarak devre dışıdır. ARGVUS varsayılanları sertlik `100`, sönümlemedir
`10`, kütle `1`, yoğunluk `1` ve ağ çözünürlüğü `8`. Sertlik ve kütle kabul edilir
`0.0001–1000`, sönümleme `0–1000`, yoğunluk `0–2` ve bir `2–32` tamsayısını meshleyin.
Daha yüksek ağ çözünürlüğü, oluşturma maliyetini artırır. Ayarlamak için ok tuşlarını kullanın
değerleri yazın veya Enter tuşuna basın. **Wobble varsayılanlarını geri yükle** taslağı değiştirir;**Uygula** bekleyen tüm animasyon düzenlemelerini kaydeder ve Hyprland'i bir kez yeniden yükler.
oturumu yeniden başlatıyorum. Animasyonlar'a geri döndüğünüzde paylaşılan taslak korunur.

Genel Animasyonlar anahtarı, bireysel özelliğini korurken Wobble'ı bastırır
ayarlar. Kanonik değerler ARGVUS durumunda `effects.wobble_*` altında bulunur;
`hypr/state.lua` oluşturulmuştur ve manuel olarak düzenlenmemelidir.

Bu, `argvus-hyprland` tarafından zorunlu kılınan Hyprland **0.57.0 veya daha yenisini** gerektirir
paket bağımlılığı. Sürüm 0.56 bu yapılandırma için desteklenmiyor.
maksimum yoğunluk [yayınlanan 0.57.0 kaynağını](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp) takip eder,
daha yeni belgelerde 3 listelense de bu onu `2` ile sınırlandırır.

Hoş Geldiniz, ARGVUS logosu ve HOŞGELDİNİZ ile açılır. Bir adımı tamamlamadan kontrol listesini göstermek için F6 Sonraki tuşuna basın.

Yeni kurulumlarda varsayılan olarak  `3` `America/New_York` kişisel saat dilimi kullanılır ve yalnızca `us` klavye düzeni etkinleştirilir. Mevcut kişisel ayarlar korunur; sistem saat dilimi değişmez.

Pil ve Bluetooth için görev çubuğu modülü seçenekleri yalnızca ilgili donanım algılandığında görünür. Saklama Uyanık Tut, süspansiyon engelleyiciyi değiştirmeden göstergesini gizler. Yardımcı programlar daraltılmış bir fareyle üzerine gelme çekmecesini destekler. Çalışma alanı simge sunumu açık pencerelere ilişkin simgeleri görüntüler; sistem tepsisi arka plandaki uygulama simgelerini kontrol eder.


Klavye kısayolları sayfası, varsayılan SUPER + A ile **AI aracı seçiciyi (isteğe bağlı argvus-ai paketi)** içerir. App.ai katalog kimliğini kullanır, bu nedenle yeniden eşleme ve devre dışı bırakma, diğer girişlerle aynı standart klavye_kısayol yapılandırmasını izler. Yürütülebilir Lua bağlantısı argvus-ai tarafından sağlanır; katalog girişi tek başına bu isteğe bağlı paket olmadan onu etkinleştirmez.


Kontrol Merkezi > Görünüm > Görev Çubuğu > Simgeler > İsteğe bağlı modüller artık **AI Aracılarını** içeriyor. Varsayılan olarak etkindir ve /taskbar/icons/ai_agents_enabled konumunda saklanır. Yardımcı program grubunu sürdürmek ve yeniden oluşturmak için bunu görev çubuğu taslağında değiştirin ve Uygula; devre dışı bırakılması SUPER + A kısayolunu veya uygulama başlatıcı girişlerini değiştirmez.

Otomatik başlatma anahtarları kullanıcının `.desktop` girişini düzenler veya bir sistem girişinin kullanıcı kopyasını oluşturur. Adlar, komutlar ve etkinleştirme işaretleri `[Desktop Entry]`'dan gelir; ek masaüstü eylem grupları değişmeden kalır.

Hizmet onayları, seçilen kullanıcı/sistem kapsamını ünite ve eylemle birlikte korur. Başka bir kapsamdaki aynı ada sahip bir birim, onaylanmış bir eylemi yeniden yönlendiremez; ayrıntı sayfaları da orijinal kapsamı korur.

Depolama, dosya sistemi envanterinden `/home`, `/boot` ve `/boot/efi` gibi iç içe geçmiş bağlama noktalarını içerir. Diagnostics, mevcut kök ve ana dosya sistemi kullanım kontrolleri için aynı envanteri kullanır.

Yeni geri yükleme güvenliği anlık görüntüleri, Snapper'ın `number` temizleme algoritmasını kullanır. Bunların kaldırılması, temizleme çalıştırıldığında yapılandırılmış numara saklama ilkesine göre yapılır. Temizleme algoritması olmayan mevcut anlık görüntüler değişmez ve manuel oluşturma, mevcut politikasını korur.

Anlık görüntü karşılaştırmaları, desteklenmeyen yol kayıtlarını ayrı olarak raporlar. Geçerli dosyalar inceleme için kullanılabilir durumda kalır; geçersiz kodlamaya, kontrol karakterlerine veya desteklenmeyen bileşenlere sahip yollar, fark veya geri yükleme eylemleri sunmaz.

Güç ayarları, `[Login]` altına ARGVUS oturum açma açılır penceresini yazar ve gerektiğinde dizinini oluşturur. Yeniden yükleme hataları raporlanır. Pil kapağı davranışı `HandleLidSwitch` ile eşleşir; eski `default` bağlamı bir takma addır; AC ve yerleştirilmiş davranış ise ayrı oturum açma anahtarlarını kullanır.

Mirrorlist uygulaması yalnızca yorumları, boş satırları ve HTTP/HTTPS `Server` girişlerini kabul eder. Yükler 64 KiB ile sınırlıdır ve yükselmeden önce ayrıcalıklı yardımcı tarafından tekrar kontrol edilir; `SigLevel` ve `XferCommand` gibi pacman direktifleri reddedilir.Kaldırılan paketler için önbellek temizleme işlemi, önbelleğe alınmış sürümleri korumaz. Depo işlemleri reddi, birden çok depo bölümü tarafından paylaşılanları içerir. Etrafında boşluk bulunan ve boş oluşturulan yerel ayar seçimine sahip ana makine adları reddedilir. Ayarlar dosyası, sembolik bağlantılı hedefleri ve mevcut geçici dosyaları reddeder.

Paket aramaları ve sayfa yüklemeleri, daha önceki bir sayfa veya sorgunun sonuçlarını atar. Geçerli sayfa daha sonra tekrar yüklenir, böylece eski bir sonuç farklı bir görünümü yakın zamanda yenilenmiş olarak işaretleyemez.

İşlem çıktısı sterilize edilir ve sınırlanır. Ekran 10.000 satıra veya 4 MiB'ye kadar tutar ve ihmal edilen çıktıyı bildirir. Yakalanan süreç akışları, canlı yürütme için varsayılan olarak 8 MiB'ye ayarlanarak yapılandırılmış limitlerine uyar; Fazla çıktı boşaltılır ve süreç bittikten sonra raporlanır.

Önyükleme envanteri, çekirdek sürümünü ayrı olarak görüntülerken çekirdek görüntülerini, initramfs dosyalarını ve mkinitcpio ön ayarlarını bulmak için Arch çekirdeği `pkgbase`'yu okur. Değişmez mkinitcpio dizileri, parantezleri eklemeden veya yapılandırmayı yürütmeden okunur.

Kapak, güç düğmesi ve uyanık tutma için yapılan güç değişiklikleri, yalnızca çalışma zamanı eylemi başarılı olduktan sonra standart yapılandırmaya kaydedilir. Yetkilendirmenin iptal edilmesi, daha önce kaydedilen değere dokunulmadan kalır. Hiçbir ARGVUS oturum açma girişi bir değer sağlamadığında, ana oturum açma yapılandırması bir geri dönüş olarak okunur.

Saklama değişikliği başarısız olduktan sonra sayfa, değerlerini tekrar göstermeden önce mevcut ayarları yeniden okur. Yapılandırma seçimi ilk girişte başlar.

ARGVUS paket işlemleri, seçici geri yüklemeyle özel bir kilidi paylaşır. Kilit tutulduğu sürece yükleme, kaldırma, yeniden yükleme, sürüm düşürme, meta veri yenileme, önbellek temizleme ve depo veya yansıtma değişiklikleri reddedilir. Bu koordinasyon, ARGVUS yardımcısı aracılığıyla başlatılan operasyonları kapsar; harici pacman ve AUR araçları bu kilidi kullanmaz.

Bir arka plan işlemi değişiklikleri uygularken, `q` ve `Ctrl+C`, Kontrol Merkezini açık tutar ve bir bekleme bildirimi gösterir. Bu aynı zamanda eylemi başlatan sayfadan ayrıldıktan sonra da geçerlidir. Sonucunu bekleyin, ardından tekrar çıkış isteğinde bulunun. Sıradan salt okunur yenilemeler çıkışı engellemez.

Wi-Fi şifresi, manuel DNS ve ağ arama alanları köşeli ayraçlı yapıştırmayı kabul eder. Yapıştırılan denetimler ve satır sonları atlanır. Desteklenmeyen Yalnızca Ctrl veya Yalnızca Alt karakter kısayolları harf eklemez. Editörler, Ctrl+Alt olarak kodlanmış yazdırılabilir AltGr olaylarını kabul eder; Ctrl+S yalnızca Alt olmadan kaydeder. Paylaşılan metin düzenleyici, görsel sütunlarda dikey gezinmeyi korur ve geniş karakterler için yatay kaydırmayı hizalar.

Mikrofonlar ve varsayılan olmayan çıkışlar için ses düzeyi ve sessize alma değişiklikleri artık kayıtlı varsayılan çıkış tercihlerinin üzerine yazılmaz. Arka uç bu tercihleri ​​yalnızca seçilen düğümü geçerli varsayılan çıktı olarak tanımlayabildiğinde sürdürür. Sayfa, PipeWire tarafından bildirildiği üzere %100'ün üzerinde güçlendirilmiş hacimleri gösteriyor; açık birim düzenlemeleri %0-100 ile sınırlı kalır.

Donanıma Genel Bakış, ticari adları parantez içinde koruyarak GPU modellerini PCI adreslerinden ve cihaz sınıflarından ayrı olarak tanımlar. Özeti, oturum için sorgulanan OpenGL oluşturucuyu ve yazılım aygıtları da dahil olmak üzere Vulkan sondası tarafından döndürülen tüm aygıt adlarını gösterir. Bu genel sonuçlar, her bir fiziksel GPU'nun yetenekleri olarak sunulmamaktadır.

Bir yönetici, otomatik oturum açma için yapılandırılan hesabı kilitlediğinde veya kaldırdığında, hesap sağlayıcı öncelikle greetd otomatik oturum açma girişini devre dışı bırakır. Bu temizleme başarısız olursa hesap işlemi reddedilir. Daha sonra kilitleme veya kaldırma işlemi başarısız olursa otomatik oturum açma devre dışı kalır. Başka bir hesap için otomatik oturum açma korunur; bir hesabın kilidinin açılması otomatik oturum açmayı etkinleştirmez. Mevcut oturumlar kapatılmaz ve greetd yeniden başlatılmaz.


## Klavye kısayolları oluşturun ve yönetin**Masaüstü ve Giriş → Klavye Kısayolları**, **Kısayol Oluştur** ve **Tüm Kısayollar**'ı içeren bir menü açar. Mevcut CLI takma adları (`keybindings`, `shortcuts`, `keyboard`) bu menüyü açar; Bir çocuğu doğrudan açmak için `create` veya `list` ekleyin. Küresel arama aynı zamanda her iki çocuğu da bulur.

**Kısayol Oluştur** bir açıklama, klavye kombinasyonu ve eylem türü ister. **Geçerli kısayol**'u seçin ve `Enter` tuşuna basın, ardından istediğiniz tuş kombinasyonuna basın. **Kısayolu değiştir** aynı yakalamayı başlatır; hiçbiri bir metin düzenleyiciyi açmaz. Yakalama yalnızca taslağı değiştirir. Bir kombinasyon zaten kullanımdaysa, açıklama ve komut doldurulmadan önce bile çakışma hemen gösterilir. **Değiştir** onaylandığında kombinasyon seçilir ve seçiminiz kaydedilir; **Oluştur**/**Kaydet**'e kadar kısayol oluşturmaz veya başka bir kısayolu devre dışı bırakmaz. Çakışmanın iptal edilmesi önceki kombinasyonu korur. Kısayola basıldığında bir **Komut** çalışır; yaratılış onu asla çalıştırmaz. **Hyprland dağıtıcısı** `window.float` gibi desteklenen bir Lua dağıtıcı adını ve JSON nesnesi gibi isteğe bağlı bağımsız değişkenleri (örneğin `{"action":"toggle"}`) kullanır. Alt haritalar, fare bağlamaları ve gelişmiş bayraklar bu formun dışında kalır.

Form ve **Eylemler** arasında geçiş yapmak için `Tab`'yu, **Oluştur** ile **İptal** arasında geçiş yapmak için `←/→`'yu ve seçilen eylemi çalıştırmak için `Enter`'yu kullanın. İptal et veya `Esc` taslağı iptal eder. Oluşturduktan sonra **Tüm Kısayollar** yeni öğe seçili ve görünür şekilde açılır. Özel girişler **Özel Kısayollar**'da görünür; arama açıklama, tuşlar ve eylemle eşleşir. `Enter` açıklamayı, eylemi ve tuşları düzenler, `Space` etkinleştirir/devre dışı bırakır ve **Sil** satırını seçip `Enter` tuşuna basmak, İptal odaklı bir onay açar. Onaylamak kaydı siler ve yenilenen listeye geri döner; iptal etmek onu korur.

Aynı bağlamda etkinleştirilmiş kısayollar, bir kombinasyonu sessizce yeniden kullanamaz: yapılandırılabilir çakışmaların değiştirilmesi için onay gerekir ve korumalı kısayollar değiştirilemez. Değiştirme, yeni kısayolu kaydeder ve tek bir yapılandırma işleminde çakışmaları devre dışı bırakır. **Varsayılanı geri yükle** yalnızca resmi girişler için geçerlidir; **Tüm kısayolları geri yükle** özel kayıtları korur. Geri yükleme işlemi etkin bir özel kısayolla çakışırsa önce o özel kısayolu değiştirin veya devre dışı bırakın.

Arka planda çalışmayı kaydedin ve uygulayın. Kaydetme başarısız olursa taslak kullanılabilir durumda kalır. Kaydedildikten sonra oturuma yapılan başvuru başarısız olursa sayfa, oturumun kaydedildiğini belirtir ve yeniden denemek için aynı kimliği korur. Özel kayıtlar, data/generated/hypr/custom-shortcuts.lua'ya yansıtılan kanonik yapılandırmada `/hyprland/custom_shortcuts`'da yayınlanır; oluşturulan dosyaları düzenlemeyin.

Kısayol hataları TUI'nin içinde kalır: açılır pencere kendisini sarılmış mesaja göre boyutlandırır ve **Tamam'ı Girin**'i görünür tutar. Teknik kısayol tanılamaları stdout/stderr yerine yerel günlüğe ( `data/generated/hypr/custom-shortcuts.lua` `journalctl -t argvus-control-center`) gider. Kaydedilen ancak uygulanamayan bir silme işlemi bu ayrımı bildirir; daha önce etkin olan bağlamayı kaldırmak için oturum yapılandırmasını yeniden yükleyin.

**Görev Çubuğu → Simgeler → İsteğe bağlı modüller**'deki **Güncelleme Durumu**, mevcut güncelleme sayısını görüntüler ve varsayılan olarak etkindir. **AUR'yi Kontrol Et** varsayılan olarak devre dışıdır ve `paru` veya `yay` gerektirir; ayarlardan birini kaydetmek için görev çubuğu taslağını uygulayın. Denetimler başlangıçta ve saatlik olarak gerçekleştirilir; göstergeye tıklamak resmi Paket güncellemeleri sayfasını açar.

## Genel pencere modu

**Denetim Merkezi → Masaüstü ve Giriş → Hyprland → Windows → Mod**'u açın ve **Pencereleri Döşe (Varsayılan)** veya **Kayan Pencereler**'i seçin. Aynı seçiciye görev çubuğu yardımcı program grubundaki pencere modu simgesinden de ulaşılabilir.Tercih her monitör ve çalışma alanı için geçerlidir ve oturum kapatıldıktan sonra hatırlanır. Kayan, mevcut uygulama pencerelerini ve yeni açılan pencereleri dönüştürür. Döşeme yalnızca bu politikanın dönüştürdüğü pencereleri geri yükler; Halihazırda yüzen diyaloglar ve diğer pencereler yüzer durumda kalır. Kayan etkinken açıkça sabitlenen bir pencere, iğnesini korumak için kayan durumda kalır. Döşenmiş düzen yeniden hesaplanır; önceki bölünmüş ağacın tamamı geri yüklenmez.

`SUPER + SHIFT + Space` Tile'da pencere başına normal davranışını koruyor. Floating'de bir pencereyi döşemeye geri döndürmez. Paneller, görev çubuğu ve bildirimler katman kabuğu yüzeyleridir ve etkilenmez. Tam ekran ve büyütme, Hyprland'ın normal davranışını kullanmaya devam ediyor.

Kanonik tercih $XDG_CONFIG_HOME/argvus/config/hyprland.json'da `/hyprland/window_mode`'dur. Oluşturulan projeksiyonu  `$XDG_CONFIG_HOME/argvus/config/hyprland.json` `data/generated/hypr/window-mode.lua`'dur; düzenlemeyin. Pencere başına geri yükleme kaydı, `$XDG_RUNTIME_DIR/argvus` kapsamında bulunur, kapsam birleştirici örneğine alınır ve yapılandırma yeniden yüklemelerinden sağ çıkar. Yeni bir oturumda tekrar kullanılmaz.

Uygulama beklemedeyse veya başarısız olursa, arayüz bunu kaydedilen tercihten ayırır. Yeniden denemek için seçeneği tekrar seçin. Bir besteci olmadan, tercih bir sonraki oturum açma işlemi için kaydedilir.

## Pencere düğmeleri ve uygulama çubuğu

Kayan mod, normal uygulama pencerelerine **Küçült**, **Büyüt / Geri Yükle** ve **Kapat** içeren bir ARGVUS başlık çubuğu ekler. Başlık çubuğuna çift tıklamak büyütmeyi değiştirir. Ek dekorasyon Döşeme ve tam ekranda gizlidir. Kendi denetimlerini çizen uygulamalar her iki çubuğu da görüntüleyebilir; tek tip ARGVUS kontrolleri kasıtlıdır.

Her monitörde ana görev çubuğunun karşısındaki kenarda ikinci bir Yol Çubuğu görünür. Küçültülmüş pencereleri de dahil olmak üzere yalnızca o monitörün geçerli çalışma alanının pencerelerini listeler. Gizli grup sekmeleri fazladan giriş oluşturmaz. Küçültülmüş girişler orijinal monitör/çalışma alanı ilişkilerini korur. Etkinleştirmek için etkin olmayan bir pencereyi, simge durumuna küçültmek için etkin bir pencereyi veya geri yüklemek için simge durumuna küçültülmüş bir pencereyi tıklayın; içerik menüsü geri yükleme, simge durumuna küçültme, büyütme/geri yükleme ve kapatma olanağı sunar. Bilinmeyen uygulama simgeleri bir pencere sembolüne geri döner.

`SUPER + Alt + Up`, ana çubuğu üste ve pencere çubuğunu aşağıya yerleştirir. `SUPER + Alt + Down` bunun tersini yapar. Bir yönün tekrarlanması konumları değiştirmeden bırakır. Döşemenin yalnızca ana çubuğu vardır. Döşemeye geri dönmek, pencere çubuğunu kaldırmadan önce simge durumuna küçültülmüş tüm pencereleri geri yükler ve ardından orijinal döşemeli/değişken ilkesini korur.

Minimizasyon, ayrılmış `special:argvus-minimized` çalışma alanını kullanır. Bu çalışma alanını manuel olarak kullanmayın. `argvus windows service` işlemi, `$XDG_RUNTIME_DIR/argvus` altında birleştirici örneğine özgü çalışma zamanı kaydında kimliği, orijinal çalışma alanını/monitörü, geometriyi, tam ekranı/büyütmeyi ve pin durumunu kaydeder. Kayıtlar, hizmetin yeniden başlatılmasından veya yapılandırmanın yeniden yüklenmesinden sonra hayatta kalır, ancak derleyici oturumları arasında geçiş yapmaz. Bir pencerenin kapatılması kaydının kaldırılmasına neden olur. Bir monitör kaybolursa, geri yükleme mevcut bir çıkışı kullanır. Eksik veya arızalı bir kurtarma çubuğu, yeni en aza indirmeleri önler; başarısız restorasyon, kaydı ve kurtarma çubuğunu korur.

Oturum, `argvus-windows.service` ve `argvus-window-bar.service`'ya sahiptir. İkincisi, Waybar çocuğunu Tile'da hareketsiz tutuyor. Hyprbars eklentisi, x86_64 üzerinde Hyprland 0.57.0 ABI için oluşturulmuş `argvus-hyprland` tarafından paketlenmiştir; kullanıcının ana dizininde derlenmez. Eklentiyi bir besteci ABI güncellemesiyle yeniden oluşturun.

Komutlar: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` ve `minimize|restore|activate|maximize|close ADDRESS`. Pencere adresleri listeden gelir. `position top --persist`, ana çubuğu yeniden başlatmadan tercihi saklar/yansıtır. Yapılandırma ve çalışma zamanı hatası ayrıntıları günlüğe yazılır; görünür eylem hataları da çevrilmiş bir bildirim gösterir.

## Windows alt menüsü ve uygulamanın kapatılması**Masaüstü ve Giriş → Hyprland → Windows** grupları **Kurallar**, **Boşluklar** ve **Mod**.
Kurallar, uygulama başına yerleşimi düzenler; Boşluklar iç ve dış boşlukları düzenler; Mod
kurallar formundan ayrı olarak doğrudan Döşeme/Kayan seçiciyi içerir.
Arama aynı sayfaları açar ve Geri, Hyprland'dan önce Windows'a döner.

Doğrudan rotalar `argvus control-center hyprland windows`'dur ve isteğe bağlı olarak takip edilir
`rules`, `spaces` veya `mode` tarafından. Mevcut `window-rules` ve
`appearance window-spaces` rotaları takma ad olarak kalır ve yeni kırıntıları gösterir.

`SUPER + Q`, özel Kontrol Merkezi penceresini hemen kapatır.
Kitty'nin işletim sistemi penceresi onayı. Uygulanmayan taslaklar pencere açıldığında atılır.
bu şekilde kapalı. Geçersiz kılmanın kapsamı `argvus-control-center` penceresine ayarlanmıştır
sınıf; normal terminaller ve diğer TUI'ler yakın teyit politikalarını korurlar.

`SUPER + Alt + Space`, görev çubuğu ve Kontrol Merkezi ile aynı tercihi kullanarak genel Döşeme/Kayan mod arasında geçiş yapar. Eşdeğer komut `argvus config window-mode toggle`'dur. Eşzamanlı çağrılar serileştirilir; Geçiş, uygulama beklemedeyken bile kaydedilen tercihi kullanır. Kısayol, Klavye Kısayolları'nda değiştirilebilir veya devre dışı bırakılabilir; `SUPER + Shift + Space` bireysel pencere eylemini koruyor.

## Git üzerinden tema yükleme

Mevcut Git kimlik bilgileriyle HTTPS veya SSH üzerinden skeleton deposunu yükleyin. Kurulum kullanıcıya özeldir ve temayı etkinleştirmez. `--ref` dal veya etiket seçer; `--replace` özel temayı değiştirir. Kurulu resmi kimlikler korunur.

```sh
argvus theme install https://github.com/USER/argvus-theme-NAME.git
argvus theme install https://gitlab.com/USER/argvus-theme-NAME.git --ref v1.0
argvus theme install git@host:USER/argvus-theme-NAME.git --replace
argvus theme apply NAME
argvus theme uninstall NAME
```

Tema **Custom themes** içinde görünür; `r` yeniler. CLI veya Control Center üzerinden etkin temayı kaldırmadan önce başka bir tema uygulayın. Değiştirdikten sonra yeniden uygulayın.

`src/usr/share/argvus/themes/<id>/` içinde kimliği eşleşen tek bir schema 2 teması gerekir. Görseller temada veya `src/usr/share/` altında sürümlenmelidir. Betikler çalıştırılmaz. Alt modüller ve Git LFS desteklenmez. `git` ve SSH için `openssh` gerekir. Hedef: `$XDG_DATA_HOME/argvus/themes/<id>`. Resmi paketleri pacman yönetir.
