---
title: ARGVUS'u yükleyin
description: ARGVUS'u yükleyin, güncelleyin, kaldırın ve yapılandırın.
---

ARGVUS, resmi ARGVUS paket deposu aracılığıyla Arch Linux paketi olarak dağıtılır.

## Kurulumdan önce

ARGVUS'u kurmadan önce resmi Arch Linux `multilib` deposunu etkinleştirin. `/etc/pacman.conf`'yu düzenleyin ve hem `[multilib]` bölümüne hem de `Include` satırına ilişkin açıklamaları kaldırın:

```ini
[multilib]
Include = /etc/pacman.d/mirrorlist
```

Ardından paket veritabanlarını senkronize edin ve tam sistem yükseltmesi gerçekleştirin:

```sh
sudo pacman -Syu
```

Hızlı yükleyici ARGVUS deposunu yapılandırır ancak `multilib`'yu sizin için etkinleştirmez.

## Hızlı kurulum

Yeni bir Arch Linux kurulumunda, ARGVUS'u önce başka bir masaüstü ortamı kurmanıza gerek kalmadan doğrudan bir TTY'den kurabilirsiniz.

Çalıştır:

```sh
curl -fsSL https://argvus.github.io/install.sh | sh
```

Yükleyici:

1. ARGVUS veri havuzu imzalama anahtarını içe aktarın ve yerel olarak imzalayın.
2. ARGVUS paket deposunu yapılandırır.
3. Paket veritabanını günceller.
4. `argvus` paketini ve bağımlılıklarını yükler.

Kurulumdan sonra oturumu kapatın ve ekran yöneticinizden **ARGVUS** seçeneğini seçin.

> **Not:** Hızlı kurulum `curl`, `sudo` ve standart Arch Linux paket yönetimi araçlarını gerektirir.

## Yalnızca depoyu yapılandırın

ARGVUS paket deposunu masaüstünü kurmadan yapılandırmak için şunu çalıştırın:

```sh
curl -fsSL https://argvus.github.io/repo-install.sh | sh
```

Daha sonra ARGVUS'u manuel olarak yükleyin:

```sh
sudo pacman -Syu argvus
```

## Manuel kurulum

Depoyu adım adım yapılandırmayı tercih ederseniz aşağıdaki komutları kullanın.

### 1. Depo imzalama anahtarını indirin

```sh
curl -fsSLo /tmp/argvus.gpg https://argvus.github.io/packages/arch/argvus.gpg
```

### 2. İmza anahtarını içe aktarın

```sh
sudo pacman-key --add /tmp/argvus.gpg
```

### 3. Anahtarı yerel olarak imzalayın

```sh
ARGVUS_KEY="$(gpg --show-keys --with-colons /tmp/argvus.gpg | grep '^pub:' | head -n1 | cut -d: -f5)"
sudo pacman-key --lsign-key "$ARGVUS_KEY"
```

### 4. ARGVUS deposunu yapılandırın

```sh
curl -fsSL https://argvus.github.io/packages/arch/argvus.conf \
  | sudo tee /etc/pacman.d/argvus.conf
```

Depo yapılandırmasını `/etc/pacman.conf`'ya ekleyin:

```sh
echo "Include = /etc/pacman.d/argvus.conf" \
  | sudo tee -a /etc/pacman.conf
```

Depo yapılandırması, `argvus` ve `argvus-extras` dahil ARGVUS depolarının `SigLevel = Required TrustedOnly` ile etkinleştirilmesini sağlar. Pacman, paketleri yalnızca imzaları geçerli olduğunda ve yerel anahtarlık tarafından güvenilir olduğunda kabul eder.

### 5. ARGVUS'u yükleyin

```sh
sudo pacman -Syu argvus
```

Kurulumdan sonra oturumu kapatın ve ekran yöneticinizden **ARGVUS** seçeneğini seçin.

## Güncelleme

ARGVUS'u Arch Linux'un geri kalanıyla birlikte güncelleyin:

```sh
sudo pacman -Syu
```

Yalnızca ARGVUS paketini güncellemek için:

```sh
sudo pacman -S argvus
```

> **Öneri:** Arch Linux genellikle bireysel paketleri güncellemek yerine `sudo pacman -Syu` ile tam sistem yükseltmesini önerir.

## ARGVUS'u kaldır

Diğer kurulu paketlerin gerektirdiği bağımlılıkları korurken ARGVUS'u kaldırın:

```sh
sudo pacman -R argvus
```

ARGVUS'u ve kurulu herhangi bir paket için artık gerekmeyen bağımlılıkları kaldırın:

```sh
sudo pacman -Rns argvus
```

### ARGVUS deposunu kaldırın

Paket deposunu da kaldırmak istiyorsanız yapılandırmasını kaldırın:

```sh
sudo rm /etc/pacman.d/argvus.conf
```

Ardından `/etc/pacman.conf`'dan ilgili satırı kaldırın:

```text
Include = /etc/pacman.d/argvus.conf
```

