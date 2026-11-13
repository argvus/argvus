---
title: 현지화
description: 영어 또는 포르투갈어로 ARGVUS를 사용하세요.
---

`argvus-i18n` 공유 카탈로그를 제공합니다, the `argvus i18n validate` 명령과 명령 QML 모듈 `org.argvus.i18n`· ARGVUS 선박 18의 공용영역 언어: 아랍어, 벵골어, 독어, 영어, 스페인어, 프랑스어, 힌디어, 인도네시아어, 이탈리아어, 일본어, 한국어, 브라질 포르투갈어, 유럽 포르투갈, 러시아, 터키어, Urdu, 베트남어 및 중국어 간체.

이름 * ARGVUS 응용 및 쉘 구성 요소는 공유 카탈로그를 소비합니다. 번역된 문자열은 1 개 이상의 저장소에 의해 사용 될 수 있습니다. 지원되는 아랍어와 Urdu 사용 우측에 왼쪽 배치 GTK· Quickshell 및 터미널 인터페이스.

**Control Center → Preferences → Language & Region → Language**를 사용하여 선택 ARGVUS 인터페이스 언어. 이 글 `ARGVUS_LANGUAGE` 으로 `$XDG_CONFIG_HOME/locale.conf` (일반적으로 `~/.config/locale.conf`). 예를 들어, `ARGVUS_LANGUAGE=pt-PT` 유럽 포르투갈어 선택. 브라질 포르투갈어 (Russian Portuguese)`pt-BR`)는 별도의 옵션입니다. 언어 카탈로그가 설치되어 있습니다. ARGVUS, 그래서 그것에 의존하지 않습니다 `locale-gen`·

제어판은 인터페이스 언어 변경 후 자동으로 다시 시작합니다. 세션 구성에 적용하기 위해 새로운 세션을 시작합니다. 별도 **지역** 설정 쓰기 `LANG` 지역 형식 및 타사 응용 프로그램을 제어합니다. 생성된 시스템 로컬이 필요합니다. `/etc/locale.gen` 지원하다 `locale-gen` 관리자 권한으로. 키보드 레이아웃, 변형 및 콘솔 키 맵은 별도 설정입니다.

## 환경 변수가 변경 될 때

ARGVUS 앱 읽기 `ARGVUS_LANGUAGE` 사용자의 `locale.conf`. 복부인 경우, 그들은 지역을 해결 `LANG` 설치 카탈로그의 값. inherited locale 변수는 명시되지 않습니다. `ARGVUS_LANGUAGE` 이름 * Third-party 응용 프로그램은 시스템 지역 Locale를 계속 사용합니다.

효과적인 값을 검사:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
printf '%s\n' "$LANG"
localectl status
```

여행 정보 `~/.config/locale.conf` 유효한 포함 `ARGVUS_LANGUAGE=<catalog>` 가치. 지역 서식 또는 제3자 응용 언어를 변경하려면 **지역 Locale**를 사용하여 생성된 Locale를 선택합니다. 제어판은 기본 변경 후 재시작됩니다. 로그 아웃 및 다른 세션 구성 요소가 여전히 이전 인터페이스 언어를 보여줍니다. [환경 변수](/docs/reference/environment-variables/) 기술 계약 및 우선.
