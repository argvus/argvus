---
title: Greeter
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
