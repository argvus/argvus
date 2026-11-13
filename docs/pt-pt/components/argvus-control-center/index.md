---
title: Centro de Controlo
description: Configure as partes persistentes do seu desktop ARGVUS.
---

O Control Center é a aplicação de definições do ARGVUS, com navegação orientada ao teclado. Use-o quando quiser configurar o desktop, e não apenas executar uma ação pontual.

## Control Center e Control Panel

| Use… | Para… |
| --- | --- |
| **Control Center** | Temas, wallpapers, efeitos, layout, comportamento da taskbar, fontes, aplicações padrão, teclado e entrada, idioma, região e configuração do sistema. |
| **Control Panel** | Status atual e ações frequentes, como volume, brilho, rede, notificações, energia e controles da sessão. |

O Control Panel pode oferecer controles rápidos para uma configuração permanente do Control Center. Por exemplo, use o painel para uma ação rápida de aparência ou display, e o Control Center quando quiser alterar a configuração do desktop por trás dela. Veja Control Panel e [Onde configurar as coisas](/pt/docs/user-guide/where-to-configure/).

## Áreas principais

O ecrã inicial atual do Control Center oferece estas áreas:

O ecrã inicial é um catálogo de destinos agrupados em seis cartões. Cada destino abre a própria página diretamente, e a busca o encontra independentemente do cartão.

* **Aparência** — personalize o desktop: Estilo do desktop (tema, modo de aparência, cor de destaque, wallpapers, ícones e fontes), Estilo das aplicações (transparência do terminal e do lançador), Taskbar, Painel de Controle e Widget Telemetry.
* **Desktop e entrada** — configure seu espaço de trabalho: Hyprland (regras de janela, espaços de janela, animações, blur, bordas, sombra e a versão instalada), Displays, Teclado e atalhos e Rato e touchpad.
* **Preferências** — definições pessoais e regionais: Idioma e região (o hub de idioma, fuso horário, formatos e layout do teclado), Data e hora, Aplicações padrão, Energia e Sessão.
* **Hardware e conectividade** — dispositivos e conexões: Visão geral do hardware, Rede e Wi-Fi, Bluetooth, Áudio e Armazenamento.
* **Administração do sistema** — gerencie o sistema operacional: Pacotes, Serviços, Boot, Nome do host, Utilizadores e grupos, Firewall e Notificações (Não perturbe).
* **Definições** — o próprio Control Center, listado depois dos cartões: Idioma (idioma do utilizador) e Estilo (ícones decorativos e transparência). Prima `s` para abri-la.
* **Ferramentas e diagnóstico** — ferramentas de desenvolvimento e manutenção: Boas-vindas ao ARGVUS, Projetos, Snippets (textos salvos para o `argvus snippets`, digitados com `SUPER + ALT + N`), Chaves SSH (pares de chaves em `~/.ssh` gerenciados pelo `argvus keys`), Chaves GPG (o keyring gerenciado pelo `argvus gpg`; o gpg-agent usa o pinentry-gtk), Serviços de desenvolvimento, System Doctor e Sobre o ARGVUS.

Quando o Não Perturbe é ativado, os pop-ups de notificação são fechados e as novas notificações ficam guardadas para depois. O Control Panel oculta o histórico até o Não Perturbe ser desativado.

## Boas-vindas ao ARGVUS

Abra **Boas-vindas ao ARGVUS** em **Tools & Diagnostics**, busque `welcome` ou execute `argvus welcome`. O Welcome é uma página integrada à Central de Controle e usa sua única janela. Enter abre a configuração selecionada; ao terminar, volte com Esc ao guia. F6 Próximo marca a etapa com `[x]` e abre a próxima configuração na mesma janela. Esc apenas volta e Enter na lista sempre abre a página selecionada. O revisa a etapa novamente, S pula sem marcar como concluída e F finaliza o guia, inclusive com itens opcionais pendentes. Cancelar ou voltar não marca a configuração como concluída. O progresso anterior é preservado.

O assistente cobre idioma e região, monitores, hardware e drivers, rede, áudio, Bluetooth, aparência, energia, aplicações padrão e arranque da sessão. Ele não instala drivers ou pacotes e não aplica alterações do sistema por conta própria. O progresso fica em `$XDG_STATE_HOME/argvus/welcome/progress.json` (ou `~/.local/state/argvus/welcome/progress.json`).

Depois que o desktop está pronto, a sessão executa `argvus welcome --if-needed`, seja o login por TTY, ARGVUS Greeter ou outro gestor. O comando abre o Welcome quando o estado por utilizador ainda não registra `opened`, conclusão ou dispensa. A primeira renderização bem-sucedida grava `opened` em `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/welcome/progress.json`; os próximos logins não reabrem o ecrã. Não depende de `/etc/skel`. Contas existentes sem esse registro também recebem uma abertura automática. Falhas anteriores à renderização permitem tentar novamente no próximo login. A abertura manual com `argvus welcome` continua disponível.

As páginas exatas dos providers podem depender dos pacotes instalados e das permissões. A ausência de um domínio opcional não significa necessariamente que o Control Center esteja quebrado.

Quando a grade tem mais itens do que cabem no ecrã, uma barra de rolagem aparece na borda direita para indicar que há mais conteúdo abaixo.

## Encontrando uma configuração

O ecrã inicial registra definições por título, categoria, identificador e palavras-chave. A busca ignora acentos e aceita termos práticos como `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` e `DPI`. Correspondências exatas de título aparecem primeiro, seguidas por prefixo do título, por qualquer palavra do título ou categoria que contenha o texto digitado e, por fim, por palavras-chave — ou seja, digitar qualquer trecho de uma palavra (não só o início) ainda encontra o destino.

O menu visível reconhece as capacidades disponíveis. Uma rota pode não aparecer quando seu provider não está instalado, quando o hardware não está disponível ou quando a sessão atual não oferece a capacidade necessária. Por isso, a busca também é uma forma de descobrir o que esta instalação pode configurar.

A árvore atual de definições inclui:

| Área | Destinos atuais |
| --- | --- |
| Aparência | Estilo do desktop: tema, modo de aparência, cor de destaque, wallpapers, ícones e fontes; Estilo das aplicações: transparência do terminal e do lançador; Taskbar; Painel de Controle; Widget Telemetry |
| Desktop e entrada | Hyprland (regras de janela, espaços de janela, animações, blur, bordas e sombra), Displays, Teclado e atalhos, Rato e touchpad |
| Preferências | Idioma e região (idioma, fuso horário, formatos, locales do sistema e layout do teclado), Data e hora, Aplicações padrão, Energia, Sessão |
| Hardware e conectividade | Visão geral do hardware (resumo, CPU, GPU, memória, dispositivos), Rede e Wi-Fi, Bluetooth, Áudio, Armazenamento |
| Administração do sistema | Pacotes, Serviços, Boot, Nome do host, Utilizadores e grupos, Firewall, Notificações |
| Definições | Idioma (idioma do utilizador), Estilo (ícones e transparência) |
| Ferramentas e diagnóstico | Projetos, Snippets, Chaves SSH, Chaves GPG, Serviços de desenvolvimento, System Doctor, Sobre o ARGVUS |

Alguns itens são páginas completas e outros são subpáginas alcançadas pela busca ou por um resumo de domínio. A lista exata depende dos providers ARGVUS instalados.

Cada página mostra o seu caminho no cabeçalho, por exemplo `Central de Controle > Hardware e conectividade > Áudio`. O caminho começa no cartão da Home que lista a página, então ela aparece do mesmo jeito pela Home, pela busca e pela CLI. A raiz e os nomes dos cartões são traduzidos; em inglês o cabeçalho mostra `Control Center > Hardware & Connectivity > Audio`.

## Para que serve cada área

### Aparência

Aparência é a principal área de personalização. Seu grupo ambiente de trabalho reúne o tema (com seu modo Sticky ou Spaced), o modo de aparência, a cor de destaque, os wallpapers, o tema de ícones e a página Fontes, e seu grupo Estilo das aplicações reúne a transparência do terminal e do lançador. Taskbar, Painel de Controle e Widget Telemetry aparecem diretamente em Aparência no ecrã inicial e reaproveitam a página de transparência compartilhada da Aparência para suas próprias superfícies. O blur é uma única configuração global, em Hyprland > Blur, e se aplica a todas as superfícies do ARGVUS com blur ativo. Nessa página, o controle Ligar e os valores do Hyprland (Tamanho, Passagens, Brilho, Ruído, Contraste, Vibrância e Escurecimento da vibrância) ficam em rascunho, e nada muda até selecionar **Aplicar**. Animações funciona da mesma forma em Hyprland > Animations (Animações): o controle Ligar, as Curvas personalizadas (Bézier, Spring, Linear e Ease-out) e as cinco categorias de animações formam um único rascunho, aplicado junto com **Aplicar**. Bézier e Spring têm parâmetros editáveis (os pontos da Bézier e a massa, a rigidez e o amortecimento da mola); Linear e Ease-out são curvas fixas, então a página apenas lista seus pontos de controle e não pode ser editada. As opções de estilo seguem o que o Hyprland 0.56.2 desenha: abertura e fechamento de janelas oferecem slide, popin e gnomed; entrada e saída de camada oferecem slide, popin e fade; e as leaves de ambientes de trabalho oferecem slide, fade, slidefade, slidevert e slidefadevert. Esses controles coordenam os componentes de aparência e sessão, em vez de alterar apenas a janela do Control Center.

### Barra de tarefas

A Taskbar configura a posição do painel, o espaçamento ao redor dele e das janelas, a altura, a borda, o grupo de ícones utilitários e os blocos de data/hora, além da própria transparência. Aparece no cartão **Aparência** do ecrã inicial. No modo Sticky, **Espaçamentos > Embaixo** começa em `2`; Spaced mantém sua margem-base de `18` pixels.

**Altura** escolhe 24, 28 (padrão), 32 ou 36 pixels. **Borda** tem um interruptor **Ligar** e a **Espessura** de 1 a 3 pixels, com 1 como padrão. Com a borda desligada, a janela da barra, o grupo da direita e o player MPRIS ficam sem borda. Essas alterações são rascunho até **Aplicar**. A altura e a borda aplicadas se mantêm quando o tema muda; até serem aplicadas, o tema decide os valores.

A página **Ícones** abre **Módulos opcionais**, **Apresentação dos workspaces**, **Lançador** e **Utilitários**. Em Módulos opcionais, cada item pode ser mostrado ou ocultado individualmente: player de áudio, Focus, lançador, rede, perfil de energia, layout do teclado, uso de memória e CPU, temperaturas, data, relógio, bateria, bandeja do sistema, notificações, dispositivos removíveis, gravação, Bluetooth, entrada e saída de áudio e manter acordado. Os workspaces permanecem disponíveis e podem mostrar seus nomes (padrão) ou os ícones das aplicações. O título da janela em foco e as ações Definições e Energia permanecem disponíveis. A visibilidade do lançador é controlada em Módulos opcionais; o submenu Lançador escolhe o ícone personalizado. As alterações são aplicadas ao selecionar **Aplicar**, que reinicia a taskbar.

### Painel de controlo

O Control Panel configura a visibilidade e a ordem dos seus cards de ação rápida, além das próprias sessões e transparência.

A página abre e aceita comandos assim que as definições dos cards são lidas. Os cards que dependem de hardware (Bluetooth e Brilho) são verificados em segundo plano: até essa verificação terminar, eles mantêm a disponibilidade vista por último nesta janela, e a lista se atualiza sozinha quando o resultado chega.

### Telemetria de widget

O Widget Telemetry configura as sessões e a transparência do widget de informações do sistema.

### Fontes

Fontes fica em **Aparência > Estilo do desktop > Fontes**. A página permite selecionar família e tamanho para taskbar, telemetria/informações do sistema, Control Panel, interface do sistema ARGVUS, aplicações, terminal e navegador. Também expõe definições de renderização como antialiasing, hinting, modo subpixel e DPI. A família principal padrão é IBM Plex Mono; cada alvo pode ser alterado independentemente e restaurado pela página Fontes.

