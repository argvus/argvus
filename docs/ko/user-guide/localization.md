---
title: 현지화
description: 영어 또는 포르투갈어로 ARGVUS를 사용하세요.
---
`argvus-i18n`은 공유 카탈로그, `argvus i18n validate` 명령 및 QML 모듈 `org.argvus.i18n`를 제공합니다. ARGVUS는 현재 영어 및 포르투갈어 카탈로그를 제공합니다.

네이티브 애플리케이션과 셸 구성 요소는 공유 카탈로그를 사용합니다. 따라서 번역된 문자열은 하나 이상의 저장소에서 사용될 수 있습니다.

**Control Center → Locale & Region**을 사용하여 언어, 지역 설정, 생성된 시스템 로케일, 시간대, 날짜/시간 및 키보드를 구성하십시오. 현재 ARGVUS 카탈로그는 영어 및 포르투갈어를 제공하며, 이는 운영 체제가 생성한 시스템 로케일 목록과는 별개입니다.

표시 언어를 변경하면 ARGVUS 환경 설정이 저장되고 `argvus-control-panel.service`가 자동으로 재시작되므로 제어판에 변경 사항이 즉시 반영됩니다. 다른 애플리케이션과 셸 화면은 세션을 다시 불러오거나 새 세션이 필요할 수 있습니다. `/etc/locale.gen` 항목을 생성하거나 변경하려면 권한이 필요하며 `locale-gen`를 실행합니다. 생성되지 않은 로케일을 선택하면 거부됩니다. 키보드 레이아웃, 변형 및 콘솔 키맵은 별도의 설정이며 인터페이스 언어와 혼동해서는 안 됩니다.

## 환경 변수로 인해 변경 사항이 무시될 때

ARGVUS 애플리케이션은 먼저 지속적인 ARGVUS 언어 파일에서 언어를 감지한 후, `LC_ALL`, `LC_MESSAGES` 및 `LANG`를 확인합니다. 따라서 로컬 환경 변수 때문에 제어 센터가 언어 변경을 무시하는 것처럼 보일 수 있으며, 특히 사용자 서비스나 그래픽 세션이 이전 환경을 유지할 경우 그렇습니다.

유효한 값을 확인하려면:

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```
만약 `~/.config/locale.conf`가 존재하고 `LANG`, `LC_ALL` 또는 `LC_MESSAGES`를 정의하면, 해당 값을 다시 도입하지 않도록 일시적으로 이름을 변경하세요. 삭제하지 말고 백업을 유지하세요:

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```
**제어 센터 → 지역 및 언어**에서 언어를 다시 적용하세요. 제어판은 자동으로 다시 시작됩니다. 다른 구성 요소가 여전히 이전 언어를 표시하면 로그아웃 후 다시 로그인하여 그래픽 세션과 `systemd --user`이 새 환경을 받도록 하세요. 그 후 진단 명령을 반복 실행하세요. 기술 계약 및 우선순위에 대한 환경 변수는 [Environment variables](/docs/reference/environment-variables/)를 참조하세요.
