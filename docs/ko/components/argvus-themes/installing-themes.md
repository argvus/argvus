---
title: 테마 설치
description: pacman으로 공식 ARGVUS 테마 패키지를 설치하십시오.
---
공식 테마는 Arch Linux 패키지로 배포됩니다. 각 테마는 `argvus-theme-<THEME_NAME>`라는 이름을 가진 독립적인 패키지입니다.

## 테마를 구할 곳

모든 공식 테마의 소스는 ARGVUS 프로젝트 조직에 있습니다:

- 각 테마는 `argvus-theme-<THEME_NAME>`라는 이름의 리포지토리이며(예: `argvus-theme-dracula`), 같은 이름으로 패키지로 빌드됩니다.

테마 패키지는 `argvus-appearance`에 의존합니다. 먼저 [Installation](/docs/getting-started/installation/)에 설명된 대로 ARGVUS를 설치하여 외형 구성 요소가 존재하도록 하십시오.

## 단일 테마 설치

모든 공식 테마를 한 번에 설치하려면 `argvus-themes` 메타패키지를 설치하십시오:

```bash
sudo pacman -S argvus-themes
```
모든 공개된 `argvus-theme-*` 패키지에 따라 다릅니다. ARGVUS Dark와 ARGVUS Light는 이미 `argvus-appearance`에서 배송되었습니다.

패키지 이름으로 테마 하나를 설치하세요:

```bash
sudo pacman -S argvus-theme-<THEME_NAME>
```
예를 들어:

```bash
sudo pacman -S argvus-theme-dracula
sudo pacman -S argvus-theme-rose-pine
```
패키지는 테마 매니페스트 `/usr/share/argvus/themes/<theme-id>/theme.toml`(이름, 카테고리, 배경화면 및 팔레트, 선택적으로 `overrides/`)과 테마의 배경화면을 `/usr/share/backgrounds/argvus/` 아래에 설치합니다. 테마가 적용되면 각 구성 요소의 파일은 팔레트에서 렌더링됩니다. 홈 디렉토리에 파일을 쓰지 않으며 현재 테마를 변경하지 않습니다. 이후 제어 센터에서 선택하고, [공식 테마](/docs/components/argvus-themes/official-themes/)를 확인하세요.

## 여러 테마 설치

모든 테마를 설치하는 메타 패키지는 없습니다. 한 번의 명령으로 원하는 테마를 설치하세요:

```bash
sudo pacman -S argvus-theme-dracula argvus-theme-sunset
```
## 사용 가능한 패키지

아래 패키지는 오늘 공개된 공식 테마 패키지입니다.

**다크 테마**

| 패키지 | 테마 |
| --- | --- |
| `argvus-theme-one-dark` | 원 다크 |
| `argvus-theme-dracula` | 드라큘라 |
| `argvus-theme-silver-dark` | 실버 다크 |
| `argvus-theme-slate-dark` | 슬레이트 다크 |
| `argvus-theme-universe` | 유니버스 |
| `argvus-theme-gruvbox-high-dark` | 그루브박스 하이 다크 |
| `argvus-theme-gruvbox-dark` | 그루브박스 다크 |
| `argvus-theme-rose-pine` | 로제 파인 |
| `argvus-theme-tokyo-night` | 도쿄 나이트 |
| `argvus-theme-solitude` | 솔리튜드 |
| `argvus-theme-sunset` | 선셋 |
| `argvus-theme-matrix` | 매트릭스 |
| `argvus-theme-monokai-dark` | 모노카이 다크 |

**라이트 테마**

| 패키지 | 테마 |
| --- | --- |
| `argvus-theme-github-light` | GitHub 라이트 |
| `argvus-theme-solarized-light` | Solarized 라이트 |
| `argvus-theme-nord-light` | Nord 라이트 |
| `argvus-theme-one-light` | One 라이트 |
| `argvus-theme-everforest-light` | Everforest 라이트 |
| `argvus-theme-catppuccin-latte` | 캣푸친 라떼 |
| `argvus-theme-gruvbox-light` | Gruvbox 라이트 |
| `argvus-theme-watercolor` | 수채화 |

내장된 **ARGVUS Dark** 및 **ARGVUS Light** 테마는 별도의 패키지가 필요하지 않습니다; `argvus-appearance`와 함께 제공됩니다.

## 설치된 항목 확인

설치된 테마 패키지를 나열하려면:

```bash
pacman -Qs argvus-theme
```
제어 센터를 열고 **모양 → 테마 → 공식 테마**로 이동하여 현재 선택기에서 사용할 수 있는 테마를 확인하세요. 테마가 설치되었지만 해당 목록에서 누락된 것은 패키징 문제입니다. [테마 가이드](/docs/components/argvus-themes/).

## 나만의 테마 추가

테마는 매니페스트이므로 자신만의 테마에는 패키지가 필요하지 않습니다. 만들기
`~/.local/share/argvus/themes/<id>/theme.toml`, 여기서 디렉터리 이름은
테마 ID(소문자, 숫자, 단일 하이픈):

```toml
schema = 2
id = "mist"
name = "Mist"
category = "dark"
wallpaper = "/usr/share/backgrounds/argvus/argvus-dark.jxl"

[palette]
bg = "#101418"
fg = "#D8DEE9"
accent = "#88C0D0"
```
오직 `bg`, `fg` 및 `accent`만 필요합니다: 다른 모든 색상은 이들로부터 기본이 파생되며, 매니페스트는 이를 변경하기 위해서만 정의합니다. 확인한 후, 다른 테마처럼 적용하세요:

```bash
argvus theme check ~/.local/share/argvus/themes/mist
argvus theme apply mist
```
`argvus theme list`는 공식 테마 옆에 사용자 테마를 표시합니다(같은 ID를 가진 공식 테마가 우선합니다). 사용자 테마는 테마 메뉴(`SUPER + Shift + T`)에 나타납니다; 로그인 전 화면에서는 홈 디렉터리를 읽을 수 없으므로, 사용자 테마가 활성화된 동안 마지막으로 적용한 공식 테마를 유지합니다.

이전 ARGVUS 버전이 `~/.config/argvus/data/<app>/themes/` 아래 작성한 테마별 복사본은 더 이상 읽히지 않습니다. 업데이트 후 첫 번째 테마 전환 때 이를 `~/.config/argvus/data/backups/theme-copies-<time>/`로 이동합니다; 편집한 파일을 유지하려면, `overrides/` 아래에 자신의 테마에 넣고 매니페스트의 `[overrides]` 테이블에 이름을 지정하세요.

## 테마 제거

단일 테마 패키지를 제거하려면:

```bash
sudo pacman -R argvus-theme-<THEME_NAME>
```
테마 패키지를 제거해도 가져온 프로필은 삭제되지 않습니다. 제거하려는 테마가 현재 적용된 테마인 경우 제거하기 전에 제어 센터에서 다른 테마를 선택하십시오.