### Aplicações padrão

Aplicações padrão seleciona programas instalados para funções como terminal, gestor de ficheiros, editor de texto, editor do terminal, navegador, visualizador de imagens, visualizador de PDF, reprodutor de vídeo, reprodutor de áudio, ferramenta de ficheiros compactados e launcher. A seleção é salva pelo ARGVUS e pode atualizar associações XDG padrão. Gestores de ficheiros TUI usam o editor do terminal selecionado ao abrir ficheiros de texto; as associações de ficheiros do ambiente de trabalho continuam usando o editor gráfico.

### Trechos

Snippets é a página dos textos que o `argvus snippets` digita com `SUPER + ALT + N`. Ela fica em **Ferramentas e diagnóstico → Snippets** (busque por `snippets` ou execute `argvus control-center snippets`).

* **Novo snippet** — Introduza um **Nome** e um **Conteúdo** e depois **Guardar snippet**. Um nome que já existe substitui o conteúdo desse snippet.
* **Snippets salvos** — listados com a posição (`1  email`) e o conteúdo em uma linha. `Enter` sobre o conteúdo o edita. **Digitar** digita o snippet, e **Remover** pede confirmação.
* **Abrir seletor** — abre o mesmo seletor de `SUPER + ALT + N`.

A digitação começa 3 segundos depois de você pressionar Enter, porque neste momento o foco está no próprio Control Center. Clique no campo que deve receber o texto dentro desse tempo. A página edita apenas conteúdo de uma linha; um snippet com várias linhas pode ser criado no terminal com `argvus snippets add`.

A página lê os snippets do `argvus-config` e grava por meio do `argvus snippets`, então a validação continua com o launcher. Quando o `argvus snippets` não está instalado, a página informa isso.

### Chaves SSH

Chaves SSH é a página dos pares de chaves que o `argvus keys` gerencia. Ela fica em **Ferramentas e diagnóstico → Chaves SSH** (busque por `chaves ssh` ou execute `argvus control-center ssh-keys`).

A página tem quatro partes, de cima para baixo:

```
Busca         filtra os pares de chaves pelo nome ou comentário
Gerar         nome, tipo e a ação Gerar par de chaves
Chaves        um bloco por par; aparece uma barra de rolagem quando não cabem
Ações         Atualizar e Descarregar todas as chaves do agente
```

#### Como usar

1. Abra **Control Center → Ferramentas e diagnóstico → Chaves SSH**. Pela Home, você também pode buscar por `chaves ssh`.
2. **Encontrar uma chave:** Prima `/` e Introduza para mostrar só os pares cujo nome ou comentário combina. `Enter` encerra a busca e executa a linha destacada, como nas outras listas, então confira a linha antes de apertar; `Esc` limpa a busca.
3. **Criar um par:** em **Gerar**, selecione **Nome**, Prima `Enter`, Introduza o nome e Prima `Enter` de novo. Selecione **Tipo** e Prima `Enter` para alternar entre `ed25519` e `rsa`. Selecione **Gerar par de chaves** e Prima `Enter`. O novo par aparece em **Chaves** imediatamente, sem palavra-passe.
4. **Copiar a chave pública:** no bloco do par, selecione **Copiar chave pública** e Prima `Enter`. O conteúdo do ficheiro `.pub` vai para a área de transferência do Wayland, pronto para colar no `authorized_keys` de um servidor ou em um host Git. A chave privada nunca é copiada. Isso precisa do `wl-clipboard` (`wl-copy`); sem ele, a página informa que a chave não foi copiada.
5. **Carregar um par no agente:** no bloco do par, selecione **Carregar no agente** e Prima `Enter`. Uma chave sem palavra-passe é carregada na hora, sem janela. Uma chave com palavra-passe abre um terminal ARGVUS, onde o OpenSSH a pede; feche o terminal com `Enter` quando ele indicar que a chave foi carregada. Enquanto a página estiver aberta, ela consulta o agente a cada poucos segundos, então a linha muda sozinha para **Descarregar do agente**. Para atualizar na hora, use **Atualizar** em **Ações**.
6. **Remover uma chave do agente:** no bloco do par, selecione **Descarregar do agente**. Para remover todas, use **Descarregar todas as chaves do agente** em **Ações**. Os ficheiros das chaves continuam no disco.
7. **Apagar um par:** no bloco do par, selecione **Apagar** e Prima `Enter`. A confirmação começa com o foco em **Cancelar**, então um `Enter` acidental não faz nada. Prima `y` para confirmar, ou `n`/`Esc` para cancelar. Uma chave carregada no agente não pode ser apagada; descarregue-a antes.

Para se mover: `↑`/`↓` ou `j`/`k` percorrem a lista, `Enter` ativa a linha, `Esc` volta e `?` mostra a ajuda. `Tab` leva o foco para **Ações**, onde `←`/`→` escolhem um botão e `Enter` o executa. `↑`/`↓` devolvem o foco à lista.

As palavras-passe nunca são digitadas no Control Center nem passadas como argumentos de comando. A página lê a lista com `argvus keys list` e grava somente por meio do `argvus keys`. Quando o `argvus-keys` não está instalado, a página informa isso.

#### Solução de problemas

| Sintoma | Causa e correção |
| --- | --- |
| `Could not open a connection to your authentication agent` após **Carregar no agente** | O terminal não recebeu o socket do agente. O Hyprland mantém o ambiente com que foi iniciado, e o agente pode ter começado depois. Atualize o `argvus-launcher`, que passa a usar `$XDG_RUNTIME_DIR/ssh-agent.socket`, e verifique `systemctl --user status argvus-ssh-agent.service`. |
| **Agente** mostra `Agente indisponível` | O launcher não consegue alcançar o agente. Verifique o serviço como acima. |
| A página diz que o `argvus-keys` não está instalado | Atualize o pacote `argvus`: `argvus keys` é um subcomando dele. |
| **Apagar** mostra um erro | A chave está carregada no agente e foi mantida. Descarregue-a primeiro e apague de novo. |

### Chaves GPG

Chaves GPG é a página do keyring que o `argvus gpg` gerencia. Ela fica em **Ferramentas e diagnóstico → Chaves GPG** (busque por `chaves gpg` ou Execute `argvus control-center gpg-keys`).

As seções da página, de cima para baixo:

```
Pinentry          o pinentry que o gpg-agent usa, só para informação: pinentry-gtk
Gerar chave ▾     nome completo, e-mail, comentário, algoritmo, tamanho, data de
                  expiração, senha, e as ações Gerar chave e Cancelar (expandida no início)
Importar ▾        Importar chave pública… e Importar chave privada… (expandida no início)
Chaves ›          o chaveiro, em uma página própria, com a busca no topo
```

`▾` marca uma seção expandida e `▸` uma recolhida. `Enter` ou `Espaço` expande ou recolhe.

#### Como usar

1. Abra **Control Center → Ferramentas e diagnóstico → Chaves GPG**. Pela Home, você também pode buscar por `chaves gpg`.
2. **Pinentry** é fixo em `pinentry-gtk`. A página apenas o mostra. Na primeira vez que uma chave é gerada, o ARGVUS grava esse pinentry no `gpg-agent.conf`, entre as suas linhas de marcação, e mantém as outras linhas. Se você já escreveu uma linha `pinentry-program` fora dessas marcas, a geração para com erro, para que você remova essa linha.
3. **Criar uma chave:** em **Gerar chave ▾**, Introduza **Nome completo** e **E-mail**. Os dois são obrigatórios, e **Gerar chave** fica desativado até que ambos tenham texto. **Comentário** é opcional. A identidade fica `Nome (comentário) <e-mail>`.
- **Algoritmo** alterna entre **RSA** e **DSA ElGamal**. Uma chave DSA recebe uma subchave de criptografia ElGamal; uma chave RSA recebe uma RSA.
- **Tamanho da chave** percorre os tamanhos do algoritmo: RSA 1024, 2048 e 4096; DSA 1024, 2048 e 3072. O gpg não cria chaves DSA acima de 3072 bits, por isso 4096 não é oferecido para DSA.
- **Data de expiração** abre uma página própria. Introduza a data no formato do seu idioma: `DD/MM/AAAA` em pt-BR, `AAAA/MM/DD` em inglês. A data precisa ser posterior a hoje. Ative **Nunca expira** para tornar a chave permanente; o campo de data fica desativado e não é usado.
- **Palavra-passe** abre uma página própria. **Proteger com palavra-passe** vem ligado por padrão; Introduza a **Palavra-passe** e depois **Confirmar palavra-passe**. As duas precisam ser iguais. Desligar a proteção limpa os dois campos, e a página pede confirmação antes de gerar uma chave sem proteção.
4. Selecione **Gerar chave** e Prima `Enter`. A chave recebe uma chave de assinatura e uma subchave de criptografia. A palavra-passe vai ao launcher pela entrada padrão, nunca como argumento de comando. Se a geração falhar, os outros campos permanecem, mas as palavras-passe são limpas e precisam ser digitadas de novo.
5. **Cancelar** limpa todos os campos do formulário e volta datas e tamanhos aos padrões.
6. **Importar:** em **Importar ▾**, escolha **Importar chave pública…** ou **Importar chave privada…**. O gestor de ficheiros abre para você escolher o ficheiro. Uma chave privada pede confirmação antes; a palavra-passe dela é pedida pelo gpg quando a chave é usada. Se o gestor de ficheiros for gráfico, ele não consegue informar o ficheiro à página; então a página abre um campo onde você digita o caminho.
7. **Chaves ›** abre o chaveiro. Cada chave mostra identidade, fingerprint, tipo, validade e se a chave privada está no chaveiro. Cada chave tem estas ações:
   - **Editar chave ›** abre a página própria da chave (descrita abaixo).
- **Exportar chave pública…** pede o caminho de um ficheiro, sugerido como `~/<fingerprint>.asc`. O ficheiro é criado com modo `600`. Um ficheiro existente nunca é substituído.
- **Exportar chave privada…** aparece só para chaves com parte privada. Pede confirmação e depois o caminho de um ficheiro, sugerido como `~/<fingerprint>-private.asc`. O ficheiro é criado com modo `600`.
- **Copiar chave pública** e **Copiar fingerprint** copiam para a área de transferência Wayland com o `wl-copy`.
- **Excluir chave…** pede confirmação, com o foco em Cancelar. Remove a chave e a parte privada dela.
8. **Encontrar uma chave:** na página **Chaves**, a barra de busca no topo filtra as chaves. Introduza para restringir a lista; a busca compara a identidade (que inclui o comentário) e o fingerprint. `Esc` limpa a busca.
9. **Editar chave ›** mostra os detalhes da chave e depois estas ações:
   - **Data de expiração ›** altera a validade da chave e de cada subchave. Use **Nunca expira** para remover a validade. **Aplicar** salva a alteração.
- **Alterar palavra-passe** pede a palavra-passe antiga e a nova na janela do pinentry.
- **Assinar chave com ›** certifica esta chave com uma das suas outras chaves que tenha parte privada. A palavra-passe do assinante é pedida. A página pede confirmação antes.
- **Revogar chave ›** lista os motivos: sem motivo, chave comprometida, chave substituída, chave não usada mais. Cada um pede confirmação. Uma chave revogada não pode mais ser usada.
- **Confiança** define o quanto você confia na chave: não definida, marginal, total ou máxima.
- **Publicar no servidor de chaves…** envia a chave pública ao servidor configurado no `gpg.conf`, depois de confirmar. **Buscar no servidor de chaves** baixa a chave desse servidor.

Para se mover: `↑`/`↓` ou `j`/`k` percorrem a lista, `Enter` ativa uma linha, `Esc` volta e `?` mostra a ajuda.

