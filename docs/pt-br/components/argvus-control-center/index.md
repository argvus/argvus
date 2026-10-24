---
title: Control Center
description: Configure as partes persistentes do seu desktop ARGVUS.
---

O Control Center é o aplicativo de configurações do ARGVUS, com navegação orientada ao teclado. Use-o quando quiser configurar o desktop, e não apenas executar uma ação pontual.

## Control Center e Control Panel

| Use… | Para… |
| --- | --- |
| **Control Center** | Temas, wallpapers, efeitos, layout, comportamento da taskbar, fontes, aplicativos padrão, teclado e entrada, idioma, região e configuração do sistema. |
| **Control Panel** | Status atual e ações frequentes, como volume, brilho, rede, notificações, energia e controles da sessão. |

O Control Panel pode oferecer controles rápidos para uma configuração permanente do Control Center. Por exemplo, use o painel para uma ação rápida de aparência ou display, e o Control Center quando quiser alterar a configuração do desktop por trás dela. Veja Control Panel e [Onde configurar as coisas](/pt/docs/user-guide/where-to-configure/).

## Áreas principais

A tela inicial atual do Control Center oferece estas áreas:

A tela inicial é um catálogo de destinos agrupados em seis cartões. Cada destino abre a própria página diretamente, e a busca o encontra independentemente do cartão.

* **Aparência** — personalize o desktop: Estilo do desktop (tema, modo de aparência, cor de destaque, wallpapers, ícones e fontes), Estilo dos aplicativos (transparência do terminal e do lançador), Taskbar, Painel de Controle e Widget Telemetry.
* **Desktop e entrada** — configure seu espaço de trabalho: Hyprland (regras de janela, espaços de janela, animações, blur, bordas e a versão instalada), Displays, Teclado e atalhos e Mouse e touchpad.
* **Preferências** — configurações pessoais e regionais: Idioma e região (o hub de idioma, fuso horário, formatos e layout do teclado), Data e hora, Aplicativos padrão, Energia e Sessão.
* **Hardware e conectividade** — dispositivos e conexões: Visão geral do hardware, Rede e Wi-Fi, Bluetooth, Áudio e Armazenamento.
* **Administração do sistema** — gerencie o sistema operacional: Pacotes, Serviços, Boot, Nome do host, Usuários e grupos, Firewall e Notificações (Não perturbe).
* **Configurações** — o próprio Control Center, listado depois dos cartões: Idioma (idioma da interface) e Estilo (ícones decorativos e transparência). Pressione `s` para abri-la.
* **Ferramentas e diagnóstico** — ferramentas de desenvolvimento e manutenção: Projetos, Snippets (textos salvos para o `argvus snippets`, digitados com `SUPER + ALT + N`), Chaves SSH (pares de chaves em `~/.ssh` gerenciados pelo `argvus keys`), Chaves GPG (o keyring gerenciado pelo `argvus gpg`; o gpg-agent usa o pinentry-gtk), Serviços de desenvolvimento, Diagnóstico e Sobre o ARGVUS.

As páginas exatas dos providers podem depender dos pacotes instalados e das permissões. A ausência de um domínio opcional não significa necessariamente que o Control Center esteja quebrado.

Quando a grade tem mais itens do que cabem na tela, uma barra de rolagem aparece na borda direita para indicar que há mais conteúdo abaixo.

## Encontrando uma configuração

A tela inicial registra configurações por título, categoria, identificador e palavras-chave. A busca ignora acentos e aceita termos práticos como `theme`, `wallpaper`, `shortcut`, `mouse`, `monitor`, `network`, `user` e `DPI`. Correspondências exatas de título aparecem primeiro, seguidas por prefixo do título, por qualquer palavra do título ou categoria que contenha o texto digitado e, por fim, por palavras-chave — ou seja, digitar qualquer trecho de uma palavra (não só o início) ainda encontra o destino.

O menu visível reconhece as capacidades disponíveis. Uma rota pode não aparecer quando seu provider não está instalado, quando o hardware não está disponível ou quando a sessão atual não oferece a capacidade necessária. Por isso, a busca também é uma forma de descobrir o que esta instalação pode configurar.

A árvore atual de configurações inclui:

| Área | Destinos atuais |
| --- | --- |
| Aparência | Estilo do desktop: tema, modo de aparência, cor de destaque, wallpapers, ícones e fontes; Estilo dos aplicativos: transparência do terminal e do lançador; Taskbar; Painel de Controle; Widget Telemetry |
| Desktop e entrada | Hyprland (regras de janela, espaços de janela, animações, blur, bordas), Displays, Teclado e atalhos, Mouse e touchpad |
| Preferências | Idioma e região (idioma, fuso horário, formatos, locales do sistema e layout do teclado), Data e hora, Aplicativos padrão, Energia, Sessão |
| Hardware e conectividade | Visão geral do hardware (resumo, CPU, GPU, memória, dispositivos), Rede e Wi-Fi, Bluetooth, Áudio, Armazenamento |
| Administração do sistema | Pacotes, Serviços, Boot, Nome do host, Usuários e grupos, Firewall, Notificações |
| Configurações | Idioma (idioma da interface), Estilo (ícones e transparência) |
| Ferramentas e diagnóstico | Projetos, Snippets, Chaves SSH, Chaves GPG, Serviços de desenvolvimento, Diagnóstico, Sobre o ARGVUS |

