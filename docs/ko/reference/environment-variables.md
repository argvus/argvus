---
title: 환경 변수
description: ARGVUS 세션 구성 요소에서 사용되는 환경 계약.
---
세션은 XDG 및 Wayland 환경 통합을 설정하며, 감지될 경우 가상화 호환성 변수를 내보낼 수 있습니다. `ARGVUS_CONFIG_HOME`, `ARGVUS_SYSTEM_CONFIG`, `ARGVUS_BOOTSTRAP`는 세션 및 디스패처 도우미가 구성 루트를 해결하는 데 사용됩니다.

환경 변수를 런타임 계약으로 취급하십시오: 지속적인 재정의를 추가하기 전에 설치된 도우미의 `--help` 또는 소스를 확인하십시오.

파생 상태는 구성 루트가 아닌 상태 루트 아래에 존재합니다. `XDG_STATE_HOME`(보통 `~/.local/state`)는 `argvus/config-projection.json`를 받으며, 투사 매니페스트 `argvus-config`는 재구성되고 `argvus sessionctl reload`가 적용되며, `argvus/session.log`도 적용됩니다. 어느 것도 진리의 출처가 아닙니다.

## 언어 환경

유일한 일반 언어 설정은 `LANG`이며, 위치는 `$XDG_CONFIG_HOME/locale.conf` (기본값 `~/.config/locale.conf`)입니다. 유효한 값이 없으면 ARGVUS는 /etc/locale.conf, PAM 또는 로그인 관리자가 포르투갈어를 제공하더라도 영어(`en-US`)를 선택합니다.  `/etc/locale.conf` `pt_BR.UTF-8`와 같은 값은 카탈로그에서 `pt-BR`로 정규화됩니다.

세션은 데스크탑 구성 요소를 시작하고 systemd 사용자 및 D-Bus 활성화 환경을 가져오기 전에 이 설정을 적용합니다. 상속된 `LANG`, `LC_MESSAGES`, `LANGUAGE`를 대체하고 `LC_ALL`를 지웁니다; 동일 파일에 있는 지역 `LC_*` 설정은 유지됩니다. 영어는 생성 시 `en_US.UTF-8`를 사용하며, 그렇지 않으면 `C.UTF-8`를 사용합니다. 기본적으로 환경 설정 파일은 자동으로 생성되지 않습니다. 언어를 변경한 후에는 로그아웃한 뒤 다시 로그인하십시오.

`session.language`, `argvus/data/language` 및 `argvus/data/internal/language`는 무시된 레거시 데이터입니다. 이들은 가져오거나 재생성되지 않으며, 기존 파일은 유지됩니다. `argvus config set` 및 `patch`는 `session.language`에 대한 새로운 쓰기를 거부하고 호출자를 `locale.conf`로 안내합니다.

진단을 위해 `argvus i18n locale`, `cat "${XDG_CONFIG_HOME:-$HOME/.config}/locale.conf"` 및 `systemctl --user show-environment`를 비교하십시오. `localectl status`는 사용자의 ARGVUS 언어가 아닌 시스템을 설명합니다. `ARGVUS_I18N_DIR`는 언어가 아닌 개발을 위한 카탈로그 디렉터리를 선택합니다.
