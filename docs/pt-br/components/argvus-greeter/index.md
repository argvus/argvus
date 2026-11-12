---
title: Saudação
description: O greeter de login que o greetd inicia, distribuído no pacote argvus.
---

`argvus-greeter` é o greeter de login do greetd. É uma interface de terminal, operada
pelo teclado, que descobre as sessões Wayland instaladas, mostra as contas com os
avatares e aplica o tema ARGVUS ativo para a conta selecionada.

É parte do pacote `argvus`, o único pacote do desktop. O código-fonte fica em
`crates/argvus-greeter` do repositório `argvus`, como membro do workspace Cargo raiz, e é
compilado junto com o restante do `argvus`.

O greeter não autentica usuários. A autenticação é feita pelo greetd através do PAM. O
lançador abre o greeter com `argvus tui-terminal`, então o greeter não precisa de um
programa de terminal separado.

Os arquivos de configuração do greeter são instalados em `/etc/argvus`. Use
`argvus greeter-setup --help` para ver as opções de configuração que o pacote oferece.
Para o fluxo de login, consulte [Sessões](/docs/user-guide/sessions/).

O socket de handoff do login autoriza cada cliente pelas credenciais fornecidas pelo kernel. Apenas root, o dono do helper e a conta autenticada pelo greetd podem enviar comandos de controle. Outras contas locais são recusadas, embora o socket permita conexões entre usuários. Um quadro DRM preparado permanece vinculado ao descritor original; o helper aguarda esse descritor antes de assumir a tela.

Quando o campo de autenticação está focado, caracteres imprimíveis (incluindo `j` e `k`) entram na resposta. Tab e Shift-Tab movem o foco; Backspace remove o último caractere. Isso se aplica a prompts secretos e não secretos.

A lista de contas rola para manter a conta selecionada visível durante a navegação e o redimensionamento do terminal.
