---
title: Analisador de uso de disco ARGVUS
description: Navegue pelo uso de disco com ncdu em uma janela Kitty com o tema ARGVUS.
---

Abra o **ARGVUS Disk Usage Analyzer** no lançador de aplicativos ou execute:

```sh
argvus du
```

O analisador abre uma janela Kitty dedicada e centralizada, com a interface real do ncdu embutida na moldura ARGVUS. O diretório inicial é `$HOME`. Edite o caminho ou selecione **Escolher** para abrir o gerenciador de arquivos configurado. No Superfile, navegue até um diretório e pressione **Espaço** para selecioná-lo; `q` ou `Esc` cancela. Outros gerenciadores configurados abrem normalmente, mas podem não retornar o diretório escolhido; nesse caso, digite ou cole o caminho no analisador.

Use Tab ou as setas para alternar entre diretório, opções e ações. O gráfico, a porcentagem, o esquema de cores escuro e a ordenação de diretórios primeiro já vêm ativados. As demais opções de varredura e exibição podem ser alternadas antes de selecionar **Analisar**. Em **Resultados**, use os atalhos do ncdu; `q` volta ao formulário e `Esc` interrompe a análise. Pressione `d` para excluir o arquivo ou diretório selecionado. O ncdu pede confirmação; a exclusão é permanente e usa as permissões do usuário atual. O aplicativo não solicita privilégios elevados.

A interface externa acompanha o tema e o idioma ativos do ARGVUS. Kitty e ncdu são necessários; o pacote `argvus-devtools` fornece o ncdu. Se Kitty ou ncdu estiver ausente, o aplicativo informa qual dependência falta.