Alguns itens são páginas completas e outros são subpáginas alcançadas pela busca ou por um resumo de domínio. A lista exata depende dos providers ARGVUS instalados.

Cada página mostra o seu caminho no cabeçalho, por exemplo `Central de Controle > Hardware e conectividade > Áudio`. O caminho começa no cartão da Home que lista a página, então ela aparece do mesmo jeito pela Home, pela busca e pela CLI. A raiz e os nomes dos cartões são traduzidos; em inglês o cabeçalho mostra `Control Center > Hardware & Connectivity > Audio`.

## Para que serve cada área

### Aparência

Aparência é a principal área de personalização. Seu grupo Área de trabalho reúne o tema (com seu modo Sticky ou Float), o modo de aparência, a cor de destaque, os wallpapers, o tema de ícones e a página Fontes, e seu grupo Estilo dos aplicativos reúne a transparência do terminal e do lançador. Taskbar, Painel de Controle e Widget Telemetry aparecem diretamente em Aparência na tela inicial e reaproveitam a página de transparência compartilhada da Aparência para suas próprias superfícies. O blur é uma única configuração global, em Hyprland > Blur, e se aplica a todas as superfícies do ARGVUS com blur ativo. Nessa página, o controle Ligar e os valores do Hyprland (Tamanho, Passagens, Brilho, Ruído, Contraste, Vibrância e Escurecimento da vibrância) ficam em rascunho, e nada muda até selecionar **Aplicar**. Animações funciona da mesma forma em Hyprland > Animations (Animações): o controle Ligar, as Curvas personalizadas (Bézier, Spring, Linear e Ease-out) e as cinco categorias de animações formam um único rascunho, aplicado junto com **Aplicar**. Bézier e Spring têm parâmetros editáveis (os pontos da Bézier e a massa, a rigidez e o amortecimento da mola); Linear e Ease-out são curvas fixas, então a página apenas lista seus pontos de controle e não pode ser editada. As opções de estilo seguem o que o Hyprland 0.56.2 desenha: abertura e fechamento de janelas oferecem slide, popin e gnomed; entrada e saída de camada oferecem slide, popin e fade; e as leaves de áreas de trabalho oferecem slide, fade, slidefade, slidevert e slidefadevert. Esses controles coordenam os componentes de aparência e sessão, em vez de alterar apenas a janela do Control Center.

### Taskbar

A Taskbar configura a posição do painel, o espaçamento ao redor dele e das janelas, o grupo de ícones utilitários e os blocos de data/hora, além da própria transparência. Aparece no cartão **Aparência** da tela inicial.

### Control Panel

O Control Panel configura a visibilidade e a ordem dos seus cards de ação rápida, além das próprias sessões e transparência.

A página abre e aceita comandos assim que as configurações dos cards são lidas. Os cards que dependem de hardware (Bluetooth e Brilho) são verificados em segundo plano: até essa verificação terminar, eles mantêm a disponibilidade vista por último nesta janela, e a lista se atualiza sozinha quando o resultado chega.

### Widget Telemetry

O Widget Telemetry configura as sessões e a transparência do widget de informações do sistema.

### Fontes

Fontes fica em **Aparência > Estilo do desktop > Fontes**. A página permite selecionar família e tamanho para taskbar, telemetria/informações do sistema, Control Panel, interface do sistema ARGVUS, aplicativos, terminal e navegador. Também expõe configurações de renderização como antialiasing, hinting, modo subpixel e DPI. A família principal padrão é IBM Plex Mono; cada alvo pode ser alterado independentemente e restaurado pela página Fontes.

### Aplicativos padrão

Aplicativos padrão seleciona programas instalados para funções como terminal, gerenciador de arquivos, editor de texto, editor do terminal, navegador, visualizador de imagens, visualizador de PDF, reprodutor de vídeo, reprodutor de áudio, ferramenta de arquivos compactados e launcher. A seleção é salva pelo ARGVUS e pode atualizar associações XDG padrão.

### Snippets

Snippets é a página dos textos que o `argvus snippets` digita com `SUPER + ALT + N`. Ela fica em **Ferramentas e diagnóstico → Snippets** (busque por `snippets` ou execute `argvus control-center snippets`).

