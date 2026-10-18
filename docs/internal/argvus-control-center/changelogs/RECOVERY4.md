# Recovery 4 — Boot funcional e seguro

## Estado e detecção

`argvus-control-center-boot` coleta firmware, ESP, bootloader, kernels,
initramfs, Plymouth, Secure Boot e entries em um `JobManager` de background.
systemd-boot só é considerado quando há loader/entries em uma ESP conhecida;
GRUB depende de configuração efetiva detectável. Quando as evidências entram em
conflito, o bootloader fica indeterminado e as mutações são recusadas.

Kernels, entries, presets e temas são listas navegáveis. Detalhes de kernel e
entry preservam a semântica de Esc/Left do chrome compartilhado.

## Operações

As ações passam por `SystemSettingsOperation` como operações `boot` enumeradas:

- `systemd-default` e `systemd-timeout` usam `bootctl`/`loader.conf` apenas
  depois de validar entry e ESP;
- `grub-timeout` e `grub-cmdline` alteram somente `/etc/default/grub`;
- `grub-regenerate` encontra `grub.cfg` e `grub-mkconfig` sem aceitar path da UI;
- `initramfs-regenerate` executa somente `mkinitcpio -P`;
- `plymouth-theme` aceita apenas nomes de diretório presentes em
  `/usr/share/plymouth/themes`.

Nenhuma operação usa shell, `sudo` ou comando root arbitrário. A validação é
repetida no helper privilegiado.

## Backup e escrita

Arquivos alterados são copiados para backups não sobrescritos com o sufixo
`.argvus-backup-<timestamp>-<counter>`. A escrita usa arquivo temporário com as
permissões originais, `sync_all` e `rename` atômico. Symlinks de configuração
são recusados. Falha no backup interrompe a operação antes da escrita.

## Plymouth e initramfs

Aplicar tema é uma sequência: validar tema, alterar tema e regenerar initramfs.
Se a primeira etapa funcionar e a segunda falhar, a UI mantém o erro como
partial failure/warning e não informa sucesso total.

## Limitações reais

- GRUB entries/submenus não são selecionáveis enquanto não houver mapeamento
  confiável; default arbitrário não é oferecido.
- Secure Boot é somente leitura.
- UKI é detectado quando o layout é reconhecível, mas não há editor completo de
  UKI.
- Operações reais de mutação não são executadas pela suíte automática; são
  cobertas por validação, mocks e arquivos temporários.
