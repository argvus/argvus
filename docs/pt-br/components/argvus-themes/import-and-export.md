---
title: Importando e exportando temas
description: Exporte o tema ativo como um tema seu, importe um arquivo de tema, aplique-o e exclua-o.
---

Exportar e importar trabalham com **temas**, no mesmo formato dos temas oficiais. Exportar salva o tema ativo, com a sua cor de destaque e o seu wallpaper, como um tema novo seu. Importar instala um arquivo de tema entre os seus temas. As duas ações ficam em **Control Center → Aparência → Temas**, e os seus temas aparecem em **Temas personalizados**.

## O que um tema exportado contém

O arquivo, `argvus-theme-<id>.tar.gz`, guarda um diretório de tema, organizado como um tema oficial:

```text
<id>/theme.toml        o manifesto (schema 2): nome, categoria, paleta, wallpaper
<id>/appearance.json  suas configurações visuais, incluindo layout, efeitos, fontes e apresentação do painel
<id>/effects.conf     valores de efeitos por superfície do tema ativo, quando houver
<id>/overrides/...     os overrides do tema de origem, se houver
<id>/wallpaper.<ext>   o wallpaper que estava na tela ao exportar
```

O manifesto parte do tema ativo, com o seu nome, um id novo, a sua cor de destaque na paleta e o wallpaper ao lado. O perfil visual preserva as configurações visuais atualmente aplicadas no ARGVUS, incluindo espaços de janelas, bordas, margens da taskbar, apresentação do painel, cores, efeitos e fontes. Wallpaper e cor de destaque personalizados são salvos com o tema. Aplicar qualquer tema reaplica todo o perfil dele (ou os padrões do ARGVUS quando não há perfil), substituindo alterações visuais independentes feitas desde a última aplicação. Trocar entre Sticky e Spaced sempre reaplica a geometria de janelas e taskbar daquele modo; salve a geometria pessoal em um tema personalizado exportado para mantê-la quando aplicar esse tema. Configurações não visuais, como áudio, teclado e rede, não fazem parte de um tema.

A configuração visual aplicada permanece na reinicialização ou recarga da sessão. A preparação da sessão reconstrói os arquivos necessários a partir da configuração atual, sem restaurar a geometria do tema original.

As alterações feitas depois de aplicar um tema são temporárias até a próxima troca de tema ou modo. Exporte o tema em **Control Center → Aparência → Temas → Exportar tema** para preservar a configuração visual atual e restaurá-la depois aplicando o tema exportado.

Os seus temas ficam instalados em `~/.local/share/argvus/themes/<id>/`. Extrair um arquivo ali também instala o tema.

## Exportar o tema ativo

1. Abra **Control Center → Aparência → Temas**.
2. Escolha **Exportar tema**, ou pressione `e` numa página de Temas.
3. Digite um nome para o tema. O campo mostra o arquivo que será gravado.
4. Pressione `Enter`.

O id vem do nome: letras minúsculas, dígitos e hífens. Quando o id já é usado por um tema instalado ou por um arquivo na sua pasta pessoal, são acrescentados `-2`, `-3` e assim por diante. O arquivo é gravado na sua pasta pessoal, por exemplo `~/argvus-theme-meu-tema.tar.gz`. O ARGVUS confere o tema antes de gravá-lo: o manifesto precisa carregar e todos os componentes precisam ser gerados nos dois modos.

## Importar um tema

1. Copie o arquivo `.tar.gz` para a sua pasta pessoal (`~`). O ARGVUS lista os arquivos dali que contêm um tema.
2. Abra **Control Center → Aparência → Temas → Importar tema**, ou pressione `i` numa página de Temas.
3. Selecione o arquivo e pressione `Enter`.

O ARGVUS confere o arquivo antes de instalá-lo. Ele precisa conter um único diretório com um `theme.toml`, só com caminhos relativos (sem `..` e sem links), no máximo 128 entradas e um limite de tamanho por arquivo; o id precisa ser igual ao nome do diretório, o wallpaper precisa estar no arquivo e todos os componentes precisam ser gerados nos dois modos. Um perfil visual incluído precisa usar seções aceitas e JSON válido. Se alguma verificação falhar, nada é instalado.

O tema entra em **Temas personalizados**. Importar não aplica o tema: você escolhe quando.

### Conflito de nomes

- Se um tema seu tem o mesmo id, o ARGVUS pergunta **Substituir o seu tema com o mesmo ID?**. Escolha **Substituir** para trocá-lo, ou pressione `Esc` para manter o seu. O tema substituído fica guardado como backup até o novo ser instalado.
- Um arquivo com o id de um tema oficial é recusado. Exporte-o de novo com outro nome.

## Aplicar um tema personalizado

1. Abra **Control Center → Aparência → Temas → Temas personalizados**.
2. Selecione o tema e pressione `Enter`.

Toda a configuração exportada do tema é aplicada, incluindo wallpaper, espaços, bordas, cor de destaque, efeitos, fontes e apresentação do painel. Os valores visuais substituem as configurações atuais. Em seguida, o ARGVUS reinicia a taskbar, o serviço de recarga do Hyprland, o wallpaper e o Control Panel para que cada superfície use o tema personalizado.

## Excluir um tema personalizado

1. Abra **Control Center → Aparência → Temas → Temas personalizados**.
2. Selecione o tema e pressione `d`.
3. Confirme com `y`, ou pressione `n` ou `Esc` para cancelar.

O diretório do tema é removido de `~/.local/share/argvus/themes`. Se ele era o tema ativo, o ARGVUS troca antes para o **ARGVUS Dark**.

## Perfis de versões anteriores

Versões anteriores exportavam *perfis de tema* (`profile.tar.gz`), uma cópia das configurações de aparência. Na primeira vez que uma página de Temas abre, cada perfil em `~/.local/share/argvus/themes/<nome>/` é convertido num tema feito a partir do tema base dele, com a cor de destaque e o wallpaper do perfil. O perfil original, e a antiga lista `custom-themes.json`, vão para `~/.config/argvus/data/backups/theme-profiles-<segundos>/`, e a linha de status informa a conversão. Um perfil cujo tema base não está instalado só é movido para o backup.

## Compartilhar um tema

Um tema exportado é um arquivo comum. Copie-o para outra máquina com ARGVUS e importe-o lá.

## Solução de problemas

- **O arquivo não aparece na lista.** Ele precisa estar diretamente na sua pasta pessoal, terminar em `.tar.gz` e conter `<id>/theme.toml`.
- **A importação é recusada.** A linha de status diz o motivo: um tema oficial com o mesmo id, wallpaper ausente, manifesto que não carrega ou componente que não é gerado.
- **A exportação é recusada.** O nome precisa de pelo menos uma letra ou um dígito, e o tema ativo precisa estar instalado.
