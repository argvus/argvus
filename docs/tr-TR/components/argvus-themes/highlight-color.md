---
title: Vurgu rengini özelleştirme
description: ARGVUS vurgu rengini seçilen temadan bağımsız olarak değiştirin ve tema varsayılanına dönün.
---

**Vurgu rengi**, masaüstündeki odak sınırları, seçim ve etkin durumlar için kullanılan vurgudur. Her temanın varsayılan bir vurgu rengi vardır ancak temayı değiştirmeden bunu istediğiniz renkle değiştirebilirsiniz.

## Vurgu rengini değiştirin

1. **Kontrol Merkezi → Görünüm → Rengi vurgula**'yı açın.
2. **Vurgu rengini düzenle**'yi seçin. Geçerli değer seçeneğin yanında gösterilir.
3. Başında `#` olsun veya olmasın, altı basamaklı bir RGB rengi yazın. Örneğin `3590BD` ve `#3590BD`'nun her ikisi de kabul edilir.
4. Uygulamak için `Enter` tuşuna basın.

Ayrıca alana bir renk de yapıştırabilirsiniz. ARGVUS, yapıştırılan metni yalnızca altı haneli geçerli bir RGB değeri olduğunda kabul eder. `#123` gibi kısa formlar ve onaltılık olmayan karakterlere sahip değerler bir hata mesajıyla reddedilir ve renk değişmez.

Yeni renk masaüstüne hemen uygulanır ve ARGVUS yapılandırmanıza kaydedilir.

## Tema varsayılanına dön

Seçilen temanın tanımladığı renge geri dönmek için **Denetim Merkezi → Görünüm → Rengi vurgula**'yı açın ve **Varsayılan temaya sıfırla** seçeneğini seçin.

## Vurgu renginin temalarla nasıl etkileşime girdiği

- Seçtiğiniz vurgu rengi bağımsız bir geçersiz kılma işlemidir. Başka bir temaya geçtiğinizde varlığını sürdürür, böylece seçtiğiniz renk aynı kalır.
- **Tema varsayılanına sıfırla** sonrasında uyguladığınız tema, vurgu rengini yeniden devralır ve kendi varsayılan rengini kullanır.
- Etkin temanın dışa aktarılması, yeni tema paletinin vurgusu olarak dışa aktardığınızda etkin olan vurgu rengini saklar. Bkz. [Temaları içe ve dışa aktarma](/docs/components/argvus-themes/import-and-export/).

## Değerin saklandığı yer

Vurgu rengi, `$XDG_CONFIG_HOME/argvus/config.json` (genellikle `~/.config/argvus/config.json`) cinsinden ARGVUS yapılandırmanızın bir parçasıdır. `$XDG_CONFIG_HOME/argvus/data/generated` altında oluşturulan dosyaları düzenlemeyin: ARGVUS, bunları yapılandırmadan yeniden yazar.