* **Novo snippet** — digite um **Nome** e um **Conteúdo** e depois **Salvar snippet**. Um nome que já existe substitui o conteúdo desse snippet.
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
2. **Encontrar uma chave:** pressione `/` e digite para mostrar só os pares cujo nome ou comentário combina. `Enter` encerra a busca e executa a linha destacada, como nas outras listas, então confira a linha antes de apertar; `Esc` limpa a busca.
3. **Criar um par:** em **Gerar**, selecione **Nome**, pressione `Enter`, digite o nome e pressione `Enter` de novo. Selecione **Tipo** e pressione `Enter` para alternar entre `ed25519` e `rsa`. Selecione **Gerar par de chaves** e pressione `Enter`. O novo par aparece em **Chaves** imediatamente, sem senha.
4. **Copiar a chave pública:** no bloco do par, selecione **Copiar chave pública** e pressione `Enter`. O conteúdo do arquivo `.pub` vai para a área de transferência do Wayland, pronto para colar no `authorized_keys` de um servidor ou em um host Git. A chave privada nunca é copiada. Isso precisa do `wl-clipboard` (`wl-copy`); sem ele, a página informa que a chave não foi copiada.
5. **Carregar um par no agente:** no bloco do par, selecione **Carregar no agente** e pressione `Enter`. Uma chave sem senha é carregada na hora, sem janela. Uma chave com senha abre um terminal ARGVUS, onde o OpenSSH a pede; feche o terminal com `Enter` quando ele indicar que a chave foi carregada. Enquanto a página estiver aberta, ela consulta o agente a cada poucos segundos, então a linha muda sozinha para **Descarregar do agente**. Para atualizar na hora, use **Atualizar** em **Ações**.
6. **Remover uma chave do agente:** no bloco do par, selecione **Descarregar do agente**. Para remover todas, use **Descarregar todas as chaves do agente** em **Ações**. Os arquivos das chaves continuam no disco.
7. **Apagar um par:** no bloco do par, selecione **Apagar** e pressione `Enter`. A confirmação começa com o foco em **Cancelar**, então um `Enter` acidental não faz nada. Pressione `y` para confirmar, ou `n`/`Esc` para cancelar. Uma chave carregada no agente não pode ser apagada; descarregue-a antes.

Para se mover: `↑`/`↓` ou `j`/`k` percorrem a lista, `Enter` ativa a linha, `Esc` volta e `?` mostra a ajuda. `Tab` leva o foco para **Ações**, onde `←`/`→` escolhem um botão e `Enter` o executa. `↑`/`↓` devolvem o foco à lista.

As senhas nunca são digitadas no Control Center nem passadas como argumentos de comando. A página lê a lista com `argvus keys list` e grava somente por meio do `argvus keys`. Quando o `argvus-keys` não está instalado, a página informa isso.

#### Solução de problemas

| Sintoma | Causa e correção |
| --- | --- |
| `Could not open a connection to your authentication agent` após **Carregar no agente** | O terminal não recebeu o socket do agente. O Hyprland mantém o ambiente com que foi iniciado, e o agente pode ter começado depois. Atualize o `argvus-launcher`, que passa a usar `$XDG_RUNTIME_DIR/ssh-agent.socket`, e verifique `systemctl --user status argvus-ssh-agent.service`. |
| **Agente** mostra `Agente indisponível` | O launcher não consegue alcançar o agente. Verifique o serviço como acima. |
| A página diz que o `argvus-keys` não está instalado | Atualize o pacote `argvus`: `argvus keys` é um subcomando dele. |
| **Apagar** mostra um erro | A chave está carregada no agente e foi mantida. Descarregue-a primeiro e apague de novo. |

### Chaves GPG

Chaves GPG é a página do keyring que o `argvus gpg` gerencia. Ela fica em **Ferramentas e diagnóstico → Chaves GPG** (busque por `chaves gpg` ou rode `argvus control-center gpg-keys`).

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
3. **Criar uma chave:** em **Gerar chave ▾**, digite **Nome completo** e **E-mail**. Os dois são obrigatórios, e **Gerar chave** fica desabilitado até que ambos tenham texto. **Comentário** é opcional. A identidade fica `Nome (comentário) <e-mail>`.
   - **Algoritmo** alterna entre **RSA** e **DSA ElGamal**. Uma chave DSA recebe uma subchave de criptografia ElGamal; uma chave RSA recebe uma RSA.
   - **Tamanho da chave** percorre os tamanhos do algoritmo: RSA 1024, 2048 e 4096; DSA 1024, 2048 e 3072. O gpg não cria chaves DSA acima de 3072 bits, por isso 4096 não é oferecido para DSA.
   - **Data de expiração** abre uma página própria. Digite a data no formato do seu idioma: `DD/MM/AAAA` em pt-BR, `AAAA/MM/DD` em inglês. A data precisa ser posterior a hoje. Ative **Nunca expira** para tornar a chave permanente; o campo de data fica desabilitado e não é usado.
   - **Senha** abre uma página própria. **Proteger com senha** vem ligado por padrão; digite a **Senha** e depois **Confirmar senha**. As duas precisam ser iguais. Desligar a proteção limpa os dois campos, e a página pede confirmação antes de gerar uma chave sem proteção.
