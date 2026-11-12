---
title: 외모
description: ARGVUS 구성 요소가 공유하는 시각적 언어를 구성하십시오.
---
외관 시스템은 하나의 테마 상태를 Hyprland, GTK, Qt, Waybar, Quickshell, Rofi, Dunst, Hyprlock 및 ARGVUS 응용 프로그램에 적용합니다.

## 테마 계열

아래 23개의 계열은 현재 테마 페이로드에 의해 정의됩니다. 색상 열은 계열의 기본 활성-테두리/강조 색상이며, 사용자는 다른 유효한 RGB 강조 색상을 선택할 수 있습니다.

| 계열 | 강조 색상 초기화 | 기본 배경 | 모드 |
| --- | --- | --- | --- |
| One Dark | `#61AFEF` | `#282C34` | 고정, 간격 |
| Dracula | `#BD93F9` | `#282A36` | 고정, 간격 |
| ARGVUS Dark | `#3590bd` | `#111316` | 고정, 간격 |
| Dark Silver | `#595959` | `#111316` | 고정, 간격 |
| Dark Slate | `#7391a5` | `#2f3541` | 고정, 간격 |
| Dark Universe | `#eeeeee` | `#000000` | 고정, 간격 |
| ARGVUS Dark Gruvbox High | `#D79921` | `#282828` | 고정, 간격 |
| ARGVUS Dark Gruvbox | `#D4BE98` | `#282828` | 고정, 간격 |
| 아르그부스 라이트 | `#181818` | `#f7f7f7` | 끈끈한 간격 |
| GitHub 라이트 | `#0969DA` | `#FFFFFF` | 끈끈한 간격 |
| 캣푸친 라떼 | `#1E66F5` | `#EFF1F5` | 끈끈한 간격 |
| ARGVUS 라이트 Gruvbox | `#458588` | `#FBF1C7` | 끈끈한 간격 |
| 로제파인 | `#C4A7E7` | `#191724` | 끈끈한 간격 |
| 도쿄나이트 | `#7AA2F7` | `#1A1B26` | 끈끈한 간격 |
| 고독 | `#798186` | `#101315` | 끈끈한 간격 |
| 일몰 | `#E2BE8A` | `#0F0F0F` | 끈끈한 간격 |
| 매트릭스 | `#82FB9C` | `#0B0C16` | 끈끈한 간격 |
| 모노카이 다크 | `#78DCE8` | `#2D2A2E` | 끈끈한 간격 |
| 레이디 미스트 | `#354B78` | `#131313` | 끈끈한 간격 |
| 하나의 빛 | `#4078F2` | `#FAFAFA` | 끈끈한 간격 |
| 에버포레스트 라이트 | `#3A94C5` | `#FDF6E3` | 끈끈한 간격 |
| 솔라라이즈 라이트 | `#268BD2` | `#FDF6E3` | 끈끈한 간격 |
| 노드 라이트 | `#5E81AC` | `#ECEFF4` | 끈끈한 간격 |
| 수채화 | `#1A639C` | `#E9E3D3` | 끈끈한 간격 |

표에 표시된 색상은 해당 패밀리가 선택되었거나 **테마 기본값으로 재설정**이 사용될 때 `config.json`에 기록되는 값입니다. 수동으로 선택한 6자리 RGB 강조 색상은 다음 테마 전환 시까지 적용되며, 그때 선택한 테마의 기본값으로 대체됩니다.

**Sticky**는 컴팩트 레이아웃으로, 타일 창은 작은 내부 간격을 사용하고 외부 간격은 0이며, 작업 표시줄은 화면 가장자리 근처에 위치합니다. **Spaced**는 더 큰 간격, 둥근 모서리, 그림자, 작업 표시줄과 셸 표면 주변의 여백을 추가합니다.

기존 `float` 레이아웃 값과 `-float` 테마 식별자는 Spaced의 레거시 이름으로 인식됩니다. 새로운 레이아웃 값과 확인된 테마 식별자는 `spaced` 및 `-spaced`를 사용합니다.

모드는 지오메트리와 표면 처리를 변경하며, 여섯 번째 색상 패밀리를 생성하지 않습니다. 선택 관련 내용은 [Themes](/docs/user-guide/appearance/)를, 작업 표시줄 및 창 배치 관련 내용은 [Desktop](/docs/user-guide/desktop/)를 참조하십시오.

- [테마](/docs/user-guide/appearance/)
- [배경화면](/docs/argvus-appearance/wallpapers/)
- [폰트 및 아이콘](/docs/argvus-appearance/fonts-and-icons/)
- [효과](/docs/argvus-appearance/effects/)

컨트롤 센터는 사용자 대면 설정을 노출합니다. 구현은 `argvus-appearance`, `argvus-session`, 셸 구성 요소 및 자산 패키지에서 공유됩니다. [외형 아키텍처](/docs/developer-guide/subsystems/appearance-and-themes/)를 참조하십시오.

## 창 흔들림

**바탕화면 & 입력 → Hyprland → 애니메이션 → 창**을 열고
**흔들림** 컨트롤을 사용하십시오. `wobble`을 검색하거나 페이지를 직접 열 수 있습니다.

```sh
argvus control-center hyprland animations windows
```
Wobble은 기본적으로 비활성화되어 있습니다. 그 ARGVUS 기본값은 강성 `100`, 감쇠 `10`, 질량 `1`, 강도 `1`, 그리고 메쉬 해상도 `8`입니다. 강성과 질량은 `0.0001–1000`를 허용하며, 감쇠 `0–1000`, 강도 `0–2`, 그리고 메쉬는 정수 `2–32`를 허용합니다. 메쉬 해상도를 높이면 렌더링 비용이 증가합니다. 화살표 키로 값을 조정하거나 Enter를 눌러 입력할 수 있습니다. **Wobble 기본값 복원**은 초안을 변경하며; **적용**은 모든 보류 중인 애니메이션 편집을 저장하고 세션을 재시작하지 않고 Hyprland를 한 번 재로드합니다. Animations 내에서 돌아가면 공유된 초안이 유지됩니다.

글로벌 Animations 스위치는 개별 설정을 유지하면서 Wobble을 억제합니다. 정규 값은 ARGVUS 상태의 `effects.wobble_*` 아래에 있으며; `hypr/state.lua`는 생성되며 수동으로 편집해서는 안 됩니다.

이것은 Hyprland **0.57.0 이상**을 필요로 하며, `argvus-hyprland`의 패키지 의존성에 의해 강제됩니다. 버전 0.56은 이 구성에서 지원되지 않습니다. 강도 최대치는 [출시된 0.57.0 소스](https://github.com/hyprwm/Hyprland/blob/v0.57.0/src/config/values/ConfigValues.cpp)를 따르며, 이는 `2`로 제한되지만, 더 최신 문서에서는 `3`로 기재될 수 있습니다.