As palavras-passe são digitadas apenas nos campos mascarados da página de palavra-passe ou na janela do pinentry. A página envia a palavra-passe da geração ao `argvus gpg` pela entrada padrão, e o launcher a entrega ao gpg por um descritor de ficheiro; assim ela nunca aparece na lista de processos, em argumento de comando ou nos logs. As alterações de chave rodam em segundo plano; enquanto uma roda, as outras esperam. A página lê o keyring com `argvus gpg list` e escreve apenas pelo `argvus gpg`. Quando o `argvus gpg` não está instalado, a página informa isso.

Ainda não está na página: buscar uma chave no servidor por nome ou e-mail. Use **Buscar no servidor de chaves** com uma impressão digital conhecida.

#### Solução de problemas

| Sintoma | Causa e correção |
| --- | --- |
| A página diz `O argvus gpg não está disponível` | O `argvus-launcher` instalado é anterior ao comando GPG. Execute `argvus gpg list` em um terminal: `unknown command` significa que o pacote precisa ser recompilado e instalado. As chaves também precisam do `gnupg`. |
| **Gerar chave** continua desativado | Introduza **Nome completo** e **E-mail**. |
| Uma linha de status diz `argvus-gpg: pinentry-gtk não está instalado` | Instale o pacote `pinentry`, que fornece o `pinentry-gtk`. |
| A geração para com erro de `pinentry-program` | O `gpg-agent.conf` define um pinentry fora do bloco do ARGVUS. Remova essa linha; o ARGVUS grava a sua. |
| **Pinentry** mostra `Não definido` | O gpg-agent usa o padrão embutido até a primeira chave ser gerada pela página. |
| A página diz que o `argvus-launcher` instalado é mais antigo que o Control Center | Reinstale o `argvus-launcher`: Execute `make build` e depois `make install` no repositório dele, e reabra a página. As ações de chave ficam desligadas até as versões baterem. |
| O gestor de ficheiros abre, mas a página pede um caminho | O gestor é gráfico e não consegue informar o ficheiro escolhido. Introduza o caminho no campo. |
| **Copiar chave pública** diz que não conseguiu copiar | O `wl-copy` não está instalado ou a sessão Wayland não está disponível. Instale o `wl-clipboard`. |
| **Editar chave ›** diz que a chave não está mais no chaveiro | O chaveiro mudou depois que a página abriu. Volte para **Chaves ›** e atualize. |
| Uma linha de status mostra `argvus-gpg: ...` | O launcher recusou a ação, e o texto explica o motivo, como uma validade inválida, um ficheiro que já existe ou um `--yes` ausente. |
| Uma ação espera e nada acontece | A janela do pinentry está escondida ou não foi aberta. Confira se a sessão expõe `WAYLAND_DISPLAY` ou `DISPLAY`. |
| A importação é recusada | O ficheiro não contém a chave do tipo escolhido. Chaves públicas vão por **Importar chave pública…**; privadas por **Importar chave privada…**. |

### Idioma e região

Este cartão lista a Língua (o ARGVUS Linguagem de interface), Regional Locale, Zona horária, Data & Hora, Locales do Sistema e Disposição do Teclado. A língua abre o seletor de 18 idiomas; Regional Locale define o locale usado para formatos regionais e aplicações de terceiros.

**Preferências > Idioma & Região > Língua** salva `ARGVUS_LANGUAGE` em `$XDG_CONFIG_HOME/locale.conf` (normalmente `~/.config/locale.conf`), por exemplo `ARGVUS_LANGUAGE=pt-PT`. Muda ARGVUS aplicações e superfícies enquanto preserva `LANG` Para a formatação regional e aplicações de terceiros. O Painel de Controle reinicia imediatamente; inicie uma nova sessão para aplicar o idioma em todos os componentes da sessão. Selecionar um idioma de interface não requer autorização de administrador ou `locale-gen`.

** Local Regional** salva `LANG` no mesmo arquivo e controla os formatos regionais e a linguagem utilizada por aplicativos de terceiros. Sua seleção UTF-8 o locale do sistema já deve ser gerado.

** Localidades do sistema** alterações `/etc/locale.gen` e corre `locale-gen`, então aplicá-lo requer autorização do administrador.

** Regional Locale** salva `LANG` no mesmo arquivo e controla os formatos regionais e o idioma usado por aplicativos de terceiros. Seu local do sistema UTF-8 selecionado já deve ser gerado.

**Locales do sistema** altera `/etc/locale.gen` e executa `locale-gen`, portanto a aplicação exige autorização de administrador. Um aviso permanente **[WARN]** acima do rodapé identifica os ecrãs com definições administrativas. O mesmo aviso abrange fuso horário, data/hora, hostname, mapa de teclado do console, edição de contas e grupos, firewall, serviços do sistema, governador da CPU, perfis e políticas de energia, configuração de boot, operações de pacotes, snapshots e alterações protegidas de rede. Preferências pessoais, serviços do utilizador e páginas somente de leitura não recebem esse aviso. O aviso é informativo; a autorização só é solicitada quando uma ação precisa dela.

### Hardware e displays

Quando os recursos correspondentes estão instalados, o Control Center pode mostrar informações de CPU, GPU, memória, energia e dispositivos. A página de entrada configura o comportamento do rato e touchpad; a área de displays lida com resolução, taxa de atualização, escala, posição, orientação, display principal, VRR e HDR quando suportados pela sessão ativa. As mudanças de display valem na hora. Depois de uma mudança que pode deixar o ecrã inutilizável (resolução ou taxa, posição, espelhamento, desativar um monitor, profundidade de cor) ou depois de **Aplicar**, o Control Center pergunta se deve mantê-la, com o foco em **Reverter**: Prima `y` para manter; `n`, `Esc` ou `Enter` revertem, e sem resposta em 15 segundos a configuração anterior volta automaticamente. Remover a configuração salva de um monitor desconectado e excluir um perfil pedem confirmação; os perfis abrem uma página própria com **Aplicar perfil**, **Renomear** e **Excluir perfil**. O layout é salvo por meio de `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` e `monitor-remove`; por isso a seção canônica `/displays` do `config.json` é a única fonte do layout dos monitores. Só o catálogo de perfis continua em `display/config.toml`.

### Conectividade e áudio

A área de rede expõe o status e as páginas do provider de rede instalado, incluindo rotas de Wi-Fi, Ethernet, VPN, DNS e proxy quando disponíveis. O firewall fica listado em **Administração do sistema**. Bluetooth depende do suporte Bluetooth. A área de áudio aparece quando o recurso opcional correspondente está ativado e integra-se ao serviço de áudio instalado.

O ecrã inicial de **Rede e Wi-Fi** e a página **Status** se preenchem à medida que cada consulta ao NetworkManager responde, em vez de esperar a mais lenta: primeiro a lista de dispositivos, depois o rádio, os endereços e o DNS das interfaces conectadas, as conexões salvas e as redes Wi-Fi que o NetworkManager já conhece (uma nova varredura só é feita na página Wi-Fi, com `r`). Até a lista de dispositivos chegar, as linhas mostram *Carregando...*, e a chave do Wi-Fi só é oferecida quando o computador tem placa Wi-Fi. Iniciar uma ação (a chave do Wi-Fi, conectar, DNS) enquanto a página atualiza substitui a atualização, e a página se atualiza de novo quando a ação termina.

#### Conexões, inspiradas no nmtui

**Rede e Wi-Fi > Conexões** lista todos os perfis de conexão do NetworkManager, agrupados por tipo, com o dispositivo em que estão ativos (`★ Ativa · eno1`), se são porta de uma bridge, bond ou team e se são somente leitura. `Enter` abre o perfil no editor, `a` conecta ou desconecta, `/` filtra pelo nome e `r` atualiza. **Ativar uma conexão** é a mesma lista, em que `Enter` ativa ou desativa um perfil, como no `nmtui`. As páginas de detalhes de uma interface, de uma rede Wi-Fi salva e de uma VPN têm a linha **Editar conexão**, que abre o perfil delas.

**Adicionar conexão** cria perfis Ethernet, Wi-Fi, Hotspot, DSL (PPPoE), Bridge, Bond, Team, VLAN, túnel IP, Loopback e WireGuard, e um perfil de VPN por plugin de VPN do NetworkManager instalado (o OpenVPN tem formulário próprio; os demais plugins editam suas definições como itens `chave=valor`). Também importa um `.conf` do WireGuard ou um ficheiro de plugin, como um `.ovpn` (`nmcli connection import`). Team precisa do pacote `libteam`, DSL precisa do `ppp` e uma VPN precisa do seu plugin (por exemplo `networkmanager-openvpn`); sem eles a linha fica desativada e diz o que instalar. WEP, InfiniBand, MACsec e Open vSwitch não são oferecidos.

O editor é uma lista de seções recolhíveis: **Geral** (nome, interface, conectar automaticamente, prioridade, disponível para todos os utilizadores, conexão limitada), as definições próprias do tipo, **IPv4** e **IPv6** (método, endereços, gateway, servidores DNS, domínios de busca, rotas, ignorar DNS ou rotas automáticas, nunca usar como rota padrão, métrica da rota, extensões de privacidade, geração de endereço). Escolhas e números mudam com `←/→`; `Enter` digita um valor; listas (endereços, DNS, rotas, IPs permitidos) abrem uma página própria, em que `Enter` edita um item, apagar o texto o remove e **Adicionar item** acrescenta outro. Campos que não valem para o método escolhido aparecem desativados. Uma bridge, bond ou team escolhe suas **Portas** Ethernet numa subpágina; o WireGuard edita seus **Peers** numa subpágina e pode **Gerar par de chaves** (com o `wg`, do `wireguard-tools`, dependência do pacote `argvus`); a chave pública da chave privada aparece logo abaixo.

Nada chega ao NetworkManager antes de **Guardar**, que fica desativado enquanto nada mudou ou enquanto algum campo é inválido, e então diz qual campo e por quê. Só as definições alteradas são gravadas. Guardar uma conexão ativa pede confirmação e depois a reconecta, para que as novas definições valham na hora (isso pode interromper a rede por um momento). `Esc` com alterações não salvas pergunta antes de descartá-las. **Excluir conexão**, na Zona de perigo, pede confirmação com o foco em Cancelar; excluir uma bridge, bond ou team também exclui suas portas.

Palavras-passe, chaves privadas e chaves pré-compartilhadas nunca são lidas de volta: as linhas delas dizem *Sem alteração* até um novo valor ser digitado, e o valor novo aparece mascarado. Elas são gravadas pela entrada padrão do `nmcli connection edit`, nunca como argumentos de comando. As alterações rodam como o utilizador da sessão: membros locais do grupo `wheel` são autorizados pela regra de polkit do NetworkManager, e os demais utilizadores recebem o pedido da palavra-passe de administrador pelo agente de polkit da sessão.

### Energia e sessão

Energia oferece os controles de energia do sistema e as páginas de política disponíveis na instalação, incluindo a opção Manter acordado. Quando ativada, Manter acordado impede que a política de ociosidade do ARGVUS inicie os temporizadores automáticos de bloqueio do ecrã e desligamento do display. Sessão expõe status da sessão ARGVUS, componentes, autostart, diagnósticos e logs. Na página **Energia**, a barra **Ações** sob a lista tem **Suspender**, **Hibernar** e **Atualizar** (`r` também atualiza): `Tab` ou `←/→` levam até ela, `Enter` executa o botão focado, `↑`/`↓` devolvem o foco à lista e `Esc` volta. Suspender e Hibernar pedem confirmação antes. Essas páginas podem exigir permissões do sistema e devem ser diferenciadas das ações rápidas do Control Panel.

### Ferramentas do sistema

A seção de sistema pode expor informações de boot, pacotes, serviços, armazenamento, diagnósticos, administração de utilizadores/grupos e informações do sistema. São ferramentas administrativas ou de diagnóstico; uma página pode ser somente leitura ou exigir autorização dependendo da operação.