4. Selecione **Gerar chave** e pressione `Enter`. A chave recebe uma chave de assinatura e uma subchave de criptografia. A senha vai ao launcher pela entrada padrão, nunca como argumento de comando. Se a geração falhar, os outros campos permanecem, mas as senhas são limpas e precisam ser digitadas de novo.
5. **Cancelar** limpa todos os campos do formulário e volta datas e tamanhos aos padrões.
6. **Importar:** em **Importar ▾**, escolha **Importar chave pública…** ou **Importar chave privada…**. O gerenciador de arquivos abre para você escolher o arquivo. Uma chave privada pede confirmação antes; a senha dela é pedida pelo gpg quando a chave é usada. Se o gerenciador de arquivos for gráfico, ele não consegue informar o arquivo à página; então a página abre um campo onde você digita o caminho.
7. **Chaves ›** abre o chaveiro. Cada chave mostra identidade, fingerprint, tipo, validade e se a chave privada está no chaveiro. Cada chave tem estas ações:
   - **Editar chave ›** abre a página própria da chave (descrita abaixo).
   - **Exportar chave pública…** pede o caminho de um arquivo, sugerido como `~/<fingerprint>.asc`. O arquivo é criado com modo `600`. Um arquivo existente nunca é substituído.
   - **Exportar chave privada…** aparece só para chaves com parte privada. Pede confirmação e depois o caminho de um arquivo, sugerido como `~/<fingerprint>-private.asc`. O arquivo é criado com modo `600`.
   - **Copiar chave pública** e **Copiar fingerprint** copiam para a área de transferência Wayland com o `wl-copy`.
   - **Excluir chave…** pede confirmação, com o foco em Cancelar. Remove a chave e a parte privada dela.
8. **Encontrar uma chave:** na página **Chaves**, a barra de busca no topo filtra as chaves. Digite para restringir a lista; a busca compara a identidade (que inclui o comentário) e o fingerprint. `Esc` limpa a busca.
9. **Editar chave ›** mostra os detalhes da chave e depois estas ações:
   - **Data de expiração ›** altera a validade da chave e de cada subchave. Use **Nunca expira** para remover a validade. **Aplicar** salva a alteração.
   - **Alterar senha** pede a senha antiga e a nova na janela do pinentry.
   - **Assinar chave com ›** certifica esta chave com uma das suas outras chaves que tenha parte privada. A senha do assinante é pedida. A página pede confirmação antes.
   - **Revogar chave ›** lista os motivos: sem motivo, chave comprometida, chave substituída, chave não usada mais. Cada um pede confirmação. Uma chave revogada não pode mais ser usada.
   - **Confiança** define o quanto você confia na chave: não definida, marginal, total ou máxima.
   - **Publicar no servidor de chaves…** envia a chave pública ao servidor configurado no `gpg.conf`, depois de confirmar. **Buscar no servidor de chaves** baixa a chave desse servidor.

Para se mover: `↑`/`↓` ou `j`/`k` percorrem a lista, `Enter` ativa uma linha, `Esc` volta e `?` mostra a ajuda.

As senhas são digitadas apenas nos campos mascarados da página de senha ou na janela do pinentry. A página envia a senha da geração ao `argvus gpg` pela entrada padrão, e o launcher a entrega ao gpg por um descritor de arquivo; assim ela nunca aparece na lista de processos, em argumento de comando ou nos logs. As alterações de chave rodam em segundo plano; enquanto uma roda, as outras esperam. A página lê o keyring com `argvus gpg list` e escreve apenas pelo `argvus gpg`. Quando o `argvus gpg` não está instalado, a página informa isso.

Ainda não está na página: buscar uma chave no servidor por nome ou e-mail. Use **Buscar no servidor de chaves** com uma impressão digital conhecida.

#### Solução de problemas