Dosyayı şununla düzenleyebilirsiniz:

```sh
sudo nano /etc/pacman.conf
```

### İmza anahtarını çıkarın

ARGVUS depo imzalama anahtarını pacman anahtarlığından kaldırmak için önce anahtarları listeleyin:

```sh
sudo pacman-key --list
```

Ardından ARGVUS anahtarını anahtar kimliğiyle birlikte kaldırın:

```sh
sudo pacman-key --delete KEY_ID
```

`KEY_ID`'yu ARGVUS imzalama anahtarı kimliğiyle değiştirin.

> **Uyarı:** Bu anahtarı yalnızca ARGVUS deposunu da kaldırdığınızda kaldırın. Diğer depoların hâlâ ihtiyaç duyduğu anahtarları kaldırmayın.

## Kullanıcı yapılandırması

Paket kurulumu, yapılandırmayı `$HOME`'ya yazmaz. ARGVUS, `/usr/share/argvus`'dan paketlenmiş varsayılanları okur ve bunları ana dizininize kopyalamadan çalışır.

Çalışma zamanı yapılandırması şu genel önceliğe uyar:

```text
~/.config/argvus/data/generated/<app>  ->  ~/.config/argvus/data/<component>/<app>  ->  $XDG_CONFIG_HOME/<app>  ->  /usr/share/argvus/<app>  ->  upstream defaults
```

`argvus-config`, `~/.config/argvus/data/generated/` yazan tek bileşendir ve bu projeksiyon etkin katmandır: yönetilen veya yerel bir kopya yalnızca karşılık gelen oluşturulan dosya yokken etkili olur.ARGVUS tarafından yönetilen yapılandırma, `~/.config/argvus/` altında yaşar. Yerel kullanıcı yapılandırmasını destekleyen uygulamalar için isteğe bağlı bir `$XDG_CONFIG_HOME/<app>` geçersiz kılma olanağı mevcuttur.

`argvus --setup` dağıtıcısı isteğe bağlıdır ve açık manuel kullanım içindir
geçersiz kılmalar, kurtarma değil. `~/.config/argvus` silinirse `SUPER + SHIFT + R`
normal profil aracılığıyla kanonik profili ve oluşturulan çalışma zamanı durumunu yeniden oluşturur
boru hattını yeniden yükleyin.

```sh
argvus --setup --copy foot-tui
argvus --setup --copy kitty-tui
argvus --setup --copy-all
```

`--copy-all`, desteklenen TUI profil dosyalarının açık bir izin verilenler listesidir; o
`/usr/share/argvus`'yu klonlamaz ve komut dosyalarını, hizmetleri, varlıkları kopyalamaz,
oluşturulan dosyalar veya `config.json`. Geçerli durumu görmek için `argvus --setup --help` komutunu çalıştırın
seçenekler. Dosyalar kopyalandıktan sonra kullanıcıya ait geçersiz kılmalar ve paket haline gelir
yükseltmeler bunların yerini almaz.

Mevcut bir kopyalanmış yapılandırmayı değiştirmek için `--force` kullanın. ARGVUS, değiştirmeden önce zaman damgalı bir yedekleme oluşturur:

```sh
argvus --setup --copy <app> --force
```

## Hyprland kullanıcı geçersiz kılmaları

Tam yerel konfigürasyon ile artımlı ARGVUS parçaları arasındaki fark da dahil olmak üzere Hyprland geçersiz kılma kılavuzunun tamamı için bkz. [Hyprland geçersiz kılmaları](/docs/argvus-hyprland/hyprland-overrides/).

```sh
mkdir -p ~/.config/argvus/data/hypr
nano ~/.config/argvus/data/hypr/user.lua
```

### Sanal makine geri dönüşü

Bazı sanal makineler Hyprland, Kitty veya Quickshell ile oluşturma sorunları yaşayabilir. ARGVUS, `vmwgfx` grafik sürücüsünü kullanarak sanal makineleri algılar ve gerektiğinde yazılım oluşturmaya olanak tanır.

Algılama sanal makinenizde çalışmıyorsa `user.lua`'da geri dönüşü manuel olarak etkinleştirin:

```lua
-- Virtual machine compatibility
hl.env("LIBGL_ALWAYS_SOFTWARE", "1")
```

`user.lua`'yu oluşturduktan veya değiştirdikten sonra, Hyprland oturumunu kapatın ve tekrar oturum açarak değişkenin oturum tarafından başlatılan uygulamalara uygulanmasını sağlayın.

Şununla doğrulayın:

```sh
echo $LIBGL_ALWAYS_SOFTWARE
```

Beklenen çıktı:

```text
1
```

> **Not:** `LIBGL_ALWAYS_SOFTWARE=1`, Mesa/OpenGL uygulamalarını genellikle `llvmpipe` aracılığıyla yazılım oluşturmayı kullanmaya zorlar. Grafik performansını düşürebilir ve etkilenen sanal makineler için bir yedek olarak kullanılmalıdır.