**Serviços** tem quatro páginas: unidades do sistema, unidades do utilizador, unidades com falha e o journal (Logs). Cada lista de unidades mostra uma linha `Filter:`, que `f` cicla, e uma caixa `Search:` com borda, aberta com `/`. Cada unidade mostra os estados Active e Enabled como bolinhas cheias ou vazias. Em Logs, `b` alterna entre o boot atual e o anterior, `u` cicla a unidade e `p` cicla a prioridade; a linha `Filter:` mostra os valores em uso. `Enter` abre uma unidade ou uma entrada de log.

**Serviços de Dev** reúne três fontes voltadas a desenvolvimento em uma única página de leitura: os serviços systemd do utilizador atual, as portas TCP em escuta (via `ss`) e os containers em execução de qualquer runtime instalado (Podman, Docker). A página só aparece na grade inicial quando pelo menos uma dessas fontes está disponível no sistema — uma máquina sem sessão systemd de utilizador, sem Podman e sem Docker não mostra a entrada. `r` atualiza as três fontes de forma independente, então uma fonte lenta ou com falha (por exemplo, um runtime de containers que parou de responder) não impede as outras de mostrar seus dados.

**Pacotes** é agrupado pela origem. **Repositórios oficiais** e, quando um auxiliar do AUR (`paru` ou `yay`) está instalado, **AUR (Arch User Repository)** abrem **Instalar**, **Remover** e **Atualizar**; depois vêm **Cache**, **Histórico**, **Downgrade** e **Mirrors**. As três tarefas usam a mesma ecrã: a caixa **Buscar** fixa no topo recebe as letras digitadas, e os pacotes ficam embaixo, no bloco **Resultados** (**Atualizações** em Atualizar), onde a lista vazia diz o motivo (por exemplo **Nenhum resultado**, **Nenhum item encontrado** ou **Nenhuma atualização pendente**). **Instalar** busca nos repositórios (`pacman -Ss`) ou no AUR quando você pressiona `Enter` na barra Buscar do topo; **Remover** lista os pacotes instalados (a lista do AUR mostra os pacotes construídos fora dos repositórios, `pacman -Qm`, e enquanto o menu AUR aparece eles saem da lista oficial); **Atualizar** lista as atualizações pendentes (`pacman -Qu`, ou `paru -Qua`/`yay -Qua` no AUR). Num pacote, `Enter` executa a tarefa do ecrã e `→` abre a página do pacote, com **Instalar** (ou **Reinstalar**) e, na **Zona de perigo**, **Remover**; `←` ou `Esc` voltam. **Repositórios oficiais → Remover** começa com **Órfãos**, onde `Space` marca pacotes (`[x]`), `Enter` abre um deles e **Remover marcados** remove todos juntos. Em Atualizar, `Tab` leva à barra **Ações** embaixo da lista (**Atualizar tudo** e **Atualizar banco**, ou **Atualizar tudo (AUR)** no AUR), `←`/`→` andam entre os botões, `Enter` executa o botão com foco e `↑`/`↓` (ou digitar) devolvem o foco à lista; **Atualizar tudo** fica apagado e é pulado enquanto não há atualização listada. O breadcrumb e a busca global seguem o mesmo caminho, por exemplo **Pacotes > Repositórios oficiais > Remover > Órfãos**. As limpezas de cache ficam na Zona de perigo da página Cache, e **Mirrors → Configurar mirrors** define as opções do reflector antes de **Gerar preview**. Toda operação de pacotes mostra o plano, ou no AUR o aviso sobre o PKGBUILD, e pede confirmação (`y` confirma, `n` ou `Esc` cancela) antes de rodar, e a saída aparece enquanto ela roda. As operações do AUR executam o auxiliar com o seu utilizador (`-S <pacote>` para instalar ou atualizar um, `-Sua` para atualizar todos); a remoção de pacotes do AUR usa o mesmo `pacman -Rns` dos pacotes oficiais. Quando outros pacotes instalados dependem do pacote que você remove (oficial ou do AUR), a confirmação lista esses pacotes e oferece dois caminhos: **Remover só** o pacote selecionado (`pacman -Rdd`, que ignora a checagem de dependências, então os pacotes que precisam dele podem parar de funcionar) ou **Remover com os dependentes** (`pacman -Rcns`, com a quantidade de pacotes removidos). O foco começa em **Cancelar**, que mantém todos os pacotes; `Enter` executa a linha com foco e não há atalho `y` nessa escolha.

A barra **Buscar** do topo é a fonte da consulta de pacotes. `/` coloca o foco nela sem apagar o texto; digitar também transfere o foco para a barra. Edite com as setas, `Home`, `End`, `Backspace`, `Delete` ou colagem. Em **Instalar**, `Enter` envia uma consulta aos repositórios/AUR mesmo com resultados anteriores e transfere o foco aos resultados; a busca oficial exige uma consulta não vazia e o AUR exige pelo menos dois caracteres. Editar a consulta de instalação cancela a requisição pendente e limpa os resultados antigos; uma nova consulta aguarda `Enter`. Em **Remover**, **Atualizar** e **Órfãos**, a edição filtra a lista carregada imediatamente, sem outra consulta ao repositório. `↓` vai da barra aos resultados, `↑` no primeiro resultado volta à barra e `Esc` na busca retorna à lista preservando a consulta. `Enter` num resultado executa a ação daquele pacote. A barra de ações de Atualizar continua acessível por `Tab`, e `/` retorna dela à busca.

Todas as operações de **Pacotes** abrem um terminal interativo integrado: pacotes oficiais, AUR, limpeza de cache, mirrors e alterações de repositórios/chaves. Responda às perguntas do programa digitando e pressionando `Enter`; nesse terminal o pacman executa sem `--noconfirm`. `Esc` oculta o ecrã e mantém o processo em execução. `F8` reabre a mesma sessão em qualquer página de Pacotes; **Mostrar terminal** também aparece nos seus menus. O rodapé informa se a transação está em execução, concluída ou com falha. Após um comando concluído com sucesso, `Done!` aparece dentro do terminal depois da última saída, inclusive se o terminal estava oculto. `Shift+PageUp` / `Shift+PageDown` percorrem o histórico, `Shift+End` volta à saída atual e `Ctrl+C` envia uma interrupção ao processo. Com o terminal em foco, `q` e `?` são entrada do programa, não atalhos globais. A saída, as cores ANSI e o cursor são preservados, e o redimensionamento atualiza o terminal do processo. O histórico mantém até 10.000 linhas até a próxima operação ou o encerramento da aplicação. Somente uma transação executa por vez; ocultar o terminal não permite encerrar a aplicação durante a execução. Nenhum shell livre é aberto.

A prévia da atualização consulta as bases existentes (`pacman -Su --print`), sem alterá-las nem exigir root. Após a confirmação, a atualização completa executa `pacman -Syyu` pelo helper privilegiado com `pkexec`; a sincronização pode encontrar atualizações adicionais. Instalação e remoção de pacotes, atualização das bases, limpeza de cache, downgrade, mudanças em repositórios/chaves e gravação de mirrors também usam esse helper privilegiado. Os helpers AUR continuam compilando como o seu utilizador.

**Repositórios oficiais → Repositórios** adiciona e gerencia repositórios não oficiais do pacman. **Adicionar** é um rascunho: escolha a **Origem** da configuração (**URL** de um ficheiro `.conf`, **Texto** digitado ou colado num editor de várias linhas, ou **Formulário** com **Nome**, **Servidor** e **SigLevel**, gravado no formato do Arch Wiki) e, em **Chave de assinatura**, uma **Chave** por **URL** ou **Texto** (ou **Nenhuma**). As escolhas mudam com `←`/`→` ou `Enter`; os campos de texto abrem um editor em que `Enter` salva uma linha, `Ctrl+S` salva várias linhas, colar funciona e `Esc` cancela. Os downloads só aceitam endereços `https://`. **Revisar e adicionar** baixa e confere tudo com o seu utilizador e então pede confirmação, listando os repositórios, servidores e SigLevel, o ID de utilizador da chave e a impressão digital em grupos de quatro, para comparar com a que o repositório publica. Confirmar importa a chave e a assina localmente (`pacman-key --add`, `--lsign-key`), grava `/etc/pacman.d/<nome>.conf` (com o nome da primeira seção) e acrescenta `Include = /etc/pacman.d/<nome>.conf` ao `/etc/pacman.conf` (uma linha comentada do mesmo ficheiro é reativada no lugar). Em seguida o Control Center oferece `pacman -Syyu`, para sincronizar os bancos novos sem atualização parcial. Sair de **Adicionar** com um rascunho pede confirmação antes de descartá-lo. **Listar** mostra todos os repositórios não oficiais, escritos no próprio `pacman.conf` ou incluídos de `/etc/pacman.d/`, ativos ou desativados; as seções oficiais nunca aparecem. A página de um repositório mostra ficheiro, SigLevel, servidores e chave registrada, a alternância **Ativado** e, na **Zona de perigo**, **Remover**. Desativar comenta a linha `Include` ou prefixa cada linha de uma seção inline com `#argvus-off `; remover apaga a linha ou a seção e o ficheiro incluído. Repositórios do mesmo ficheiro incluído são ativados, desativados e removidos juntos. Quando o Control Center importou a chave, **Remover** também oferece retirá-la do chaveiro do pacman (`pacman-key --delete`). Toda mudança no `/etc/pacman.conf` deixa uma cópia de segurança ao lado (`pacman.conf.argvus-backup-…`), e nenhum pacote instalado é removido.

### Utilizadores e grupos

Quando o provider de contas está disponível, abra **Control Center → Administração do sistema → Utilizadores e grupos → Utilizadores** ou busque por **utilizadores**. A interface pode listar contas normais e de sistema, criar utilizadores, editar metadados, gerir grupos suplementares e primários, alterar ou bloquear uma palavra-passe, desbloqueá-la, exigir troca de palavra-passe no próximo login, definir ou remover um avatar, e ativar o login automático.

A criação pede username, nome completo, shell, grupos suplementares e uma palavra-passe opcional com confirmação. Uma palavra-passe vazia cria a conta com a palavra-passe bloqueada; não cria silenciosamente uma palavra-passe utilizável. A página de um utilizador agrupa os campos da conta (nome completo, shell, grupo principal e grupos suplementares, seguidos de **Guardar alterações**), o alternador de **Login automático**, as ações de **Palavra-passe** (alterar, bloquear, desbloquear, exigir troca no próximo login) e as de **Avatar**. Username, UID / GID e diretório home aparecem só como informação. A **Zona de perigo** no fim reúne as ações destrutivas separadas para remover a conta mantendo o diretório home ou removendo também o home. Leia atentamente a confirmação antes de escolher a segunda opção. Operações de conta exigem privilégio e podem abrir o prompt de autorização do sistema. Avatares podem ser consumidos pelo greeter por meio da integração padrão de imagem da conta.

O **Login automático** inicia a sessão da conta selecionada no próximo boot sem mostrar o ecrã de login, a mesma convenção usada por GNOME, LightDM e SDDM para esse recurso: nenhuma palavra-passe é armazenada em lugar nenhum. Ativá-lo para uma conta desativa automaticamente qualquer conta configurada anteriormente (apenas uma conta pode ter login automático por vez). Ele vale apenas para o próximo boot; depois de um logout manual, o ecrã de login normal volta a aparecer pelo resto daquela sessão. O alternador é aplicado imediatamente, com uma confirmação, e está disponível apenas para administradores.

Os grupos possuem suas próprias páginas de lista e edição, incluindo gerenciamento de membros e exclusão de grupo. O Control Center não substitui as políticas de contas do sistema: contas protegidas e operações que exigem autorização continuam sujeitas às restrições do sistema operacional.

### Definições do Control Center