| Sintoma | Causa e correção |
| --- | --- |
| A página diz `O argvus gpg não está disponível` | O `argvus-launcher` instalado é anterior ao comando GPG. Rode `argvus gpg list` em um terminal: `unknown command` significa que o pacote precisa ser recompilado e instalado. As chaves também precisam do `gnupg`. |
| **Gerar chave** continua desabilitado | Digite **Nome completo** e **E-mail**. |
| Uma linha de status diz `argvus-gpg: pinentry-gtk não está instalado` | Instale o pacote `pinentry`, que fornece o `pinentry-gtk`. |
| A geração para com erro de `pinentry-program` | O `gpg-agent.conf` define um pinentry fora do bloco do ARGVUS. Remova essa linha; o ARGVUS grava a sua. |
| **Pinentry** mostra `Não definido` | O gpg-agent usa o padrão embutido até a primeira chave ser gerada pela página. |
| A página diz que o `argvus-launcher` instalado é mais antigo que o Control Center | Reinstale o `argvus-launcher`: rode `make build` e depois `make install` no repositório dele, e reabra a página. As ações de chave ficam desligadas até as versões baterem. |
| O gerenciador de arquivos abre, mas a página pede um caminho | O gerenciador é gráfico e não consegue informar o arquivo escolhido. Digite o caminho no campo. |
| **Copiar chave pública** diz que não conseguiu copiar | O `wl-copy` não está instalado ou a sessão Wayland não está disponível. Instale o `wl-clipboard`. |
| **Editar chave ›** diz que a chave não está mais no chaveiro | O chaveiro mudou depois que a página abriu. Volte para **Chaves ›** e atualize. |
| Uma linha de status mostra `argvus-gpg: ...` | O launcher recusou a ação, e o texto explica o motivo, como uma validade inválida, um arquivo que já existe ou um `--yes` ausente. |
| Uma ação espera e nada acontece | A janela do pinentry está escondida ou não foi aberta. Confira se a sessão expõe `WAYLAND_DISPLAY` ou `DISPLAY`. |
| A importação é recusada | O arquivo não contém a chave do tipo escolhido. Chaves públicas vão por **Importar chave pública…**; privadas por **Importar chave privada…**. |

### Idioma e região

Este cartão lista Idioma (a localidade regional), Fuso horário, Data e hora, Locales do sistema e Layout do teclado, e cada um abre diretamente pela tela inicial. As páginas de teclado também concentram a variante e o mapa de teclado do console. A busca inicial também expõe atalhos de teclado e mouse/touchpad como rotas diretas de configuração.

### Hardware e displays

Quando os recursos correspondentes estão instalados, o Control Center pode mostrar informações de CPU, GPU, memória, energia e dispositivos. A página de entrada configura o comportamento do mouse e touchpad; a área de displays lida com resolução, taxa de atualização, escala, posição, orientação, display principal, VRR e HDR quando suportados pela sessão ativa. As mudanças de display valem na hora. Depois de uma mudança que pode deixar a tela inutilizável (resolução ou taxa, posição, espelhamento, desativar um monitor, profundidade de cor) ou depois de **Aplicar**, o Control Center pergunta se deve mantê-la, com o foco em **Reverter**: pressione `y` para manter; `n`, `Esc` ou `Enter` revertem, e sem resposta em 15 segundos a configuração anterior volta automaticamente. Remover a configuração salva de um monitor desconectado e excluir um perfil pedem confirmação; os perfis abrem uma página própria com **Aplicar perfil**, **Renomear** e **Excluir perfil**. O layout é salvo por meio de `argvus config monitor-set`, `monitor-primary`, `monitor-workspaces` e `monitor-remove`; por isso a seção canônica `/displays` do `config.json` é a única fonte do layout dos monitores. Só o catálogo de perfis continua em `display/config.toml`.

### Conectividade e áudio

A área de rede expõe o status e as páginas do provider de rede instalado, incluindo rotas de Wi-Fi, Ethernet, VPN, DNS e proxy quando disponíveis. O firewall fica listado em **Administração do sistema**. Bluetooth depende do suporte Bluetooth. A área de áudio aparece quando o recurso opcional correspondente está habilitado e integra-se ao serviço de áudio instalado.

### Energia e sessão

Energia oferece os controles de energia do sistema e as páginas de política disponíveis na instalação, incluindo a opção Manter acordado. Quando ativada, Manter acordado impede que a política de ociosidade do ARGVUS inicie os temporizadores automáticos de bloqueio da tela e desligamento do display. Sessão expõe status da sessão ARGVUS, componentes, autostart, diagnósticos e logs. Essas páginas podem exigir permissões do sistema e devem ser diferenciadas das ações rápidas do Control Panel.

### Ferramentas do sistema

A seção de sistema pode expor informações de boot, pacotes, serviços, armazenamento, diagnósticos, administração de usuários/grupos e informações do sistema. São ferramentas administrativas ou de diagnóstico; uma página pode ser somente leitura ou exigir autorização dependendo da operação.

**Serviços de Dev** reúne três fontes voltadas a desenvolvimento em uma única página de leitura: os serviços systemd do usuário atual, as portas TCP em escuta (via `ss`) e os containers em execução de qualquer runtime instalado (Podman, Docker). A página só aparece na grade inicial quando pelo menos uma dessas fontes está disponível no sistema — uma máquina sem sessão systemd de usuário, sem Podman e sem Docker não mostra a entrada. `r` atualiza as três fontes de forma independente, então uma fonte lenta ou com falha (por exemplo, um runtime de containers que parou de responder) não impede as outras de mostrar seus dados.

