---
title: Calculadora ARGVUS
description: Básico · Científica · Programação · Financeiro · Conversão de unidades
---

# Calculadora ARGVUS

`argvus calculator` · `SUPER + C` · launcher: **ARGVUS Calculator**

Escolha um perfil em Configurações → Perfil usando ↑/↓ e Enter, ou clique em uma linha. O perfil e suas opções são salvos imediatamente e restaurados na próxima abertura. Cada perfil mantém sua própria expressão durante a execução. Enter ou = substitui a expressão pelo resultado sem selecioná-lo, com o cursor no final.

As taxas financeiras são percentuais por período; use o mesmo período para taxa, quantidade e pagamento. Os pagamentos ocorrem no fim do período. Taxa zero é aceita. Juros simples e compostos retornam o montante acumulado. A conversão é offline, sem moedas. O histórico mantém até 100 registros, incluindo perfil, opções e campos financeiros; selecionar um resultado restaura esse contexto. Limpar o histórico também limpa o arquivo salvo.

## Configurações

- `s Configurações` → **Perfil** → Básico / Científica / Programação / Financeiro / Conversão de unidades.
- ↑ / ↓ · Enter · Escape / ←.
- **Copiar resultado** · **Histórico** · **Limpar histórico**.
- **Básico**: 370 × 420; **Científica**: 440 × 640.
- **Programação**: 520 × 640; **Financeiro**: 440 × 640; **Conversão de unidades**: 440 × 520.

## Básico / Científica

`+ - * / %` · `50% = 0.5` · `1.5 = 1,5` · `2+2` → Enter → `4` → `+2` → Enter → `6`.

`sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `sqrt`, `log`, `ln`, `exp`, `pi`, `e`, `^`, `(`, `)`.

**Unidade angular**: Graus / Radianos. `-2^2 = -4`; `2^3^2 = 512`; `2*pi`.

IEEE 754 / f64 · 15 dígitos significativos · 4096 bytes · 64 níveis do analisador.

## Programação

Padrão: decimal, 64 bits com sinal. A aritmética dá a volta na largura escolhida; o deslocamento à direita com sinal preserva o sinal. A divisão inteira trunca em direção a zero. Divisão por zero e deslocamentos fora de [0, bits) são erros. Números sem prefixo usam a base selecionada. Mudar a base converte a expressão válida; mudar largura ou sinal reinterpreta os mesmos bits.

- **Base**: 2 / 8 / 10 / 16; **Tamanho da palavra (bits)**: 8 / 16 / 32 / 64.
- **Com sinal** / **Sem sinal** · `0b1010`, `0o12`, `0xA`.
- `+ - * / %`, `AND OR XOR NOT`, `<< >>`, `& | ^ ~`, `(`, `)`.
- `9007199254740993+2 = 9007199254740995`; `0xFF AND 0b1010 = 10` (base 10).
- 8-bit: `255+1 = 0`; `NOT 0 = -1` (Com sinal); `-7/2 = -3`.

## Financeiro

Juros simples · Juros compostos · Prestação fixa · Valor presente · Valor futuro · Variação percentual.

- `P`: Capital inicial; `FV`: Montante futuro; `A`: Pagamento por período.
- `r`: Taxa por período (%) / 100; `n`: Número de períodos.
- `G=(1+r)^n`; `H=(G-1)/r` (`r=0`: `H=n`).
- Juros simples: `P*(1+r*n)`; Juros compostos: `P*G`.
- Prestação fixa: `P*G/H`; Valor presente: `(FV+A*H)/G`; Valor futuro: `P*G+A*H`.
- Variação percentual: `(Valor final - Valor original) / Valor original * 100`.

## Conversão de unidades

Comprimento · Massa · Temperatura · Área · Volume · Tempo · Velocidade · Armazenamento de dados.

**De** / **Para**: Configurações · `1500 m → 1.5 km` · `32 °F → 0 °C` · `K ≥ 0`.

`1 B = 8 bit`; `1 kB = 1000 B`; `1 KiB = 1024 B`; `US gal = 3.785411784 L`.

## Histórico

100 · Perfil + `context` · `Limpar histórico`.

`$XDG_DATA_HOME/argvus/calculator/history.json` (→ `~/.local/share/argvus/calculator/history.json`).

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (→ `~/.config/argvus/calculator/settings.json`).

`{"scientific":true}` → `{"profile":"scientific"}`; `{"scientific":false}` → `{"profile":"basic"}`.

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`.

Os textos dos aplicativos seguem Applications Font em Control Center → Appearance → Desktop Style → Fonts. Aplicativos GTK herdam gtk-font-name; os tamanhos específicos de números e títulos são preservados. O launcher lê apps_family gerado e acompanha alterações.

O cabeçalho combina o selo ARGVUS na cor de destaque com o nome do aplicativo. O rodapé clicável s Settings ocupa uma faixa de ponta a ponta com fundo distinto do tema, seguindo a hierarquia visual do Control Center.
