---
title: 공식 테마
description: 공식 ARGVUS 테마 카탈로그, 기본 하이라이트 색상, 테마와 그 스티키 또는 스페이스드 모드를 선택하는 방법.
---
이 페이지에는 테마 패키지로 제공되는 공식 테마가 나열되어 있으며, 테마를 선택하는 방법을 설명합니다. 설치 방법은 [테마 설치 Installing themes](/docs/components/argvus-themes/installing-themes/)]를 참조하세요.

## 테마 선택

1. **제어 센터 → 외형 → 테마**를 엽니다.
2. **공식 테마**를 선택합니다.
3. **다크** 또는 **라이트**를 선택합니다.
4. 테마 패밀리를 선택합니다. 테마는 선택하는 즉시 적용됩니다.

선택한 테마는 데스크탑 전체에 한 번에 적용됩니다: 실행 중인 작업 표시줄, 패널, 알림, 창 및 배경 화면의 색상이 로딩 화면 없이, 재시작 없이 변경됩니다. ARGVUS는 테마가 자체 배경 화면과 함께 적용되지 않는 한, 사용자가 선택한 배경 화면을 유지합니다.

키보드에서 `SUPER + SHIFT + T`를 눌러 테마 선택기를 열 수도 있습니다. 화살표 키로 카테고리를 이동하고, `Enter`로 선택하며, `Esc`로 닫습니다.

## 스티키와 간격

각 테마는 두 가지 모드에서 작동합니다. 모드는 테마와 독립적입니다: 테마를 변경해도 현재 모드는 유지되며, 모드를 변경해도 현재 테마는 유지됩니다.

| 모드 | 창 간격 | 외부 간격 | 창 모서리 둥글기 | 작업 표시줄 및 셸 여백 |
| --- | ---: | ---: | ---: | ---: |
| 스티키 | `2` | `0` | `0` | 가장자리에 근접 |
| 간격 | `10` | `18` | `4` | `18` |

모드를 전환하려면 **제어 센터 → 모양 → 모드**를 열고 **스티키** 또는 **간격**을 선택하세요. 이러한 레이아웃 값에 대한 수동 조정은 다음 명시적 테마 선택까지 유지됩니다.

## 다크 테마

| 테마 | 패키지 | 기본 하이라이트 |
| --- | --- | --- |
| 원 다크 | `argvus-theme-one-dark` | `#61AFEF` |
| 드라큘라 | `argvus-theme-dracula` | `#BD93F9` |
| 실버 다크 | `argvus-theme-silver-dark` | `#595959` |
| 슬레이트 다크 | `argvus-theme-slate-dark` | `#7391A5` |
| 유니버스 | `argvus-theme-universe` | `#EEEEEE` |
| 그루브박스 하이 다크 | `argvus-theme-gruvbox-high-dark` | `#D79921` |
| 그루브박스 다크 | `argvus-theme-gruvbox-dark` | `#D4BE98` |
| 로제 파인 | `argvus-theme-rose-pine` | `#C4A7E7` |
| 도쿄 나이트 | `argvus-theme-tokyo-night` | `#7AA2F7` |
| 솔리튜드 | `argvus-theme-solitude` | `#798186` |
| 선셋 | `argvus-theme-sunset` | `#E2BE8A` |
| 매트릭스 | `argvus-theme-matrix` | `#82FB9C` |
| 모노카이 다크 | `argvus-theme-monokai-dark` | `#78DCE8` |
| 레이디 미스트 | `argvus-theme-lady-mist` | `#354B78` |

## 라이트 테마

| 테마 | 패키지 | 기본 하이라이트 |
| --- | --- | --- |
| GitHub Light | `argvus-theme-github-light` | `#0969DA` |
| Solarized Light | `argvus-theme-solarized-light` | `#268BD2` |
| Nord Light | `argvus-theme-nord-light` | `#5E81AC` |
| One Light | `argvus-theme-one-light` | `#4078F2` |
| Everforest Light | `argvus-theme-everforest-light` | `#3A94C5` |
| Catppuccin Latte | `argvus-theme-catppuccin-latte` | `#1E66F5` |
| Gruvbox Light | `argvus-theme-gruvbox-light` | `#458588` |
| Watercolor | `argvus-theme-watercolor` | `#1A639C` |
| Obsidian Light | `argvus-theme-obsidian-light` | `#334A78` |

## 내장 테마

**ARGVUS Dark**와 **ARGVUS Light**는 `argvus-appearance`에서 제공되며 항상 **공식 테마**에서 사용할 수 있습니다. ARGVUS Dark는 기본 하이라이트 `#3590BD`를 사용하며, ARGVUS Light는 `#181818`를 사용합니다.

## 테마가 제어하는 것

테마는 데스크톱 구성 요소에서 사용하는 팔레트와 표면 처리, 기본 강조 색상, 기본 배경화면 연관, 투명도 및 흐림 기본값을 정의합니다. 데스크톱 구성 요소는 자체 복사본을 저장하지 않고 이 상태를 읽기 때문에, 테마 변경은 합성기, 작업 표시줄, 제어판, 잠금 화면, 알림 및 애플리케이션에 동시에 적용됩니다.

기본 강조 색상은 시작점일 뿐입니다. 원하는 6자리 색상으로 바꿀 수 있으며, [강조 색상 사용자 지정](/docs/components/argvus-themes/highlight-color/)]를 참조하십시오.
