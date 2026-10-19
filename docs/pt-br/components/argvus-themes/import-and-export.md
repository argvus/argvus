---
title: Importando e exportando temas
description: Salve a aparência atual como perfil de tema, importe um perfil, aplique-o e exclua-o.
---

Um **perfil de tema** é um arquivo `.tar.gz` que guarda a sua aparência atual para que você possa restaurá-la depois, levá-la para outra instalação do ARGVUS ou compartilhá-la. Os perfis são gerenciados em **Control Center → Aparência → Temas → Exportar / Importar**.

## O que um perfil contém

Um perfil registra:

- o tema ativo e o seu modo (Sticky ou Float);
- a cor de destaque, quando você escolheu uma personalizada;
- os valores de layout, efeitos, transparência e blur, além da fonte e da seleção de cards do Control Panel;
- o wallpaper, quando o wallpaper atual é um arquivo legível.

Arquivos de execução gerados, como a configuração do Hyprland, da Waybar e do shell, não fazem parte do arquivo. O ARGVUS os recria quando você aplica o perfil.

## Exportar a aparência atual

1. Abra **Control Center → Aparência → Temas**.
2. Escolha **Exportar / Importar** e depois **Exportar perfil do tema**. Você também pode pressionar `e` na página de Temas.
3. Digite um nome para o tema. O nome aparece na lista de temas personalizados.
4. Pressione `Enter` para exportar.

O arquivo é gravado no seu diretório pessoal, com o nome do tema seguido da data e hora da exportação, por exemplo `meu-tema-20260103-101500.tar.gz`. Se já existir um arquivo com o mesmo nome, é acrescentado um sufixo numérico.

## Importar um perfil

1. Copie o perfil `.tar.gz` para o seu diretório pessoal (`~`). O ARGVUS procura perfis ali.
2. Abra **Control Center → Aparência → Temas → Exportar / Importar** e escolha **Importar perfil do tema**. Você também pode pressionar `i` na página de Temas.
3. Selecione o arquivo na lista e pressione `Enter`.

O ARGVUS valida o arquivo antes de instalá-lo. A validação verifica o manifesto, as somas de verificação e o wallpaper incluído. Se o arquivo for inválido, a importação é cancelada e nada é instalado.

O perfil importado é adicionado a **Temas personalizados**. Importar e aplicar são etapas separadas: importar apenas adiciona o perfil, e você escolhe quando aplicá-lo.

### Conflitos de nome

Se já existir um tema personalizado com a mesma identidade, o ARGVUS pergunta **Substituir o tema importado existente?**. Escolha **Substituir** para sobrescrevê-lo, ou pressione `Esc` para manter o existente.

### Wallpapers

O wallpaper incluído é restaurado no seu diretório pessoal quando possível:

- o ARGVUS usa o caminho original, quando ele está disponível;
- caso contrário, usa a sua pasta de imagens, ou `~/Pictures/ARGVUS`;
- se já existir um arquivo com esse nome, o arquivo restaurado recebe um sufixo `-imported-N`.

Quando o wallpaper de um perfil não pode ser restaurado de forma alguma, a importação falha em vez de criar um perfil que aponta para um arquivo inexistente.

## Aplicar um tema personalizado

1. Abra **Control Center → Aparência → Temas → Temas personalizados**.
2. Selecione o tema importado e pressione `Enter`.

Aplicar restaura o estado do tema, a cor de destaque, os efeitos, o layout e o wallpaper incluído. Selecionar um perfil recarrega as superfícies afetadas do desktop. Se o wallpaper do perfil não puder ser restaurado como arquivo, o perfil é aplicado com o wallpaper oficial e você verá um aviso.

## Excluir um tema importado

1. Abra **Control Center → Aparência → Temas → Temas personalizados**.
2. Selecione o tema e pressione `d`.
3. Confirme com `Enter`, ou pressione `Esc` para cancelar.

Excluir um tema importado remove o perfil do ARGVUS. O wallpaper é mantido. Se o perfil excluído era o ativo, o ARGVUS volta para o **ARGVUS Dark**.

## Compartilhar um perfil

Um perfil é um arquivo comum. Copie-o para outra máquina que use o ARGVUS e importe-o ali. O perfil não inclui os valores de transparência e blur salvos para temas inativos.

## Solução de problemas

- **O arquivo aparece na lista, mas a importação falha.** O perfil é inválido ou o payload do wallpaper não pode ser lido. Exporte a aparência novamente na máquina de origem.
- **O tema importado mostra o wallpaper oficial.** O wallpaper salvo não pôde ser restaurado como arquivo. Escolha um wallpaper novamente em **Control Center → Aparência → Papéis de parede**.
- **O perfil não aparece na lista.** Confirme que o arquivo está diretamente no seu diretório pessoal e termina em `.tar.gz`.
