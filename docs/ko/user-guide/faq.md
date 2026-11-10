---
title: 자주 묻는 질문
description: ARGVUS, 그 아키텍처 및 일상적인 사용에 대한 일반적인 질문들.
---
이 페이지는 ARGVUS가 무엇인지, 어떻게 배포되는지, 어디에서 구성할 수 있는지에 대한 일반적인 질문에 답합니다. 자세한 절차는 각 답변의 링크를 참조하세요.

## ARGVUS란 무엇인가요?

ARGVUS는 Hyprland를 중심으로 구축된 Wayland 및 Arch Linux용 모듈식 데스크탑 환경입니다. ARGVUS는 Hyprland와 독립적인 데스크탑 환경이 아니며, Hyprland 위에 조립되어 Arch Linux에 맞게 맞춤화된 데스크탑 경험입니다. ARGVUS는 패키지 모음을 통해 그래픽 세션, 셸 표면, 설정 애플리케이션, 외관, 데스크탑 서비스 및 통합 도구를 조정합니다.

Hyprland 구성이나 개인 dotfiles 모음 이상입니다: 지원되는 세션 진입점, 공유 구성 및 외관 상태를 제공하며, 작업 표시줄, 컨트롤 센터, 알림, 네트워킹 및 디스플레이와 같은 구성 요소의 명확한 소유권을 정의합니다. 전체 구성 요소가 어떻게 맞물리는지에 대한 개요는 [introduction](/docs/introduction/)]를 참조하세요.

## ARGVUS는 무료인가요?

예. ARGVUS는 다운로드 및 사용이 무료이며, 핵심 프로젝트 패키지는 [GNU 일반 공중 사용 허가서 버전 3](https://www.gnu.org/licenses/gpl-3.0.html)]에 따라 자유 소프트웨어로 배포됩니다. 이를 통해 사용자는 라이선스 조건 하에 소프트웨어를 실행, 연구, 수정 및 재배포할 자유를 가집니다.

ARGVUS는 서명된 Arch Linux 패키지로 배포됩니다. 소프트웨어 자체는 구매 비용이 없지만, 패키지를 다운로드하는 데에는 일반적인 인터넷 사용료나 호스팅 비용이 발생할 수 있습니다. 제3자가 제공하는 구성 요소와 애플리케이션은 각각 별도의 라이선스를 가질 수 있습니다.

## ARGVUS는 리눅스 배포판인가요?

아니요. ARGVUS는 Arch Linux 위에 설치되는 데스크톱 환경 계층입니다. Arch Linux는 운영 체제, 커널, 패키지 관리자 및 기본 패키지를 제공하며, Hyprland는 Wayland 합성을 제공하고, ARGVUS는 데스크톱 세션과 그 통합 구성 요소들을 조정합니다.

지원되는 설치 모델에 대해서는 [installation](/docs/getting-started/installation/)]를, 아키텍처적 차이에 대해서는 [What ARGVUS is—and is not](/docs/introduction/#what-argvus-isand-is-not)]를 참조하세요.

## 왜 배포판 대신 데스크톱 환경인가요?

ARGVUS를 데스크탑 환경으로 유지하면 사용자가 시스템의 나머지 부분을 더 많이 제어할 수 있습니다. 일반적으로 배포판은 커널, 패키지 선택, 저장소, 기본값 및 시스템 정책을 포함한 완전한 기준선을 선택하고 유지 관리합니다. ARGVUS는 이러한 광범위한 결정을 사용자와 기본 Arch Linux 설치에 맡기면서 데스크탑 경험에 집중합니다.

이 모델은 또한 프로젝트를 모듈화된 상태로 유지합니다: 세션 관리, Hyprland 구성, 외형, 작업 표시줄, 설정 및 제공자는 하나의 데스크탑으로 함께 작동하면서 별도로 개발되고 패키징될 수 있습니다.

## 지원되는 시스템은 무엇인가요?

현재 지원되는 기본 시스템은 Arch Linux입니다. ARGVUS는 Arch Linux용으로 패키징되어 있으며, [설치 요구사항](/docs/getting-started/installation/requirements/)]에 설명된 Wayland, Hyprland 및 시스템 구성요소에 의존합니다.

다른 배포판에서는 일부 동일한 업스트림 구성 요소를 제공할 수 있지만, 그렇다고 해서 그곳에서 전체 ARGVUS 패키지 세트나 세션 통합이 지원되는 것은 아닙니다.

## ARGVUS를 어떻게 설치합니까?

[설치 가이드](/docs/getting-started/installation/)]를 따르세요. 이 가이드는 요구 사항, 서명된 ARGVUS 패키지 저장소 구성 방법, 패키지 세트 설치 방법 및 첫 번째 세션 시작 방법을 설명합니다.