**Definições** é o último item do ecrã inicial. A página **Idioma** define o idioma do utilizador por locale.conf, e a página **Estilo** controla os ícones decorativos e a transparência do Control Center. O blur não fica em Estilo: ele é uma configuração global do Hyprland, em Hyprland > Blur. A página Estilo altera a apresentação da aplicação de definições; ela não desativa serviços ARGVUS nem remove providers. Guardar os ícones pode exigir autorização, enquanto a alteração visual é aplicada à aplicação atual. `Esc` em Estilo volta à página Definições, e `Esc` nela volta à ecrã inicial.

### Sobre o ARGVUS

**Sobre o ARGVUS** fica em **Ferramentas e diagnóstico → Sobre o ARGVUS** (busque `sobre`, `hardware` ou `firmware`, ou execute `argvus control-center about`). A página tem cinco abas, trocadas com `Tab`/`Shift+Tab`, `←`/`→` ou `h`/`l`: **Sistema e hardware**, **Sobre**, **Donate**, **Créditos** e **Direitos autorais**. O cabeçalho mostra o caminho completo, por exemplo `Central de Controle > Ferramentas e diagnóstico > Sobre o ARGVUS > Sistema e hardware`.

**Sistema e hardware** é somente leitura e agrupa os dados desta máquina em três seções, com os valores alinhados numa coluna:

| Seção | Campos |
| --- | --- |
| Boot | Firmware (fabricante, versão e data), Tipo (UEFI ou Legado (BIOS)), Secure Boot (só em UEFI), Virtualização, Bootloader |
| Sistema | ARGVUS, Hostname, Sistema operacional, Distribuidor, Kernel, Arquitetura, Gestor de janelas (Hyprland), Compositor (Wayland), Driver de GPU, GTK, Tema (com Escuro ou Claro), Monitores (uma linha por monitor), Áudio (PipeWire), Shell (Quickshell), Login (greetd), Init (systemd), Pacotes, Tempo ligado, Locale |
| Hardware | Bateria (uma linha por bateria), CPU (com a frequência máxima), Threads, Núcleos, GPU, Memória (usada / total), Swap, Armazenamento, Modelo (só quando difere da placa-mãe), Placa-mãe |

Todos os valores são lidos do sistema em execução, sem privilégios, e a página funciona do mesmo jeito em qualquer máquina. Um valor que não pode ser lido aparece como `N/D`, e uma linha que não se aplica é omitida:

- **Firmware**, **Modelo** e **Placa-mãe** vêm do DMI (`/sys/class/dmi/id`); valores de preenchimento como `To be filled by O.E.M.` são ignorados. **Tipo** e **Secure Boot** vêm de `/sys/firmware/efi`, e **Virtualização** do `systemd-detect-virt`.
- **Bootloader** é lido da variável EFI `LoaderInfo`, publicada pelo systemd-boot, pelo Limine e por outros bootloaders que seguem a Boot Loader Interface. O GRUB não a publica, então é reconhecido por `/boot/grub` (ou `/efi/grub`) e mostrado com a versão do pacote `grub`.
- As versões do ARGVUS, Wayland, Mesa, PipeWire, Quickshell, greetd, systemd e GRUB vêm de uma única consulta `pacman -Q` e aparecem sem a época e sem o release do pacote (`1:25.2.3-1` vira `25.2.3`). A versão do Hyprland vem do `hyprctl version`, ou do pacote quando nenhuma sessão responde.
- **Driver de GPU** mostra o driver de cada placa DRM em `/sys/class/drm`. Um driver proprietário mostra a versão do módulo (por exemplo `nvidia 580.82.07`); um driver do kernel mostra a versão do Mesa (por exemplo `amdgpu (Mesa 25.2.3)`).
- **Monitores** lista os monitores ativos pelo `hyprctl monitors -j`: conector, resolução, taxa de atualização e escala (a escala é omitida quando é 1).
- **Pacotes** conta todos os pacotes instalados, os instalados explicitamente e os externos (AUR ou construídos localmente), pelo `pacman -Qq`, `-Qqe` e `-Qqm`.
- **Tempo ligado** vem de `/proc/uptime` e é o valor do momento em que a página foi aberta. **Locale** vem de `LANG` em `/etc/locale.conf`.
- **Bateria** lista as baterias do computador em `/sys/class/power_supply`, escolhidas pelo tipo, então periféricos como mouses sem fio não entram. Cada linha mostra a carga, o estado, a saúde (capacidade atual comparada com a de fábrica) e os ciclos de carga quando o firmware os informa. Uma máquina sem bateria mostra **Não detectada**.
- **CPU** acrescenta a maior frequência máxima entre todos os núcleos (cpufreq), que é a máxima nominal em processadores híbridos; ela é omitida onde o cpufreq não existe, como na maioria das máquinas virtuais. **Threads** e **Núcleos** vêm de `/proc/cpuinfo`.
- **Memória** e **Swap** vêm de `/proc/meminfo` (zram incluído); um sistema sem swap mostra **Desativado**.
- **Armazenamento** descreve o disco sob `/`: modelo, tipo (NVMe, SSD, HDD ou eMMC/SD; discos virtuais não mostram tipo), sistema de ficheiros e espaço usado e total. O disco é encontrado atravessando partições, LUKS e LVM até o dispositivo físico.

Os valores são lidos em segundo plano na primeira vez que a página é desenhada, então nem a abertura do Control Center nem a da página esperam por eles; enquanto isso, a aba mostra **Carregando informações do sistema...**. Prima `c` nesta aba para copiar o relatório inteiro como texto para a área de transferência (com `wl-copy`), por exemplo para colar num relato de bug.

## Busca e recursos opcionais

O ecrã inicial possui um registro pesquisável de rotas. Os termos incluem conceitos voltados ao utilizador, como aparência, temas, wallpapers, fontes, atalhos de teclado, rato, touchpad, displays, rede e sessão. Providers opcionais só aparecem quando o recurso foi compilado e sua capacidade de runtime foi detectada; por isso o menu visível é consciente das capacidades disponíveis.

Enquanto a busca do ecrã inicial estiver ativa, as teclas imprimíveis são inseridas na consulta, incluindo `j` e `k`; use as setas `↑` e `↓` para percorrer as rotas encontradas. Prima `Enter` para abrir a rota selecionada, `Backspace` para editar a consulta e `Esc` para limpar a consulta ou sair do modo de busca.

## Fluxo de aparência

Abra **Aparência** para ver os controles visuais integrados. Seus grupos são:

* **ambiente de trabalho** — **Tema** (escolha uma família incluída e seu modo Sticky ou Spaced, ou exporte, importe, aplique e exclua temas seus, veja [Importando e exportando temas](/pt/docs/components/argvus-themes/import-and-export/)), **Modo de aparência** (Sticky ou Spaced, independente do tema), **Cor de destaque** (um acento incluído ou uma cor RGB válida de seis dígitos), **Wallpaper** (um wallpaper incluído ou um ficheiro personalizado) **Ícones** (um tema de ícones instalado: pesquise na lista com `/`, escolha um com Espaço ou Enter e selecione **Aplicar**; nada muda antes de Aplicar) e **Fontes** (os alvos de fonte e as definições de renderização, abertos na página Fontes).
* **Estilo das aplicações** — a transparência do **Terminal** e do **Lançador**.

O cartão Aparência reúne as superfícies que usam essas definições: **Widget Telemetry** (ative a superfície de telemetria, selecione seus blocos disponíveis e reordene-os com `Shift+↑`/`Shift+↓` em um bloco focado) e **Control Panel** (ative, desative e reordene os cards do painel com `Shift+↑`/`Shift+↓` em um card focado, aplicado na hora através do mesmo mecanismo que o arrastar do painel real usa).

Essas ações atualizam o estado lógico do ARGVUS e aplicam a configuração de runtime afetada. Veja [Aparência](/pt/docs/user-guide/appearance/), [Temas](/pt/docs/user-guide/appearance/) e [Janelas e layout](/pt/docs/argvus-hyprland/windows-and-layout/).

## Uso pelo teclado

A aplicação de definições suporta navegação pelo teclado. Cada página é uma lista única: use as setas ou `j`/`k` para mover, `Enter` para abrir ou executar a linha selecionada, `Space` para alternar, `Esc` para voltar, `/` para buscar listas e `?` para ajuda; o rodapé mostra as teclas que valem para a linha selecionada. As ações são linhas da lista, não botões, e `Tab` não leva até elas, exceto nas páginas com a barra **Ações** sob a lista (Aplicações padrão, Fuso horário, Layout do teclado, Locales do sistema Chaves SSH e Criar/Editar atalho). Linhas que só mostram informação são puladas. Enquanto um campo de texto, uma busca ou um filtro de lista estiver ativo, `q` e `?` são digitados como texto em vez de sair ou abrir a ajuda. Em **Aplicações padrão**, **Restaurar padrões** é o botão da barra **Ações** sob a lista: `Tab` ou `←/→` levam até ele e `Enter` pede a restauração. **Restaurar padrões** nas outras páginas e **Restaurar todos os atalhos** são linhas de uma **Zona de perigo** no fim das suas páginas; `r` pede a mesma restauração, e na lista de atalhos `r` restaura o atalho selecionado depois de uma confirmação. Em **Idioma e região → Layout do teclado**, `/` foca a caixa **Buscar** no topo da página, e `Esc` a limpa; O `●` marca o layout em uso. Ele acompanha a barra de tarefas e o atalho de teclado que trocam os layouts, e a página se atualiza em cerca de um segundo sem gravar nada na configuração. `Enter` define o layout padrão, o primeiro que o Hyprland usa ao iniciar e que aparece com `Padrão` no detalhe; `Space` adiciona ou remove um layout (`[x]`). Essas teclas só alteram um rascunho, e o botão **Aplicar** na barra **Ações**, fixa embaixo da lista, grava os layouts e o padrão no Hyprland de uma vez. Na lista, `Tab` ou `←/→` levam o foco até essa barra enquanto **Aplicar** estiver ativado (o rodapé mostra então `←/→ Ações`); `←/→` percorrem os botões dela e `↑/↓` devolvem o foco à lista. Sair da página com alterações não aplicadas pede confirmação antes. Em **Idioma e região → Fuso horário** o layout é o mesmo: `/` foca a caixa **Buscar** no topo, `Enter` escolhe um fuso em um rascunho (marcado com `●`), e o botão **Aplicar** na barra **Ações**, embaixo da lista, salva o fuso no `environment.d/20-timezone.conf` do utilizador; `Tab`, `←/→` e `↑/↓` funcionam como em Layout do teclado. Sair com um fuso não aplicado pede confirmação antes. Em **Locales do sistema** o layout é o mesmo do Fuso horário: `/` foca a caixa **Buscar** no topo, `Enter` ou `Space` marcam ou desmarcam um locale (`[x]`) em um rascunho, e o botão **Aplicar** na barra **Ações**, embaixo da lista, grava a seleção em `/etc/locale.gen` e roda `locale-gen`. Como isso regenera os locales do sistema, **Aplicar** pede confirmação antes; `Tab`, `←/→` e `↑/↓` funcionam como nos outros dois. Em **Desktop e entrada → Atalhos de teclado** a caixa de busca também fica no topo, e `/` a foca, mas a página não tem barra **Ações**: a lista mantém as seções de atalhos e a **Zona de perigo**, e o rodapé mostra `r Restaurar padrão` somente na própria lista, não em **Editar atalho**.

As páginas que juntam alterações antes de guardar (um utilizador, um grupo, a criação de conta, a troca de palavra-passe e a configuração do firewall) terminam esse bloco com a linha **Guardar** ou **Aplicar**, que fica apagada até haver alguma alteração. Se voltar descartaria alterações não salvas, o Control Center pergunta antes. Toda confirmação mostra **Confirmar** e **Cancelar** como duas linhas, com o foco em **Cancelar**: `Enter` executa a linha em foco, `y` confirma e `n` ou `Esc` cancelam.

