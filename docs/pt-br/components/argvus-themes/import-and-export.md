---
title: Importando e exportando temas
description: Exporte o tema ativo como um tema seu, importe um arquivo de tema, aplique-o e exclua-o.
---

Exportar e importar trabalham com **temas**, no mesmo formato dos temas oficiais. Exportar salva o tema ativo, com a sua cor de destaque e o seu wallpaper, como um tema novo seu. Importar instala um arquivo de tema entre os seus temas. As duas ações ficam em **Control Center → Aparência → Temas**, e os seus temas aparecem em **Temas personalizados**.

## O que um tema exportado contém

O arquivo, `argvus-theme-<id>.tar.gz`, guarda um diretório de tema, organizado como um tema oficial:

```text
<id>/theme.toml        o manifesto (schema 2): nome, categoria, paleta, wallpaper
<id>/overrides/...     os overrides do tema de origem, se houver
<id>/wallpaper.<ext>   o wallpaper que estava na tela ao exportar
```

O manifesto é o do tema ativo, com o seu nome, um id novo, a sua cor de destaque como destaque da paleta e o wallpaper ao lado. O tema mantém os dois modos, Sticky e Float, como qualquer tema; o modo continua sendo uma configuração separada. Fontes, efeitos, espaçamentos e as demais configurações do desktop não fazem parte de um tema.

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

O ARGVUS confere o arquivo antes de instalá-lo. Ele precisa conter um único diretório com um `theme.toml`, só com caminhos relativos (sem `..` e sem links), no máximo 128 entradas e um limite de tamanho por arquivo; o id precisa ser igual ao nome do diretório, o wallpaper precisa estar no arquivo e todos os componentes precisam ser gerados nos dois modos. Se alguma verificação falhar, nada é instalado.

O tema entra em **Temas personalizados**. Importar não aplica o tema: você escolhe quando.

### Conflito de nomes

- Se um tema seu tem o mesmo id, o ARGVUS pergunta **Substituir o seu tema com o mesmo ID?**. Escolha **Substituir** para trocá-lo, ou pressione `Esc` para manter o seu. O tema substituído fica guardado como backup até o novo ser instalado.
- Um arquivo com o id de um tema oficial é recusado. Exporte-o de novo com outro nome.

## Aplicar um tema personalizado

1. Abra **Control Center → Aparência → Temas → Temas personalizados**.
2. Selecione o tema e pressione `Enter`.

O tema é aplicado como um oficial, com a cor de destaque e o wallpaper dele, no modo atual.

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