Nas listas de **Pacotes** que filtram ao digitar (Buscar, AUR, Instalados, Órfãos e Atualizações), as letras vão para o filtro; use as setas para navegar e `/` para digitar uma nova busca. `Enter` num pacote abre a página dele, com **Instalar** (ou **Reinstalar**) e, na **Zona de perigo**, **Remover**. Em **Órfãos**, `Space` marca pacotes (`[x]`) e **Remover marcados** remove todos juntos. **Atualizar tudo** e **Atualizar banco** são linhas da página Atualizações, as limpezas de cache ficam na Zona de perigo da página Cache, e **Mirrors → Configurar mirrors** define as opções do reflector antes de **Gerar preview**. Toda operação de pacotes mostra o plano e pede confirmação (`y` confirma, `n` ou `Esc` cancela) antes de rodar, e a saída aparece enquanto ela roda.

### Usuários e grupos

Quando o provider de contas está disponível, abra **Control Center → Administração do sistema → Usuários e grupos → Usuários** ou busque por **usuários**. A interface pode listar contas normais e de sistema, criar usuários, editar metadados, gerenciar grupos suplementares e primários, alterar ou bloquear uma senha, desbloqueá-la, exigir troca de senha no próximo login, definir ou remover um avatar, e ativar o login automático.

A criação pede username, nome completo, shell, grupos suplementares e uma senha opcional com confirmação. Uma senha vazia cria a conta com a senha bloqueada; não cria silenciosamente uma senha utilizável. A página de um usuário agrupa os campos da conta (nome completo, shell, grupo principal e grupos suplementares, seguidos de **Salvar alterações**), o alternador de **Login automático**, as ações de **Senha** (alterar, bloquear, desbloquear, exigir troca no próximo login) e as de **Avatar**. Username, UID / GID e diretório home aparecem só como informação. A **Zona de perigo** no fim reúne as ações destrutivas separadas para remover a conta mantendo o diretório home ou removendo também o home. Leia atentamente a confirmação antes de escolher a segunda opção. Operações de conta exigem privilégio e podem abrir o prompt de autorização do sistema. Avatares podem ser consumidos pelo greeter por meio da integração padrão de imagem da conta.

O **Login automático** inicia a sessão da conta selecionada no próximo boot sem mostrar a tela de login, a mesma convenção usada por GNOME, LightDM e SDDM para esse recurso: nenhuma senha é armazenada em lugar nenhum. Ativá-lo para uma conta desativa automaticamente qualquer conta configurada anteriormente (apenas uma conta pode ter login automático por vez). Ele vale apenas para o próximo boot; depois de um logout manual, a tela de login normal volta a aparecer pelo resto daquela sessão. O alternador é aplicado imediatamente, com uma confirmação, e está disponível apenas para administradores.

Os grupos possuem suas próprias páginas de lista e edição, incluindo gerenciamento de membros e exclusão de grupo. O Control Center não substitui as políticas de contas do sistema: contas protegidas e operações que exigem autorização continuam sujeitas às restrições do sistema operacional.

### Configurações do Control Center

**Configurações** é o último item da tela inicial. A página **Idioma** define o idioma da interface, e a página **Estilo** controla os ícones decorativos e a transparência do Control Center. O blur não fica em Estilo: ele é uma configuração global do Hyprland, em Hyprland > Blur. A página Estilo altera a apresentação do aplicativo de configurações; ela não desativa serviços ARGVUS nem remove providers. Salvar os ícones pode exigir autorização, enquanto a alteração visual é aplicada ao aplicativo atual. `Esc` em Estilo volta à página Configurações, e `Esc` nela volta à tela inicial.

## Busca e recursos opcionais

A tela inicial possui um registro pesquisável de rotas. Os termos incluem conceitos voltados ao usuário, como aparência, temas, wallpapers, fontes, atalhos de teclado, mouse, touchpad, displays, rede e sessão. Providers opcionais só aparecem quando o recurso foi compilado e sua capacidade de runtime foi detectada; por isso o menu visível é consciente das capacidades disponíveis.

Enquanto a busca da tela inicial estiver ativa, as teclas imprimíveis são inseridas na consulta, incluindo `j` e `k`; use as setas `↑` e `↓` para percorrer as rotas encontradas. Pressione `Enter` para abrir a rota selecionada, `Backspace` para editar a consulta e `Esc` para limpar a consulta ou sair do modo de busca.

## Fluxo de aparência

Abra **Aparência** para ver os controles visuais integrados. Seus grupos são:

