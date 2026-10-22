---
title: Contas
description: Perfis locais, nomes de exibição, avatares e grupos, pelo argvus accounts.
---

`argvus accounts` lista e edita as contas locais do desktop ARGVUS. É um executável privado do
pacote `argvus`, instalado em `/usr/lib/argvus/commands/accounts`. O comando `argvus` o inicia, e
só as operações que alteram outras contas ou dados do sistema rodam como root.

## Comandos

```text
argvus accounts list [--all]                   lista usuários (--all inclui contas de sistema)
argvus accounts show USUARIO                   mostra os detalhes de uma conta
argvus accounts name USUARIO "NOME"            altera o nome de exibição de outro usuário
argvus accounts avatar USUARIO IMAGEM          define o avatar de outro usuário
argvus accounts avatar USUARIO --remove        remove o avatar de outro usuário
argvus accounts groups USUARIO                 lista os grupos
argvus accounts groups USUARIO --add G --remove G altera grupos (exige autorização)
argvus accounts passwd USUARIO                 altera uma senha (lê três linhas da entrada padrão)
argvus accounts self                           mostra a sua conta
argvus accounts self name "NOME"               altera o seu nome de exibição
argvus accounts self avatar IMAGEM             define o seu avatar
argvus accounts self avatar --remove           remove o seu avatar
argvus accounts self groups                    lista os seus grupos
argvus accounts --verbose ...                  imprime informações de diagnóstico no stderr
```

## Nomes de exibição e avatares

O nome de exibição é guardado no campo GECOS padrão. O nome de usuário Unix nunca é alterado.

Um avatar é uma imagem nos formatos PNG, JPEG ou WebP, de até 20 MiB. A imagem é redimensionada
para 256×256 pixels e salva como PNG com modo `0644` em `~/.face`, o local freedesktop lido pelo
greeter e pela maioria dos gerenciadores de login. O seu próprio avatar é gravado dentro do seu
diretório pessoal e não exige autorização.

## Autorização

Somente `list`, `show`, `self`, `self groups` e o seu próprio avatar funcionam sem autorização.
Qualquer outra alteração pede autorização ao agente polkit do desktop. A regra do ARGVUS
(`/usr/share/polkit-1/rules.d/50-argvus-accounts.rules`) define quem precisa se autenticar:

- um usuário de um grupo de administradores (`wheel`, o padrão do Arch, ou `sudo`) confirma com a
  própria senha;
- qualquer outro usuário confirma com a senha do root.

| Operação | Sem autorização | Com autorização |
| --- | --- | --- |
| Listar usuários, mostrar uma conta, listar grupos | ✓ | ✓ |
| Alterar o seu próprio avatar | ✓ | ✓ |
| Alterar o seu próprio nome de exibição | — | ✓ |
| Alterar a sua própria senha | — | ✓ (após comprovar a senha atual) |
| Alterar nome, avatar, grupos ou senha de outro usuário | — | ✓ |

Ao alterar a própria senha, a senha atual é verificada antes. Uma senha errada é recusada antes
de o pedido de autorização aparecer.

Não é preciso executar o comando com `sudo`. O comando pede autorização ao polkit quando precisa.

## Limites de segurança

- `passwd` lê a senha atual, a nova e a confirmação da entrada padrão, uma por linha, e nunca as
  recebe como argumentos. A linha da senha atual pode ficar vazia quando um administrador redefine a
  senha de outro usuário. Exemplo:

  ```sh
  printf '%s\n%s\n%s\n' "$ANTIGA" "$NOVA" "$NOVA" | argvus accounts passwd ghost
  ```

  No terminal, digite as três linhas e pressione Enter após cada uma. O texto aparece na tela à
  medida que é digitado.
- Editar uma conta com UID abaixo de 1000 (e `nobody`) é recusado, assim como bloqueá-la, removê-la
  ou expirar a senha dela. A sua própria conta não pode ser bloqueada nem removida.
- Renomear ou remover um grupo com GID abaixo de 1000, ou um grupo de administradores (`wheel`,
  `sudo`), é recusado; os membros dele ainda podem mudar, mas você não pode se remover de um grupo
  de administradores.
- A condição de administrador é o grupo `wheel` ou o grupo `sudo`. O Control Center adiciona novos
  administradores ao `wheel` quando o sistema o tem, e ao `sudo` caso contrário. Remover a própria
  condição de administrador por uma edição é recusado.

## Exemplos

```sh
# Defina o seu nome de exibição e o avatar após o primeiro login
argvus accounts self name "William Canin"
argvus accounts self avatar ~/Pictures/perfil.png

# Veja os seus grupos e adicione outro usuário ao grupo wheel (exige autorização)
argvus accounts self groups
argvus accounts groups ghost --add wheel

# Administrador: mostre os detalhes de um usuário e altere o nome dele
argvus accounts show ghost
argvus accounts name ghost "Conta Ghost"
```

## Solução de problemas

- `pkexec was not found`: instale `polkit`. O pacote `argvus` depende dele.
- O pedido de autorização não aparece: verifique se há um agente de autenticação polkit rodando na
  sessão. O pacote `argvus` depende de `hyprpolkitagent`, que fornece um.
- Erros de `unix_chkpwd` ao alterar senha: o pacote `pam` fornece o auxiliar.
- Use `argvus accounts --verbose ...` para imprimir no stderr as etapas do comando.

Para notas de desenvolvimento sobre a fronteira de privilégio, veja `docs/internal/argvus-accounts/`.
