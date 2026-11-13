---
title: Temaları yükleme
description: Resmi ARGVUS tema paketlerini pacman ile yükleyin.
---

Resmi temalar Arch Linux paketleri olarak yayınlanmaktadır. Her tema `argvus-theme-<THEME_NAME>` adlı kendi paketidir.

## Temalar nereden alınır

Her resmi temanın kaynağı ARGVUS proje organizasyonunda yaşar:

- Her tema, aynı ada sahip bir paket olarak oluşturulmuş `argvus-theme-<THEME_NAME>` (örneğin `argvus-theme-dracula`) adlı bir depodur.

Tema paketleri `argvus-appearance`'ya bağlıdır. Görünüm bileşeninin mevcut olması için önce [Kurulum](/docs/getting-started/installation/) bölümünde açıklandığı gibi ARGVUS'u yükleyin.

## Tek bir tema yükleyin

Tüm resmi temaları tek seferde yüklemek için `argvus-themes` meta paketini yükleyin:

```bash
sudo pacman -S argvus-themes
```

Yayınlanan tüm `argvus-theme-*` paketlerine bağlıdır. ARGVUS Karanlık ve ARGVUS
Light zaten `argvus-appearance` tarafından gönderiliyor.

Paket adıyla bir tema yükleyin:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```

Örneğin:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```

Paket, `/usr/share/argvus/themes/<theme-id>/theme.toml` (isteğe bağlı `overrides/` ile birlikte adı, kategorisi, duvar kağıdı ve paleti) tema bildirimini ve temanın duvar kağıdını `/usr/share/backgrounds/argvus/` altına yükler. Tema uygulandığında her bileşenin dosyaları paletten oluşturulur. Ana dizininize yazmaz ve mevcut temanızı değiştirmez. Daha sonra Kontrol Merkezi'nden seçin, bkz. [Resmi temalar](/docs/components/argvus-themes/official-themes/).

## Birkaç tema yükleyin

Her temayı yükleyen bir meta paket yoktur. İstediğiniz temaları tek komutla yükleyin:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## Mevcut paketler

Aşağıdaki paketler bugün yayınlanan resmi tema paketleridir.

**Karanlık temalar**

| Paket | Tema |
| --- | --- |
| `argvus-theme-one-dark` | Bir Karanlık |
| `argvus-theme-dracula` | Drakula |
| `argvus-theme-silver-dark` | Gümüş Koyu |
| `argvus-theme-slate-dark` | Arduvaz Koyu |
| `argvus-theme-universe` | Evren |
| `argvus-theme-gruvbox-high-dark` | Gruvbox Yüksek Koyu |
| `argvus-theme-gruvbox-dark` | Gruvbox Koyu |
| `argvus-theme-rose-pine` | Gül Çamı |
| `argvus-theme-tokyo-night` | Tokyo Gecesi |
| `argvus-theme-solitude` | Yalnızlık |
| `argvus-theme-sunset` | Gün batımı |
| `argvus-theme-matrix` | Matris |
| `argvus-theme-monokai-dark` | Monokai Koyu |

**Açık temalar**

| Paket | Tema |
| --- | --- |
| `argvus-theme-github-light` | GitHub Işık |
| `argvus-theme-solarized-light` | Solarize Işık |
| `argvus-theme-nord-light` | Kuzey Işık |
| `argvus-theme-one-light` | Bir Işık |
| `argvus-theme-everforest-light` | Everforest Işığı |
| `argvus-theme-catppuccin-latte` | Catppuccin Latte |
| `argvus-theme-gruvbox-light` | Gruvbox Işık |
| `argvus-theme-watercolor` | Suluboya |

Yerleşik **ARGVUS Dark** ve **ARGVUS Light** temalarının ayrı bir pakete ihtiyacı yoktur; `argvus-appearance` ile birlikte gelirler.

## Neyin yüklü olduğunu kontrol edin

Yüklü tema paketlerini aşağıdakilerle listeleyin:

```bash
pacman -Qs argvus-theme
```

Seçicide mevcut olan temaları görmek için Kontrol Merkezini açın ve **Görünüm → Temalar → Resmi Temalar** seçeneğine gidin. Yüklenen ancak bu listede eksik olan bir tema paketleme sorunudur; [Temalar kılavuzunun](/docs/components/argvus-themes/) sorun giderme bölümüne bakın.

## Kendinize ait bir tema ekleyin

Bir tema bir bildirimdir, dolayısıyla kendi temanızın pakete ihtiyacı yoktur. Oluştur
`~/.local/share/argvus/themes/<id>/theme.toml`, burada dizin adı
tema kimliği (küçük harfler, rakamlar ve tek kısa çizgiler):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```

Yalnızca `bg`, `fg` ve `accent` gereklidir: diğer tüm renklerin varsayılan değerleri vardır
onlardan türetilmiştir ve manifest, yalnızca onu değiştirmek için birini tanımlar. Kontrol et,
daha sonra bunu başka bir tema gibi uygulayın:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```

`argvus theme list`, temalarınızı resmi olanların yanında gösterir (resmi bir
aynı kimliğe sahip tema kazanır). Temalarınız tema menüsünde görünür
(`SUPER + Shift + T`); giriş öncesi karşılayıcı ana dizininizi okuyamaz, bu nedenle
temalarınızdan biri aktifken uyguladığınız son resmi temayı korur.

Eski ARGVUS sürümlerinin altında yazdığı temaya özel kopyalar`~/.config/argvus/data/<app>/themes/` artık okunmuyor. İlk tema anahtarı
güncelleme onları `~/.config/argvus/data/backups/theme-copies-<time>/`'ya taşıdıktan sonra;
Düzenlediğiniz bir dosyayı saklamak için `overrides/` altında kendi temanıza yerleştirin ve
manifest'in `[overrides]` tablosunda bunu adlandırın.

## Bir temayı kaldır

Tek bir tema paketini şununla kaldırın:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```

Bir tema paketini kaldırmak, içe aktarılan profillerinizi silmez. Kaldırılan tema şu anda uygulanan temaysa, kaldırmadan önce Kontrol Merkezi'nden başka bir tema seçin.

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
