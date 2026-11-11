---
title: Projeleri
description: Argvus projeleri aracılığıyla proje seçici ve proje yapılandırması.
---

`argvus projects` projelerinizi listeler ve açar. Bu, `argvus`'nun işlem içi bir komutudur.
paket ve `SUPER + O` seçiciyi açar.

## Komutlar

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```

## Proje nedir

Proje, yapılandırmada (`/projects/paths`) listelenen bir dizin veya
`/projects/roots`'da listelenen bir kök. En fazla dokuz proje listeleniyor:
`SUPER + CTRL + N` kısayolları. Bir projeyi açmak Hyprland çalışma alanına odaklanır. İlk kez bir
proje açılır, ARGVUS bir terminal başlatır ve bunun içinde `/projects/editor`'da ayarlanan düzenleyici bulunur.

Yapılandırma `argvus config` aracılığıyla okunur ve yazılır, böylece Kontrol Merkezi ve
aynı listeye bakın.

## Proje çalışma alanları ve görev çubuğu

Projeler özel çalışma alanlarını değil, sıradan adlandırılmış Hyprland çalışma alanlarını kullanır. `SUPER + O` proje numarasını, dizini ve doğrudan `SUPER + CTRL + N` kısayolunu gösterir. Her iki giriş noktası da aynı çalışma alanına odaklanır. Görev çubuğunda yalnızca `Project: name` etiketi, etkin durum ve tam yol ile kısayolu içeren bir araç ipucuyla açık çalışma alanları görünür. Geri dönmek için çalışma alanı düğmesini tıklayın. Görev çubuğu çalışma alanının monitörünü takip eder.

Kurallı dizin yolu, kararlı bir çalışma alanı kimliğini belirler; liste sırasını değiştirmek çalışma alanını değil kısayolu değiştirir. Aynı dizin adına sahip projeler ayrı kalır ve aynı dizine olan sembolik bağlantılar tekilleştirilir. Terminal ve düzenleyici başlatmaları açıkça projenin çalışma alanını hedefler. Mevcut bir süreci/pencereyi yeniden kullanan düzenleyiciler, o uygulamanın kendi çalışma alanı davranışını koruyabilir.

Güncellemeden önce açılan çalışma alanları mevcut pencerelerini ve yerel adlarını korur. Güncellemeden sonra bir projenin açılması, onun yeni yoldan türetilmiş kimliğini kullanır; Uygun olduğunda eski proje pencerelerini kapatın. Eski bir ad dizini güvenilir bir şekilde tanımlamadığından hiçbir otomatik ilişkilendirme veya taşıma girişiminde bulunulmaz. Yinelenen dizin adlarıyla, belirsiz bir ad yerine seçiciyi, bir sayıyı veya tam kurallı yolu kullanın.

Bir projenin açılması, `$XDG_RUNTIME_DIR/argvus/project-workspaces.json`'yu yapılandırılmış proje listesinden ve çevirilerden atomik olarak yeniden oluşturur. Bu, yapılandırma değil, çalışma alanı etiketleri ve araç ipuçlarına ilişkin oturum meta verileridir; düzenlemeyin. Etiketlerini yenilemek için listeyi veya dili değiştirdikten sonra projeyi yeniden açın. Bu entegrasyon için hem `argvus` hem de `argvus-waybar`'nun yanı sıra `argvus-taskbar` yapılandırmasını güncelleyin.