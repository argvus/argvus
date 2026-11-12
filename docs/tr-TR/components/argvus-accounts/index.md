---
title: Hesapları
description: Argvus hesapları aracılığıyla yerel kullanıcı profilleri, görünen adlar, avatarlar ve grup üyelikleri.
---

`argvus accounts`, ARGVUS masaüstünün yerel kullanıcı hesaplarını listeler ve düzenler. Bu özel bir
 `argvus` `/usr/lib/argvus/commands/accounts` konumunda kurulu `argvus` paketinin yürütülebilir dosyası.
argvus komutu onu başlatır ve yalnızca diğer hesapları veya sistem verilerini değiştiren işlemler
root olarak çalıştırın.

## Komutlar

```text
argvus accounts list [--all]                   list users (--all includes system accounts)
argvus accounts show USER                      show account details
argvus accounts name USER "DISPLAY NAME"       change another user's display name
argvus accounts avatar USER IMAGE              set another user's avatar
argvus accounts avatar USER --remove           remove another user's avatar
argvus accounts groups USER                    list groups
argvus accounts groups USER --add G --remove G change groups (authorization required)
argvus accounts passwd USER                    change a password (reads three lines from stdin)
argvus accounts self                           show your own account
argvus accounts self name "DISPLAY NAME"       change your own display name
argvus accounts self avatar IMAGE              set your avatar
argvus accounts self avatar --remove           remove your avatar
argvus accounts self groups                    list your groups
argvus accounts --verbose ...                  print diagnostic information to stderr
```

## İsimleri ve avatarları görüntüle

Görünen ad standart GECOS alanında saklanır. Unix kullanıcı adı hiçbir zaman değişmez.

Avatar, PNG, JPEG veya WebP formatında, en fazla 20 MiB boyutunda bir resimdir. Resim 256×256 olarak yeniden boyutlandırıldı
tarafından okunan ücretsiz masaüstü konumu olan ~/.face konumunda `0644` moduyla PNG olarak kaydedildi.
selamlayan ve çoğu görüntü yöneticisi tarafından. Kendi avatarınız ana dizininizin içine yazılır ve
yetki gerektirmez.

## Yetkilendirme

Yalnızca  `~/.face` `list`, `show`, `self`, `self groups` ve kendi avatarınız yetkilendirme olmadan çalışır. Her
diğer değişiklik masaüstünün polkit aracısından yetki ister. ARGVUS kuralı
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) kimin kimlik doğrulaması yapması gerektiğine karar verir:

- yönetici grubundaki bir kullanıcı (`wheel`, Arch varsayılanı veya `sudo`) kendi kullanıcılarıyla onaylar
  şifre;
- başka herhangi bir kullanıcı root şifresiyle onaylar.

| Operasyon | Yetkisiz | Yetkilendirme ile |
| --- | --- | --- |
| Kullanıcıları listeleyin, bir hesap gösterin, grupları listeleyin | ✓ | ✓ |
| Kendi avatarınızı değiştirin | ✓ | ✓ |
| Kendi görünen adınızı değiştirin | — | ✓ |
| Kendi şifrenizi değiştirin | — | ✓ (geçerli şifreyi doğruladıktan sonra) |
| Başka bir kullanıcının adını, avatarını, gruplarını veya şifresini değiştirme | — | ✓ |

Kendi şifrenizi değiştirdiğinizde öncelikle mevcut şifreniz kontrol edilir. Yanlış şifre reddedildi
Yetkilendirme istemi görünmeden önce.

Komutu `sudo` ile çalıştırmanıza gerek yoktur. Komut, polkit'ten şu durumlarda yetkilendirme ister:
buna ihtiyacı var.

## Güvenlik sınırları

- `passwd` standart girişten mevcut, yeni ve onay şifrelerini her satıra bir tane olacak şekilde okur,
  ve bunları asla argüman olarak kabul etmez. Yönetici tarafından kullanıldığında mevcut şifre satırı boş olabilir.
  başka bir kullanıcının şifresini sıfırlar. Örnek:

  ```sh
  printf '%s\n%s\n%s\n' "$OLD" "$NEW" "$NEW" | argvus accounts passwd ghost
  ```

  Etkileşimli olarak üç satırı yazın ve her birinin ardından Enter tuşuna basın. Giriş ekranda gösterilir.
  yazıldığı gibi terminal.
- UID 1000'in (ve `nobody`) altındaki bir hesabın düzenlenmesi reddedilir ve bu nedenle kilitlenip silinir
  ve şifresinin süresinin dolması. Kendi hesabınız kilitlenemez veya silinemez.
- GID 1000'in altındaki bir grubu veya bir yönetici grubunu (`wheel`, `sudo`) yeniden adlandırmak veya silmek,
  reddetti; kendinizi bir gruptan çıkaramamanız dışında, üyeleri yine de değiştirilebilir.
  yönetici grubu.
- Yönetici üyeliği `wheel` veya `sudo` grubudur. Kontrol Merkezi yeni ekler
  Sistemde varsa yöneticiler `wheel`'ya, aksi takdirde `sudo`'ya. Kendinin kaldırılması
  Bir düzenleme yoluyla yönetici üyeliği reddedilir.

## Örnekler

```sh
# Set your display name and avatar after the first login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/profile.png

# Check your groups, then add another user to the wheel group (authorization required)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrator: show a user's details and reset that user's name
argvus accounts show ghost
argvus accounts name ghost "Ghost Account"
```

## Sorun Giderme

- `pkexec was not found`: `polkit`'yu yükleyin. `argvus` paketi buna bağlıdır.
- Yetkilendirme istemi görünmüyor: polkit kimlik doğrulama aracısının çalışıp çalışmadığını kontrol edin
  oturumunuzda. `argvus` paketi, bir tane sağlayan `hyprpolkitagent`'ya bağlıdır.
- Parolayı değiştirirken `unix_chkpwd` hataları: `pam` paketi yardımcı sağlar.
- Komutun adımlarını stderr'e yazdırmak için `argvus accounts --verbose ...` komutunu çalıştırın.

Ayrıcalık sınırıyla ilgili geliştirici notları için bkz. `docs/internal/argvus-accounts/`.Bir yönetici, otomatik oturum açma için yapılandırılan hesabı kilitlediğinde veya kaldırdığında, hesap sağlayıcı öncelikle greetd otomatik oturum açma girişini devre dışı bırakır. Bu temizleme başarısız olursa hesap işlemi reddedilir. Daha sonra kilitleme veya kaldırma işlemi başarısız olursa otomatik oturum açma devre dışı kalır. Başka bir hesap için otomatik oturum açma korunur; bir hesabın kilidinin açılması otomatik oturum açmayı etkinleştirmez. Mevcut oturumlar kapatılmaz ve greetd yeniden başlatılmaz.