* **Área de trabalho** — **Tema** (escolha uma família incluída e seu modo Sticky ou Float, ou gerencie perfis de tema personalizados), **Modo de aparência** (Sticky ou Float, independente do tema), **Cor de destaque** (um acento incluído ou uma cor RGB válida de seis dígitos), **Wallpaper** (um wallpaper incluído ou um arquivo personalizado) **Ícones** (um tema de ícones instalado: pesquise na lista com `/`, escolha um com Espaço ou Enter e selecione **Aplicar**; nada muda antes de Aplicar) e **Fontes** (os alvos de fonte e as configurações de renderização, abertos na página Fontes).
* **Estilo dos aplicativos** — a transparência do **Terminal** e do **Lançador**.

O cartão Aparência reúne as superfícies que usam essas configurações: **Widget Telemetry** (ative a superfície de telemetria, selecione seus blocos disponíveis e reordene-os com `Shift+↑`/`Shift+↓` em um bloco focado) e **Control Panel** (ative, desative e reordene os cards do painel com `Shift+↑`/`Shift+↓` em um card focado, aplicado na hora através do mesmo mecanismo que o arrastar do painel real usa).

Essas ações atualizam o estado lógico do ARGVUS e aplicam a configuração de runtime afetada. Veja [Aparência](/pt/docs/user-guide/appearance/), [Temas](/pt/docs/user-guide/appearance/) e [Janelas e layout](/pt/docs/argvus-hyprland/windows-and-layout/).

## Uso pelo teclado

O aplicativo de configurações suporta navegação pelo teclado. Cada página é uma lista única: use as setas ou `j`/`k` para mover, `Enter` para abrir ou executar a linha selecionada, `Space` para alternar, `Esc` para voltar, `/` para buscar listas e `?` para ajuda; o rodapé mostra as teclas que valem para a linha selecionada. As ações são linhas da lista, não botões, e `Tab` não leva até elas. Linhas que só mostram informação são puladas. Enquanto um campo de texto, uma busca ou um filtro de lista estiver ativo, `q` e `?` são digitados como texto em vez de sair ou abrir a ajuda. **Restaurar padrões** e **Restaurar todos os atalhos** são linhas de uma **Zona de perigo** no fim das suas páginas; `r` pede a mesma restauração, e na lista de atalhos `r` restaura o atalho selecionado depois de uma confirmação. Em **Idioma e região → Layout do teclado**, `/` foca a caixa **Buscar** no topo da página, e `Esc` a limpa; `Enter` define o layout padrão (marcado com `●`) e `Space` adiciona ou remove um layout (`[x]`); essas teclas só alteram um rascunho, e o botão **Aplicar** na barra **Ações**, fixa embaixo da lista, grava os layouts e o padrão no Hyprland de uma vez. Na lista, `Tab` ou `←/→` levam o foco até essa barra enquanto **Aplicar** estiver habilitado (o rodapé mostra então `←/→ Ações`); `←/→` percorrem os botões dela e `↑/↓` devolvem o foco à lista. Sair da página com alterações não aplicadas pede confirmação antes. Em **Idioma e região → Fuso horário** o layout é o mesmo: `/` foca a caixa **Buscar** no topo, `Enter` escolhe um fuso em um rascunho (marcado com `●`), e o botão **Aplicar** na barra **Ações**, embaixo da lista, grava o fuso no sistema com `timedatectl`; `Tab`, `←/→` e `↑/↓` funcionam como em Layout do teclado. Sair com um fuso não aplicado pede confirmação antes. Em **Locales do sistema**, `Enter` ou `Space` marcam um locale e a linha **Aplicar** no fim grava a seleção.

As páginas que juntam alterações antes de salvar (um usuário, um grupo, a criação de conta, a troca de senha, a configuração do firewall e Locales do sistema) terminam esse bloco com a linha **Salvar** ou **Aplicar**, que fica apagada até haver alguma alteração. Se voltar descartaria alterações não salvas, o Control Center pergunta antes. Toda confirmação mostra **Confirmar** e **Cancelar** como duas linhas, com o foco em **Cancelar**: `Enter` executa a linha em foco, `y` confirma e `n` ou `Esc` cancelam.

Em **Aparência**, cada página é uma lista única: o rodapé mostra só as teclas que valem para a linha selecionada, e `Space` também abre ou seleciona a linha. As páginas que juntam alterações antes de aplicar (Taskbar, Widget Telemetry, Control Panel e os controles de efeito) terminam com a linha **Aplicar**, que fica apagada até haver alguma alteração. Nas linhas de porcentagem, `←`/`→` (ou `+`/`-`) mudam o valor de 5 em 5 e `Enter` permite digitá-lo; use `Esc` para voltar. Se voltar descartaria alterações não aplicadas, o Control Center pergunta antes. Excluir um tema personalizado (`d`) ou substituir um tema importado pede confirmação; o foco começa em **Cancelar**, `y` confirma e `n` ou `Esc` cancelam.