ARGVUS는 조정된 Arch 패키지 세트로 제공됩니다. `argvus` 패키지가 주요 설치 대상이며, 이 패키지가 조정하는 데스크톱 구성 요소들을 포함합니다.

## ARGVUS 설정을 어떻게 변경합니까?

**Control Center**를 사용하여 테마, 배경화면, 효과, 레이아웃, 글꼴, 키보드, 입력, 언어, 지역 및 기본 애플리케이션을 포함한 지속적인 데스크탑 구성을 설정할 수 있습니다. 가장 효율적인 열기 방법은 `SUPER + Alt + C` 단축키를 사용하거나, `SUPER + D`로 실행기에서 **ARGVUS Control Center**를 입력하는 것입니다. 또한 `argvus`을 입력하여 ARGVUS 애플리케이션 목록을 볼 수도 있습니다.

추가 옵션으로, 터미널에서 다음을 사용하여 열 수 있습니다:

```sh
argvus --control-center
```
`argvus control-center`, 예를 들어 `argvus control-center appearance themes`로 집중 영역을 열 수도 있습니다. 설치된 버전에서 가능한 경로를 보려면 `argvus control-center --help`를 실행하세요.

현재 상태와 볼륨, 밝기, 네트워크, 알림, 전원 및 세션 제어와 같은 자주 사용하는 동작은 **제어판**을 사용하세요. 이는 두 번째 구성 데이터베이스가 아닙니다. 구분에 대해서는 [설정하는 위치](/docs/user-guide/where-to-configure/)]와 [제어 센터](/docs/components/argvus-control-center/)]를 참조하세요.

## ARGVUS 설정은 어디에 저장되나요?

사용자 소유 구성은 `$XDG_CONFIG_HOME/argvus`(보통 `~/.config/argvus`) 아래에 저장됩니다. 표준 논리 구성은 그곳에 유지되며, Hyprland, Waybar 및 텔레메트리 위젯, GTK, Qt6ct, Quickshell, Dunst, Hyprlock, Yazi, Superfile, 터미널, 글꼴, 효과, 달력, 입력, 키보드, 전원 및 이동식 장치용 파생 소비자 파일은 `~/.config/argvus/data/generated/` 아래에 존재합니다. 해당 트리는 `argvus-config`에 의해서만 작성되며, 표준 구성에서 다시 빌드됩니다; `argvus-appearance`는 외관 변경 사항을 커밋하고, 자체적으로 표준 구성을 읽을 수 없는 외부 소비자만 조정합니다. Rofi는 예외입니다: `argvus launcher`는 구성과 테마를 패키징하며, 사용자가 편집한 사본은 `~/.config/argvus/data/rofi/` 아래에 존재합니다.

생성된 파일을 영구 구성 방법으로 편집하지 마십시오. 사용자 지정하려는 구성요소에 대해서는 Control Center 또는 문서화된 기본 재정의 경로를 사용하십시오. [configuration files](/docs/reference/configuration-files/)] 및 [configuration and state](/docs/developer-guide/architecture/configuration-and-state/)]를 참조하십시오.

## 변경 사항이 적용되지 않으면 어떻게 해야 합니까?

먼저 변경 사항이 소유 페이지에 저장되었거나 적용되었는지 확인하십시오. 그런 다음 영향을 받는 구성요소가 세션 또는 서비스 재시작이 필요한지 확인하십시오. 커밋된 변경 사항은 `argvus-config`에 의해 소비자 파일에 반영되지만, 반영한다고 해서 모든 실행 중인 소비자가 이를 다시 로드하는 것이 보장되지는 않습니다. `argvus sessionctl reload`는 이미 커밋된 생성물을 다시 투사하는 대신 투사 매니페스트에서 적용하며, 정규 상태가 변경되지 않은 경우 그래픽 재로드를 건너뜁니다.

설치, 세션 시작, 그래픽, 디스플레이, Hyprland, 테마 및 배경화면과 관련하여 관련 [문제 해결 가이드](/docs/user-guide/troubleshooting/)]를 사용하세요. 문제를 보고할 때는 구성 요소, 변경된 설정, 사용된 명령어나 페이지, 관련 상태 또는 진단 명령의 출력을 포함하세요.

## 어디서 더 배울 수 있나요?

- 권장 첫 세션 흐름에 대한 [첫 번째 구성](/docs/user-guide/getting-started/)].
- 기능별 지침에 대한 [사용자 가이드](/docs/user-guide/)].
- 명령, 경로, 서비스 및 구성 계약에 대한 [참조](/docs/reference/)].
- 아키텍처, 패키지 소유권 및 기여 워크플로우에 대한 [개발자 가이드](/docs/developer-guide/)].