Em **Aparência**, cada página é uma lista única: o rodapé mostra só as teclas que valem para a linha selecionada, e `Space` também abre ou seleciona a linha. As páginas que juntam alterações antes de aplicar (Taskbar, Widget Telemetry, Control Panel e os controles de efeito) terminam com a linha **Aplicar**, que fica apagada até haver alguma alteração. Ao selecionar **Aplicar**, o ecrã de carregamento só aparece quando a alteração recria uma superfície: mudar a altura, o grupo de utilitários, os ícones ou o formato de data e hora da taskbar reinicia a taskbar; ligar ou desligar o Widget Telemetry ou mudar os blocos ou a ordem dele reinicia o widget; ligar ou desligar o Control Panel, ou mudar os cards dele enquanto está ligado, reinicia o painel. O ecrã fica até a última alteração da página ser aplicada, então nenhuma mudança aparece antes dela, e a superfície é reiniciada uma vez. Bordas, transparência, blur, animações, espaçamentos, fontes, wallpaper e as demais definições são aplicadas no lugar, sem o ecrã de carregamento. Nas linhas de porcentagem, `←`/`→` (ou `+`/`-`) mudam o valor de 5 em 5 e `Enter` permite digitá-lo; use `Esc` para voltar. Se voltar descartaria alterações não aplicadas, o Control Center pergunta antes. Excluir um tema personalizado (`d`) ou substituir um tema importado pede confirmação; o foco começa em **Cancelar**, `y` confirma e `n` ou `Esc` cancelam.

## Persistência e restauração

As páginas aplicam alterações pelo provider responsável e salvam o estado de utilizador suportado. As mudanças de aparência ficam na configuração canônica, e o `argvus-config` é o único componente que escreve os ficheiros de consumo derivados em `~/.config/argvus/data/generated/`; esses ficheiros gerados não são o local para uma edição permanente. As páginas de entrada e atalhos possuem estado persistente e ações de restauração próprias. Outras alterações do sistema podem exigir permissões ou recarga de serviço.

Não existe uma restauração global para todas as definições do ARGVUS. Restaure uma alteração pela página responsável ou use o procedimento de recuperação documentado para o recurso correspondente.

O suporte à restauração é específico de cada domínio:

* **Aparência** oferece restauração da cor de destaque para o padrão do tema ativo. Não existe uma restauração global de aparência; restaure os demais valores pelos próprios controles.
* **Fontes** pode restaurar todas as fontes, um alvo ou uma configuração individual.
* **Aplicações padrão** pode restaurar todos os padrões, uma categoria ou um seletor individual.
* **Atalhos de teclado** pode restaurar um atalho ou todos e então recarregar os atalhos gerados da sessão.
* **Regras de janela** ficam em **Desktop e entrada → Hyprland → Janelas → Regras** (busque por `regras de janela` ou execute `argvus control-center window-rules`). Cada regra tem a linha **Workspace** (`Enter` ou `←/→` percorrem de 1 a 10) e a linha **Classes de janela**, que abre um campo de texto com expressões regulares separadas por vírgula. **Adicionar regra** cria `rule-N` no workspace 1, sem classes. **Remover regra** de cada regra fica na **Zona de perigo** e pede confirmação. Cada alteração é gravada em `hyprland.window_rules` no `argvus-config` e recarrega a configuração na hora; padrões de classe inválidos são recusados e mostrados como erro.

* **Projetos** ficam em **Ferramentas e diagnóstico → Projetos** (busque por `projetos` ou execute `argvus control-center projects`). **Adicionar pasta de projeto** e **Adicionar pasta raiz** ficam no topo da página e abrem um campo de texto com o caminho; o launcher confere se é um diretório e mostra o erro caso não seja, inclusive quando a pasta levaria o total efetivo de projetos além de 9 (o `SUPER + CTRL + 1..9` não consegue endereçar mais que isso). Cada pasta configurada, marcada como raiz (seus subdiretórios são projetos) ou projeto único, ganha seu próprio bloco abaixo do formulário com uma linha **Remover** que pede confirmação. A página chama o `argvus-projects`, então as alterações são gravadas pelo launcher e valem na próxima vez que `SUPER + O` ou `SUPER + CTRL + 1..9` forem usados. Se o `argvus-projects` não estiver instalado, a página informa isso.
* **Outras páginas do sistema** exibem ações de restauração, aplicação, exclusão ou recuperação somente quando o provider responsável oferece esse recurso.

Remover uma preferência do utilizador pode fazer um provider retornar ao estado padrão, mas apagar ficheiros manualmente não é um procedimento geral de recuperação. Prefira a ação de restauração da página ou o comando documentado para o recurso.

## Entradas pela CLI