## Persistência e restauração

As páginas aplicam alterações pelo provider responsável e salvam o estado de usuário suportado. As mudanças de aparência ficam na configuração canônica, e o `argvus-config` é o único componente que escreve os arquivos de consumo derivados em `~/.config/argvus/data/generated/`; esses arquivos gerados não são o local para uma edição permanente. As páginas de entrada e atalhos possuem estado persistente e ações de restauração próprias. Outras alterações do sistema podem exigir permissões ou recarga de serviço.

Não existe uma restauração global para todas as configurações do ARGVUS. Restaure uma alteração pela página responsável ou use o procedimento de recuperação documentado para o recurso correspondente.

O suporte à restauração é específico de cada domínio:

* **Aparência** oferece restauração da cor de destaque para o padrão do tema ativo. Não existe uma restauração global de aparência; restaure os demais valores pelos próprios controles.
* **Fontes** pode restaurar todas as fontes, um alvo ou uma configuração individual.
* **Aplicativos padrão** pode restaurar todos os padrões, uma categoria ou um seletor individual.
* **Atalhos de teclado** pode restaurar um atalho ou todos e então recarregar os atalhos gerados da sessão.
* **Regras de janela** ficam em **Desktop e entrada → Hyprland → Regras de janela** (busque por `regras de janela` ou execute `argvus control-center window-rules`). Cada regra tem a linha **Workspace** (`Enter` ou `←/→` percorrem de 1 a 10) e a linha **Classes de janela**, que abre um campo de texto com expressões regulares separadas por vírgula. **Adicionar regra** cria `rule-N` no workspace 1, sem classes. **Remover regra** de cada regra fica na **Zona de perigo** e pede confirmação. Cada alteração é gravada em `hyprland.window_rules` no `argvus-config` e recarrega a configuração na hora; padrões de classe inválidos são recusados e mostrados como erro.

* **Projetos** ficam em **Ferramentas e diagnóstico → Projetos** (busque por `projetos` ou execute `argvus control-center projects`). **Adicionar pasta de projeto** e **Adicionar pasta raiz** ficam no topo da página e abrem um campo de texto com o caminho; o launcher confere se é um diretório e mostra o erro caso não seja, inclusive quando a pasta levaria o total efetivo de projetos além de 9 (o `SUPER + ALT + 1..9` não consegue endereçar mais que isso). Cada pasta configurada, marcada como raiz (seus subdiretórios são projetos) ou projeto único, ganha seu próprio bloco abaixo do formulário com uma linha **Remover** que pede confirmação. A página chama o `argvus-projects`, então as alterações são gravadas pelo launcher e valem na próxima vez que `SUPER + O` ou `SUPER + ALT + 1..9` forem usados. Se o `argvus-projects` não estiver instalado, a página informa isso.
* **Outras páginas do sistema** exibem ações de restauração, aplicação, exclusão ou recuperação somente quando o provider responsável oferece esse recurso.

Remover uma preferência do usuário pode fazer um provider retornar ao estado padrão, mas apagar arquivos manualmente não é um procedimento geral de recuperação. Prefira a ação de restauração da página ou o comando documentado para o recurso.

## Entradas pela CLI

O auxiliar instalado é `argvus-control-center`, executado por `argvus control-center`. Ele pode abrir áreas específicas diretamente, por exemplo:

```sh
argvus control-center appearance themes
argvus control-center appearance wallpapers
argvus control-center input
argvus control-center keybindings
```

Execute `argvus control-center --help` para ver as rotas disponíveis na versão instalada. A rota pela linha de comando é um atalho para o mesmo aplicativo, não um segundo sistema de configuração.

Rotas diretas úteis incluem `apps`, `window-rules`, `projects`, `snippets`, `ssh-keys`, `gpg-keys`, `firewall`, `fonts`, `locale`, `input`, `keybindings`, `language`, `config`, `system`, `hardware`, `services`, `dev-services`, `network`, `audio`, `bluetooth`, `boot`, `packages`, `storage`, `diagnostics`, `power`, `session`, `displays` e `appearance`. Appearance também aceita rotas específicas como `themes`, `wallpapers`, `icons`, `accents`, `effects`, `spaces`, `taskbar` e `widget-telemetry`; displays aceita `resolution`, `refresh`, `scale`, `position`, `orientation`, `primary`, `vrr` e `hdr`. Use a saída `--help` instalada ao criar scripts, pois a disponibilidade ainda depende do build instalado.

## Relacionados

* [Primeira configuração](/pt/docs/getting-started/)
* Control Panel
* [Aparência](/pt/docs/user-guide/appearance/)
* [Atalhos de teclado](/pt/docs/argvus-hyprland/keyboard-shortcuts/)
* [Mouse e touchpad](/pt/docs/argvus-hyprland/input/)
