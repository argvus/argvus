---
title: 프로젝트
description: argvus 프로젝트를 통해 프로젝트 선택기와 프로젝트 구성.
---
`argvus projects`은 프로젝트를 나열하고 엽니다. 이는 `argvus` 패키지의 진행 중인 명령이며, `SUPER + O`은 선택기를 엽니다.

## 명령어

```text
argvus projects                    pick a project (SUPER + O)
argvus projects open SELECTOR      open a project by position, full path or unique name
argvus projects add [--root] DIR   add a project, or a root holding projects
argvus projects remove DIR         remove a project or root
argvus projects list               print configured roots and paths
argvus projects pick [PROMPT]      print the path picked from the list, for other tools
```
## 프로젝트란 무엇인가

프로젝트는 구성 파일(`/projects/paths`)에 나열된 디렉터리이거나, `/projects/roots`에 나열된 루트의 하위 디렉터리입니다. 최대 아홉 개의 프로젝트가 나열됩니다: `SUPER + CTRL + N` 바로 가기 번호들입니다. 프로젝트를 열면 해당 프로젝트의 Hyprland 작업 공간으로 포커스가 이동합니다. 프로젝트를 처음 열 때, ARGVUS는 터미널을 시작하고 그 안에서 `/projects/editor`에 설정된 편집기를 실행합니다.

구성은 `argvus config`를 통해 읽고 쓰므로, 제어 센터와 셸이 동일한 목록을 확인할 수 있습니다.

## 프로젝트 작업 공간과 작업 표시줄

프로젝트는 특수 워크스페이스가 아닌 일반적으로 명명된 Hyprland 워크스페이스를 사용합니다. `SUPER + O`는 프로젝트 번호, 디렉터리 및 직접 `SUPER + CTRL + N` 바로가기를 보여줍니다. 두 진입점은 동일한 워크스페이스에 초점을 맞춥니다. 열린 워크스페이스만 작업 표시줄에 나타나며, `Project: name` 레이블, 활성 상태 및 전체 경로와 바로가기가 포함된 툴팁이 표시됩니다. 워크스페이스 버튼을 클릭하면 해당 워크스페이스로 돌아갈 수 있습니다. 작업 표시줄은 워크스페이스의 모니터를 따릅니다.

정식 디렉터리 경로는 안정적인 워크스페이스 정체성을 결정합니다. 목록 순서를 변경하면 워크스페이스가 아니라 바로가기가 변경됩니다. 동일한 디렉터리 이름을 가진 프로젝트는 별도로 유지되며, 동일한 디렉터리에 대한 심볼릭 링크는 중복 제거됩니다. 터미널 및 편집기 실행은 명시적으로 프로젝트의 워크스페이스를 대상으로 합니다. 기존 프로세스/창을 재사용하는 편집기는 해당 애플리케이션의 자체 워크스페이스 동작을 유지할 수 있습니다.

업데이트 이전에 열린 작업 공간은 기존 창과 기본 이름을 유지합니다. 업데이트 후 프로젝트를 열면 새 경로 기반 식별자를 사용하게 되며, 편리할 때 이전 프로젝트 창을 닫으십시오. 오래된 이름은 디렉터리를 신뢰할 수 있게 식별하지 못하므로 자동 연결이나 이동은 시도되지 않습니다. 중복된 디렉터리 이름이 있는 경우, 모호한 이름 대신 선택기, 숫자 또는 전체 정규 경로를 사용하십시오.

프로젝트를 열면 구성된 프로젝트 목록과 번역에서 `$XDG_RUNTIME_DIR/argvus/project-workspaces.json`가 원자적으로 다시 생성됩니다. 이는 작업 공간 라벨 및 툴팁용 세션 메타데이터이며 설정이 아니므로 수정하지 마십시오. 목록이나 언어를 변경한 후 프로젝트를 다시 열어 라벨을 새로 고치십시오. 이 통합을 위해 `argvus` 및 `argvus-waybar`와 함께 `argvus-taskbar` 설정도 업데이트하십시오.