O auxiliar instalado é `argvus-control-center`, executado por `argvus control-center`. Ele pode abrir áreas específicas diretamente, por exemplo:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```

Execute `argvus control-center --help` para ver as rotas disponíveis na versão instalada. A rota pela linha de comando é um atalho para o mesmo aplicação, não um segundo sistema de configuração.

Rotas diretas úteis incluem `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network` (com as páginas `status`, `connections`, `activate`, `add`, `interfaces`, `wifi`, `ethernet`, `vpn`, `dns` e `proxy`), `audio`, `bluetooth`, `boot`, `packages` (com as páginas `official`, `search`, `installed`, `updates`, `orphans`, `aur`, `aur-remove`, `aur-updates`, `repositories`, `repositories-add`, `repositories-list`, `cache`, `history`, `downgrade` e `mirrors`), `storage`, `system-doctor` (com aliases `diagnostics` e `diagnostic`), `about` (com as abas `system`, `about`, `donate`, `credits` e `copyright`), `power`, `session`, `displays` e `appearance`. Appearance também aceita rotas específicas como `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` e `widget-telemetry`; displays aceita `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` e `hdr`. Use a saída `--help` instalada ao criar scripts, pois a disponibilidade ainda depende do build instalado.

## Relacionados

* [Primeira configuração](/pt/docs/getting-started/)
* Control Panel
* [Aparência](/pt/docs/user-guide/appearance/)
* [Atalhos de teclado](/pt/docs/argvus-hyprland/keyboard-shortcuts/)
* [Rato e touchpad](/pt/docs/argvus-hyprland/input/)

### Médico do sistema

Abra **Ferramentas e diagnóstico → System Doctor**, procure pelo nome ou execute `argvus control-center system-doctor`. As rotas anteriores `diagnostics` e `diagnostic` e seus seletores de categoria permanecem disponíveis. Use `r` para atualizar, Enter para abrir uma verificação, `o` nos detalhes para abrir a página relacionada e Esc para voltar. A coleta ocorre em segundo plano e é somente leitura.

As verificações usam contagens de unidades systemd com falha, ferramentas gráficas e o snapshot de Storage para uso dos sistemas de ficheiros e saúde SMART. Um problema exige evidência; ferramentas ausentes ou dados SMART inacessíveis aparecem como indisponíveis, e uma sondagem que falhou ou excedeu o tempo limite aparece como erro. Categorias sem evidência coletada não indicam sucesso. As atualizações usam o banco local do pacman sem sincronizar repositórios; um ficheiro de lock pode ser antigo e não comprova que há uma operação em andamento. SMART pode exigir permissões e não solicita acesso elevado. O resumo registra a última coleta concluída em tempo Unix.

Storage e System Doctor compartilham um snapshot em memória por até 10 segundos. A tecla `r` solicita uma nova coleta; solicitações simultâneas reutilizam uma coleta recém-concluída. A saída dos subprocessos de Storage e do diagnóstico é limitada a 1 MiB por fluxo; uma saída maior é tratada como falha de coleta.

## Luz noturna

Abra **ambiente de trabalho e entrada > Monitores > Luz noturna** ou execute `argvus control-center displays night-light`. Configure temperatura, ativação manual e horários locais por `argvus config display night-light`. Veja [Luz noturna](/pt/docs/user-guide/night-light/) para comandos, estado e requisitos de runtime.

## Instantâneo

Para comparar snapshots, selecione a base e Prima **b**; selecione outro
snapshot como destino e Prima **v**. A página mostra os IDs ordenados, as
quantidades de caminhos criados, removidos e modificados e os seis indicadores
do `snapper status`. Selecione um caminho e Prima **Enter** para consultar
o diff textual; **Esc** volta aos caminhos e depois à lista de snapshots.
O primeiro indicador representa criação, remoção ou alteração de conteúdo/tipo;
os seguintes representam permissões, proprietário, grupo, atributos estendidos
e ACL. Um ponto indica ausência de alteração nessa posição. `status` lista
caminhos e `diff` mostra conteúdo. Conteúdo binário ou não UTF-8 não é exibido;
diff maior que 256 KiB é informado como truncado. Linhas de status malformadas
são rejeitadas. O formato do Snapper não representa sem ambiguidade caminhos com
quebra de linha; esses nomes não são suportados. Em caso de
permissão negada, use **a — Autorizar**. Se um snapshot desaparecer ou a
configuração da raiz mudar, atualize a lista. Os testes do backend usam um
`ProcessRunner` falso, sem exigir Btrfs, Snapper ou root.

Na comparação, selecione um caminho e Prima **r** para revisar a restauração
do ficheiro a partir do snapshot de destino. O helper confirma que os snapshots
continuam na configuração da raiz, que o caminho ainda pertence à comparação e
que a origem é um ficheiro regular na visualização padrão do Snapper em
`/.snapshots/<id>/snapshot`. O diff limitado do Snapper aparece quando é textual;
diff binário ou grande é identificado sem carregar todo o conteúdo na interface.
**Enter** abre uma confirmação separada com origem e destino; **Esc** cancela sem
alterar o ficheiro. Ao confirmar, é criado primeiro um snapshot `single` de
segurança; depois o Snapper executa `undochange` somente para esse caminho e a
página verifica que ele corresponde ao snapshot escolhido. O ID de segurança é
informado e o snapshot permanece disponível.
A restauração compartilha o lock das atualizações de pacotes do ARGVUS e não
ocorre ao mesmo tempo que uma transação pacman oficial do Control Center.

A primeira versão aceita somente ficheiros regulares de root, em diretórios de
root sem escrita para grupo ou outros. Diretórios, links simbólicos, hard links,
ficheiros especiais, pontos de montagem/subvolumes, o diretório interno do Snapper
e locais graváveis por utilizadores são recusados. Ficheiros ausentes no snapshot
selecionado nunca são excluídos. O `undochange` na raiz não garante consistência
de aplicações ou bancos de dados e não é atômico; o snapshot de segurança é um
ponto de recuperação, não uma garantia. O Snapper preserva somente os metadados
suportados pela versão instalada. Subvolumes separados e ficheiros fora da raiz
configurada permanecem fora da operação.

Abra **Central de Controle > Administração do sistema > Snapshot**, pesquise
`Snapper` ou execute `argvus control-center snapshot`. A feature Cargo `snapshot`
está em `full` (padrão) e independe de `storage`.
O aviso **Experimental** informa que o recurso ainda está em desenvolvimento e
pode estar incompleto.

A página gerencia somente uma configuração Snapper existente com `SUBVOLUME=/`
e `FSTYPE=btrfs`. A raiz é detectada com `findmnt`; o nome `root` sozinho não
identifica uma configuração válida. Se houver várias definições correspondentes,
é necessário selecionar uma. O pacote Arch `snapper` é opcional e sua instalação
é manual. É necessário suporte a `--jsonout` e às colunas ativo/padrão.
Nenhuma configuração é criada automaticamente.

Em uma raiz que não usa Btrfs, a mensagem é **Este sistema de ficheiros não é Btrfs**,
sem ações de snapshot. Erro de detecção, Snapper ausente, ausência de configuração
da raiz e permissão negada são estados separados. Se a consulta sem privilégios
for negada, Prima **a — Autorizar** para solicitar acesso via Polkit. A TUI
permanece como utilizador comum; somente o helper restrito `system-settings` executa
ações administrativas. Consultas posteriores podem pedir nova autenticação.
É necessário um agente gráfico Polkit disponível na sessão.

Teclas: **↑/↓** selecionar, **Enter** detalhes/configuração, **c** criar, **d**
excluir, **p** políticas de retenção, **r** atualizar, **Esc** voltar e **?** ajuda.
No ecrã de retenção, **↑/↓** selecionam uma política, **e** edita, **c** alterna
algoritmos de limpeza e **Enter** abre a confirmação. A descrição é opcional,
com até 256 caracteres imprimíveis; vazia, usa “Snapshot manual do ARGVUS”.
Criação e exclusão exigem confirmação, inicialmente focada em Cancelar. A exclusão
atinge apenas um snapshot selecionado e bloqueia o número 0, o snapshot ativo e o
padrão. O helper verifica novamente essas condições antes de chamar Snapper.
Após uma operação, a lista é atualizada. Operações interrompidas não são repetidas
automaticamente: confira a lista atualizada antes de tentar novamente.

Os snapshots manuais usam o tipo `single`, somente leitura pelo padrão do Snapper,
e nenhum algoritmo de limpeza é atribuído pela página. Timers e operações externas
do Snapper continuam valendo. Um snapshot não é uma imagem completa: subvolumes
separados, como um `/home` separado, não são incluídos. Esta versão **não oferece
rollback completo, instalação ou criação de configuração Snapper**. Em uma
configuração válida da raiz, Prima **p** para consultar as políticas conhecidas.
Valores ausentes são identificados como valores alternativos do ARGVUS, não como
valores lidos do template de criação instalado; chaves desconhecidas são apenas
diagnósticas. Prima **e** para editar um valor suportado. A confirmação
mostra o valor anterior e o novo; após guardar, a página consulta novamente e
verifica a configuração. Prima **c** para alternar algoritmos de limpeza e
**Enter** para revisar e confirmar. A limpeza usa o algoritmo oficial do Snapper;
não há prévia exata, e a quantidade removida é medida comparando as listas após
a execução. O Snapper 0.13.2 do Arch fornece
`snapper-timeline.timer` e `snapper-cleanup.timer`; a criação por timeline depende
do primeiro, e políticas de limpeza não são executadas sem o agendamento ativo.
A página apenas consulta o estado das unidades e não as habilita. Limites de espaço exigem qgroups Btrfs
configurados e não são expostos nesta ecrã.

### Operações de pacotes e snapshots

Instalação, atualização e remoção de pacotes não exigem Snapper, Btrfs nem uma
configuração de snapshots. **Atualizar todos** executa `pacman -Syyu` pelo helper
privilegiado com `pkexec`; atualizações oficiais individuais executam
`pacman -S <pacote>`. O ARGVUS não cria snapshots automáticos nessas operações.
A página Snapshots permanece independente, com seus próprios requisitos de
Btrfs/Snapper. Hooks configurados separadamente no pacman continuam sob controle
do sistema.

### Validação manual

- Em raiz ext4/xfs, confira a mensagem exata em português e que c/d não abrem operações.
- Em VM Btrfs descartável, verifique Snapper ausente, ausência de configuração da
  raiz e uma configuração chamada `root` que aponta para outro subvolume.
- Com configuração válida para `/`, liste snapshots, veja detalhes, crie um snapshot
  com espaços/caracteres especiais, cancele uma exclusão e exclua apenas o snapshot
  recém-criado, confirmando número e descrição.
- Confira autenticação negada/cancelada, proteção ativo/padrão, atualização,
  teclado, redimensionamento e ambos os idiomas. Preserve snapshots de recuperação.

Somente o pacote `argvus` muda. Execute `make build` em `de/argvus` para gerar o
`.pkg.tar.zst`; a instalação é uma ação separada do operador.

A página Language em Language & Region salva o locale no perfil do utilizador (`~/.config/locale.conf`), sem alterar `/etc/locale.conf`. A sessão usa a preferência após novo login. Durante o Welcome, F6 Próximo ou o botão Próximo no rodapé registra a revisão e avança; Shift+Tab focaliza o botão e Enter o ativa, inclusive em páginas informativas.

Bluetooth só aparece no Welcome quando o Control Center detecta um adaptador. O progresso anterior dessa etapa é preservado quando ela fica oculta.

O idioma e o layout do teclado da sessão são preferências pessoais e não exigem privilégios administrativos. Novos perfis usam inglês e teclado US por padrão; preferências pessoais existentes são preservadas. Data/hora, NTP, geração de locales do sistema e mapa do console (TTY) mantêm o comportamento administrativo global.

Para teclado brasileiro na sessão, selecione `br`, aplique o layout e escolha `abnt2` em Variante. Isso grava a configuração pessoal do Hyprland sem autorização administrativa; o mapa do console é separado. ABNT2 usa teclas de composição: `~` seguido de `a` produz `ã`, e `~` seguido de Espaço produz `~`. Aplicar a variante ativa o layout padrão configurado; falhas de ativação são informadas.

Fuso horário salva TZ em `$XDG_CONFIG_HOME/environment.d/20-timezone.conf` sem autorização. Saia e entre novamente para as aplicações herdarem a preferência. Data e hora continua mostrando e editando o relógio global no fuso do sistema, com autorização administrativa; NTP permanece igual.

Perfis de monitores e associações de workspaces são aplicados pela configuração Lua gerada e um reload do Hyprland. A energia dos monitores usa o dispatcher Lua DPMS suportado pelo Hyprland 0.56.2.

A potência de renderização da sombra é limitada a 1–4 (padrão 4) para compatibilidade com Hyprland 0.57. Valores antigos acima de 4 são limitados ao carregar a configuração.

## Wobble das janelas

Abra **Desktop e entrada → Hyprland → Animações → Janelas** e use os controles
**Wobble**. Busque por `wobble` ou abra a página diretamente:

```sh
argvus control-center hyprland animations windows
```

O Wobble começa desativado. Os padrões do ARGVUS são rigidez `100`, amortecimento
`10`, massa `1`, intensidade `1` e resolução da malha `8`. Rigidez e massa aceitam
`0.0001–1000`, amortecimento `0–1000`, intensidade `0–2` e malha um inteiro `2–32`.
Uma malha maior aumenta o custo de renderização. Use as setas para ajustar valores
ou Enter para digitá-los. **Restaurar padrões do Wobble** altera o rascunho;
**Aplicar** salva todas as edições de animações pendentes e recarrega o Hyprland
uma vez, sem reiniciar a sessão. Voltar dentro de Animações mantém o rascunho
compartilhado.

A chave global de Animações desativa o Wobble sem perder suas definições
individuais. Os valores canônicos ficam em `effects.wobble_*` no estado do ARGVUS;
`hypr/state.lua` é gerado e não deve ser editado manualmente.

O recurso exige Hyprland **0.57.0 ou superior**, garantido pela dependência do
pacote `argvus-hyprland`. A versão 0.56 não é compatível com esta configuração.
O máximo de intensidade segue o [código da versão 0.57.0](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp),
que limita o valor a `2`, embora a documentação mais recente possa indicar `3`.

O Welcome abre com o logo ARGVUS e WELCOME. Prima F6 Next para ver as etapas sem concluir nenhuma etapa.

Instalações novas usam o fuso pessoal `America/New_York` e somente o layout de teclado `us` ativado. Preferências pessoais existentes são preservadas; o fuso global do sistema não é alterado.

As opções de bateria e Bluetooth da taskbar aparecem somente quando o hardware correspondente é detectado. Ocultar Keep awake remove seu indicador sem alterar o bloqueio de suspensão. Utilities oferece um grupo recolhido que expande com o rato. A apresentação de workspaces por ícones mostra as janelas abertas; a bandeja controla os ícones de aplicações em segundo plano.


A página de atalhos inclui **Seletor de agentes de IA (pacote opcional argvus-ai)** com padrão SUPER + A. O identificador app.ai permite remapear e desativar pelo mesmo modelo canônico keyboard_shortcuts dos demais atalhos. O bind Lua executável é fornecido pelo argvus-ai; a entrada do catálogo, sozinha, não ativa o atalho sem o pacote opcional.


Control Center > Appearance > Taskbar > Icons > Optional modules inclui **Agentes de IA**. A opção vem ativada e é persistida em /taskbar/icons/ai_agents_enabled. Altere o rascunho da taskbar e aplique para guardar e regenerar o grupo de utilitários; desabilitá-la não altera SUPER + A nem as entradas do lançador de aplicações.

Os controles de arranque automática editam o ficheiro `.desktop` do utilizador ou criam uma cópia de uma entrada do sistema. Nomes, comandos e opções de ativação vêm de `[Desktop Entry]`; os grupos de ações adicionais permanecem inalterados.

As confirmações de serviços preservam o escopo de utilizador/sistema junto da unidade e da ação. Uma unidade com o mesmo nome em outro escopo não pode redirecionar a ação confirmada; as páginas de detalhes também preservam o escopo original.

Armazenamento inclui pontos de montagem aninhados, como `/home`, `/boot` e `/boot/efi`, no inventário de sistemas de ficheiros. Diagnósticos usa esse mesmo inventário nas verificações de uso da raiz e de `/home`.

Novos snapshots de segurança da restauração usam o algoritmo de limpeza `number` do Snapper. A remoção segue a política de retenção por número quando a limpeza é executada. Snapshots existentes sem algoritmo de limpeza não são alterados, e a criação manual mantém sua política atual.

Comparações de snapshots informam separadamente os registros de caminhos não suportados. Ficheiros válidos continuam disponíveis para revisão; caminhos com codificação inválida, caracteres de controle ou componentes não suportados não oferecem ações de diff ou restauração.

As definições de energia gravam o drop-in do logind do ARGVUS em `[Login]`, criando o diretório quando necessário. Falhas de recarga são informadas. O comportamento da tampa na bateria usa `HandleLidSwitch`; o contexto legado `default` é um alias, enquanto alimentação externa e dock usam chaves próprias do logind.

A aplicação da lista de espelhos aceita apenas comentários, linhas vazias e entradas `Server` HTTP/HTTPS. O conteúdo é limitado a 64 KiB e verificado antes da elevação e novamente pelo helper privilegiado; diretivas do pacman como `SigLevel` e `XferCommand` são rejeitadas.

A limpeza de pacotes desinstalados não mantém versões no cache. Operações de repositório recusam includes compartilhados por várias seções. Hostnames com espaços nas extremidades e seleções vazias de locales para geração são rejeitados. Gravações de definições recusam destinos symlink e ficheiros temporários já existentes.

Buscas e carregamentos de pacotes descartam resultados de uma página ou consulta anterior. A página atual é carregada novamente, evitando que um resultado antigo marque outra visualização como atualizada recentemente.

A saída de transações é sanitizada e limitada. A exibição mantém até 10.000 linhas ou 4 MiB e informa quando omite conteúdo. Fluxos capturados respeitam o limite configurado, com padrão de 8 MiB na execução ao vivo; o excesso continua sendo drenado e é informado após o processo terminar.

O inventário de boot lê o `pkgbase` do kernel Arch para localizar imagens, initramfs e presets do mkinitcpio, exibindo separadamente o release do kernel. Arrays literais do mkinitcpio são lidos sem incluir os parênteses nem executar a configuração.

Alterações de tampa, botão de energia e manter acordado são salvas na configuração canônica somente após a aplicação em execução ter sucesso. Cancelar a autorização mantém o valor salvo anterior. Quando o drop-in do logind do ARGVUS não fornece um valor, a configuração principal do logind é usada como fallback.

Se uma alteração de retenção falhar, a página relê as definições atuais antes de exibir os valores novamente. A seleção de configuração começa na primeira entrada.

As operações de pacotes do ARGVUS compartilham um lock exclusivo com a restauração seletiva. Instalação, remoção, reinstalação, downgrade, atualização de metadados, limpeza de cache e alterações de repositórios ou mirrors são recusadas enquanto esse lock estiver ocupado. Essa coordenação cobre operações iniciadas pelo helper do ARGVUS; pacman e ferramentas AUR externos não usam esse lock.

Enquanto uma operação em segundo plano aplica alterações, `q` e `Ctrl+C` mantêm o Control Center aberto e exibem um aviso para aguardar. Isso também se aplica após sair da página que iniciou a ação. Aguarde o resultado e solicite a saída novamente. Atualizações comuns de leitura não bloqueiam a saída.

Os campos de palavra-passe Wi-Fi, DNS manual e busca de rede aceitam colagem pelo terminal. Controles e quebras de linha colados são omitidos. Atalhos de caracteres com apenas Ctrl ou Alt que não são suportados não inserem letras. Os editores aceitam caracteres imprimíveis de AltGr codificados como Ctrl+Alt; Ctrl+S salva apenas sem Alt. O editor compartilhado mantém a coluna visual na navegação vertical e alinha a rolagem horizontal de caracteres largos.

Alterações de volume e mute de microfones e saídas não padrão não sobrescrevem mais as preferências salvas da saída padrão. O backend persiste essas preferências apenas quando identifica o dispositivo selecionado como a saída padrão atual. A página exibe volumes amplificados acima de 100% conforme informados pelo PipeWire; ajustes explícitos de volume continuam limitados a 0–100%.

A visão geral de hardware identifica os modelos das GPUs separadamente do endereço PCI e da classe do dispositivo, preservando nomes comerciais entre colchetes. O Resumo mostra o renderizador OpenGL consultado para a sessão e todos os nomes de dispositivos retornados pela consulta Vulkan, incluindo dispositivos por software. Esses resultados globais não são apresentados como capacidades de cada GPU física.

Quando um administrador bloqueia ou remove a conta configurada para login automático, o provedor de contas primeiro desabilita a entrada de login automático do greetd. Se essa limpeza falhar, a operação na conta é recusada. Se o bloqueio ou a remoção falhar depois, o login automático permanece desativado. O login automático de outra conta é preservado; desbloquear uma conta não habilita login automático. Sessões abertas não são encerradas e o greetd não é reiniciado.


## Criar e gerir atalhos de teclado

**Desktop e entrada → Atalhos de Teclado** abre um menu com **Criar atalhos** e **Todos os atalhos**. Os aliases CLI existentes (`keybindings`, `shortcuts`, `keyboard`) abrem esse menu; acrescente `create` ou `list` para abrir uma subtela diretamente. A busca global também encontra as duas subtelas.

**Criar atalhos** pede descrição, combinação de teclas e tipo de ação. Selecione **Atalho atual** e Prima `Enter`; em seguida, Prima a combinação desejada. **Alterar atalho** inicia a mesma captura; nenhuma das opções abre um editor de texto. A captura altera somente o rascunho. Se a combinação já estiver em uso, o conflito aparece imediatamente, mesmo antes de preencher descrição e comando. Confirmar **Substituir** seleciona a combinação e registra sua escolha; isso não cria o atalho nem desativa outro até **Criar**/**Guardar**. Cancelar o conflito preserva a combinação anterior. Um **Comando** executa quando o atalho é pressionado; a criação nunca o executa. Um **Dispatcher do Hyprland** usa um nome Lua suportado, como `window.float`, com argumentos opcionais em um objeto JSON, por exemplo `{"action":"toggle"}`. Submaps, botões do rato e flags avançadas ficam fora desse formulário.

Use `Tab` para alternar entre formulário e **Ações**, `←/→` para mover entre **Criar** e **Cancelar**, e `Enter` para executar a ação selecionada. Cancelar ou `Esc` descarta o rascunho. Depois de criar, **Todos os atalhos** abre com o novo item selecionado e visível. Os personalizados ficam em **Atalhos personalizados**; a busca considera descrição, teclas e ação. `Enter` edita descrição, ação e teclas, `Space` ativa/desativa, e selecionar a linha **Excluir** e pressionar `Enter` abre uma confirmação com foco inicial em Cancelar. Confirmar exclui o registro e volta à lista atualizada; cancelar mantém o atalho.

Atalhos ativos no mesmo contexto não podem reutilizar uma combinação silenciosamente: conflitos configuráveis pedem confirmação para substituir; atalhos protegidos não podem ser substituídos. A substituição salva o novo atalho e desativa os conflitos em uma transação da configuração. **Restaurar padrão** vale somente para os oficiais; **Restaurar todos os atalhos** preserva os registros personalizados. Se a restauração causar colisão com um personalizado ativo, altere ou desative esse personalizado antes de restaurar.

Gravação e aplicação executam em segundo plano. Se a gravação falhar, o rascunho permanece disponível. Se a aplicação na sessão falhar depois de guardar, o ecrã informa que foi salvo e mantém o mesmo ID para tentar novamente. Os registros personalizados ficam em `/hyprland/custom_shortcuts` na configuração canônica, projetados em `data/generated/hypr/custom-shortcuts.lua`; não edite ficheiros gerados.

Os erros de atalhos ficam dentro da TUI: o popup se ajusta à mensagem com quebra de linhas e mantém **Enter OK** visível. Os diagnósticos técnicos de atalhos vão para o journal local (`journalctl -t argvus-control-center`) em vez de stdout/stderr. Uma exclusão salva que não pôde ser aplicada informa essa diferença; recarregue a configuração da sessão para remover o binding que estava ativo.

**Estado das atualizações** em **Barra de tarefas → Ícones → Módulos opcionais** mostra a quantidade de atualizações e vem ativado. **Verificar AUR** vem desativado e requer `paru` ou `yay`; aplique o rascunho da taskbar para guardar as opções. As consultas ocorrem ao iniciar e a cada hora; clicar no indicador abre a página de atualizações oficiais em Pacotes.

## Modo global de janelas

Abra **Control Center → Desktop e entrada → Hyprland → Janelas → Modo** e escolha **Janelas em mosaico (padrão)** ou **Janelas flutuantes**. O mesmo seletor está disponível pelo ícone de modo de janelas no grupo de utilitários da taskbar.

A preferência vale para todos os monitores e workspaces e permanece após encerrar a sessão. O modo flutuante converte janelas existentes de aplicações e novas janelas. O modo mosaico restaura apenas as janelas convertidas pela política; diálogos e outras janelas que já eram flutuantes permanecem assim. Uma janela explicitamente fixada durante o modo flutuante também permanece flutuante para preservar sua fixação. O layout em mosaico é recalculado; a árvore exata de divisões anterior não é restaurada.

`SUPER + SHIFT + Space` mantém o comportamento individual no modo mosaico. No modo flutuante, não coloca uma janela em mosaico. Painéis, taskbar e notificações são superfícies layer-shell e não são afetados. Ecrã cheia e maximização continuam seguindo o comportamento do Hyprland.

A preferência canônica é `/hyprland/window_mode`, em `$XDG_CONFIG_HOME/argvus/config/hyprland.json`. A projeção gerada é `data/generated/hypr/window-mode.lua`; não edite esse ficheiro. O registro de restauração por janela fica em `$XDG_RUNTIME_DIR/argvus`, separado por instância do compositor, e sobrevive ao reload da configuração. Ele não é reutilizado em uma nova sessão.

Se a aplicação estiver pendente ou falhar, a interface distingue esse estado da preferência salva. Selecione novamente a opção para tentar outra vez. Sem compositor, a preferência fica salva para o próximo login.

## Botões de janela e barra de aplicações

O modo flutuante adiciona uma barra de título ARGVUS com **Minimizar**, **Maximizar / Restaurar** e **Fechar** às janelas normais de aplicações. O clique duplo na barra alterna a maximização. A decoração adicional fica oculta em mosaico e ecrã cheia. Aplicações que desenham seus próprios controles podem exibir as duas barras; os controles ARGVUS uniformes são intencionais.

Uma segunda Waybar aparece em cada monitor, na borda oposta à taskbar principal. Ela lista apenas as janelas do workspace atual daquele monitor, incluindo as minimizadas desse workspace. Abas ocultas de grupos não geram entradas extras. As minimizadas mantêm a associação ao monitor e workspace originais. Clique em uma janela sem foco para ativá-la, na janela ativa para minimizá-la ou em uma minimizada para restaurá-la; o menu de contexto oferece restaurar, minimizar, maximizar/restaurar e fechar. Ícones de aplicações desconhecidos usam um símbolo de janela.

`SUPER + Alt + seta para cima` coloca a barra principal no topo e a barra de janelas no rodapé. `SUPER + Alt + seta para baixo` faz o inverso. Repetir a direção mantém as posições. Em mosaico existe apenas a barra principal. Voltar ao mosaico restaura todas as minimizadas antes de remover a barra de janelas e então preserva a política original de janelas tiled/flutuantes.

A minimização usa o workspace reservado `special:argvus-minimized`. Não use esse workspace manualmente. O processo `argvus windows service` registra identidade, workspace/monitor de origem, geometria, ecrã cheia/maximização e fixação em um registro de runtime separado por instância do compositor, em `$XDG_RUNTIME_DIR/argvus`. Os registros sobrevivem ao reinício do serviço ou reload da configuração, mas não atravessam sessões do compositor. Fechar uma janela remove seu registro. Se um monitor desaparecer, a restauração usa uma saída disponível. Uma barra de recuperação ausente ou com falha impede novas minimizações; falhas de restauração preservam o registro e a barra de recuperação.

A sessão gerencia `argvus-windows.service` e `argvus-window-bar.service`. O segundo mantém sua Waybar inativa em mosaico. O plugin hyprbars é distribuído por `argvus-hyprland`, compilado para a ABI do Hyprland 0.57.0 em x86_64; não é compilado no diretório pessoal do utilizador. Recompile o plugin ao atualizar a ABI do compositor.

Comandos: `argvus windows list --json`, `watch --json`, `position top|bottom`, `restore-all` e `minimize|restore|activate|maximize|close ADDRESS`. Os endereços das janelas vêm da listagem. `position top --persist` salva/projeta a preferência sem reiniciar a barra principal. Detalhes de erros vão para o journal; falhas de ações visíveis também exibem uma notificação traduzida.

## Submenu Janelas e fechamento da aplicação

**Desktop e entrada → Hyprland → Janelas** agrupa **Regras**, **Espaços** e **Modo**.
Regras edita o posicionamento por aplicação; Espaços edita os gaps internos e
externos; Modo contém o seletor imediato Tile/Floating, separado do formulário
de regras. A busca abre as mesmas páginas, e Voltar passa por Janelas e Hyprland.

As rotas diretas são `argvus control-center hyprland windows`, opcionalmente
seguidas de `rules`, `spaces` ou `mode`. As rotas existentes `window-rules` e
`appearance window-spaces` continuam como aliases, com os novos breadcrumbs.

`SUPER + Q` fecha a janela dedicada do Control Center imediatamente, sem a
confirmação de janela do Kitty. Rascunhos não aplicados são descartados ao fechar
dessa forma. A opção vale apenas para a classe `argvus-control-center`;
terminais comuns e outros TUIs mantêm sua política de confirmação.

`SUPER + Alt + Espaço` alterna o modo global entre mosaico e flutuante, usando a mesma preferência da taskbar e do Control Center. O comando equivalente é `argvus config window-mode toggle`. Acionamentos simultâneos são serializados; a alternância considera a preferência salva, mesmo quando sua aplicação está pendente. O atalho pode ser alterado ou desativado em Atalhos de teclado; `SUPER + Shift + Espaço` mantém a ação individual.

## Idiomas e Unicode

A página Idioma descobre os catálogos instalados e mostra os nomes nativos dos dez idiomas. Habilite o locale UTF-8 em System Locales antes de selecionar a língua. A seleção continua gravada em `~/.config/locale.conf`; saia e entre novamente para aplicar à sessão inteira. Datas das chaves GPG usam dia/mês/ano nos idiomas europeus e ano/mês/dia em inglês e nos idiomas CJK.
