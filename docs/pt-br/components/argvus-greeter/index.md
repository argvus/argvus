---
title: Greeter
description: O greeter de login que o greetd inicia, distribuído no pacote argvus-greeter.
---

`argvus-greeter` é o greeter de login do greetd. É uma interface de terminal, operada
pelo teclado, que descobre as sessões Wayland instaladas, mostra as contas com os
avatares e aplica o tema ARGVUS ativo para a conta selecionada.

É o terceiro pacote separado do pacote mestre `argvus`. O código-fonte fica no diretório
`greeter/` do repositório `argvus`, como workspace Cargo próprio, e o pacote é gerado
junto com `argvus` e `argvus-gtk`. Instalar `argvus` não exige ordem de instalação.

O greeter não autentica usuários. A autenticação é feita pelo greetd através do PAM. O
lançador abre o greeter com `argvus tui-terminal`, então o greeter não precisa de um
programa de terminal separado.

Os arquivos de configuração do greeter são instalados em `/etc/argvus`. Use
`argvus greeter-setup --help` para ver as opções de configuração que o pacote oferece.
Para o fluxo de login, consulte [Sessões](/docs/user-guide/sessions/).
