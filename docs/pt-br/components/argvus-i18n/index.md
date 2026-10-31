---
title: Internacionalização ARGVUS
description: Sistema de suporte multilíngue para ARGVUS
---

# Internacionalização ARGVUS (i18n)

O módulo de Internacionalização do ARGVUS fornece suporte de idioma em todos os componentes e aplicativos ARGVUS, habilitando uma experiência verdadeiramente multilíngue na área de trabalho.

## Idiomas Suportados

Os idiomas atualmente suportados incluem:
- English (en)
- Portuguese - Brasil (pt-BR)
- Idiomas adicionais em breve

## Seleção de idioma

Escolha o idioma do usuário em Control Center > Preferências > Idioma e região > Idioma ou Configurações > Idioma. As duas telas salvam `LANG` em `$XDG_CONFIG_HOME/locale.conf` (padrão `~/.config/locale.conf`). Sem preferência válida, o ARGVUS usa inglês. Variáveis globais de locale e arquivos legados de idioma do ARGVUS não substituem esse arquivo. Saia e entre novamente após trocar o idioma.

## Para Colaboradores

Para contribuir com traduções ou adicionar suporte para novos idiomas:

1. Visite a plataforma de tradução
2. Junte-se como tradutor
3. Envie traduções para revisão
4. As traduções são mescladas em cada lançamento

## Arquivos de Tradução

Os arquivos de tradução usam o formato padrão `.po` (Portable Object) e são gerenciados através de:
- Diretório `po/` nos repositórios ARGVUS
- Sistema de memória de tradução para consistência

## Localização

Além do idioma, a localização inclui:
- Formatos de data e hora
- Formatação de números
- Exibição de moeda
- Layouts de teclado

Para personalização, verifique as configurações de localização do sistema ARGVUS.
