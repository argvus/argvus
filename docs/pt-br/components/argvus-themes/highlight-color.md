---
title: Personalizando a cor de destaque
description: Altere a cor de destaque do ARGVUS independentemente do tema selecionado e volte ao padrão do tema.
---

A **cor de destaque** é o acento usado nas bordas de foco, na seleção e nos estados ativos em todo o desktop. Cada tema tem uma cor de destaque padrão, mas você pode substituí-la por qualquer cor, sem trocar de tema.

## Alterar a cor de destaque

1. Abra **Control Center → Aparência → Cor de destaque**.
2. Escolha **Editar cor de destaque**. O valor atual aparece ao lado da opção.
3. Digite uma cor RGB de seis dígitos, com ou sem o `#` inicial. Por exemplo, `3590BD` e `#3590BD` são aceitos.
4. Pressione `Enter` para aplicar.

Você também pode colar uma cor no campo. O ARGVUS aceita o texto colado somente quando ele é um valor RGB válido de seis dígitos. Formas abreviadas, como `#123`, e valores com caracteres que não são hexadecimais são rejeitados com uma mensagem de erro, e a cor não é alterada.

A nova cor é aplicada ao desktop imediatamente e salva na sua configuração do ARGVUS.

## Voltar ao padrão do tema

Para voltar à cor definida pelo tema selecionado, abra **Control Center → Aparência → Cor de destaque** e escolha **Restaurar padrão do tema**.

## Como a cor de destaque interage com os temas

- A cor de destaque que você escolhe é uma substituição independente. Ela sobrevive quando você troca para outro tema, então a cor escolhida continua a mesma.
- Um tema aplicado depois de **Restaurar padrão do tema** volta a controlar a cor de destaque e usa o padrão dele.
- Exportar e importar mantêm a cor de destaque ativa no momento da exportação. Veja [Importando e exportando temas](/pt/docs/components/argvus-themes/import-and-export/).

## Onde o valor é guardado

A cor de destaque faz parte da sua configuração do ARGVUS, em `$XDG_CONFIG_HOME/argvus/config.json` (normalmente `~/.config/argvus/config.json`). Não edite os arquivos gerados em `$XDG_CONFIG_HOME/argvus/data/generated`: o ARGVUS os reescreve a partir da configuração.
