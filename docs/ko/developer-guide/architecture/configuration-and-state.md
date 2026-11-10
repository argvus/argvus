---
title: 구성 및 상태
description: 공유 상태, 오버라이드 및 생성된 런타임 파일.
---
일반 사용자 루트는 `$XDG_CONFIG_HOME/argvus`입니다(보통 `~/.config/argvus`). 관리되는 논리적 환경설정은 `config.json`에 저장됩니다. 테마, 강조색, 배경화면, 간격 및 테두리 환경설정이 그곳에서 표준입니다; 도트 파일들은 현재 Hyprland Lua 및 셸 사용자가 `data/`로 이전하는 동안 호환성 유도 투영으로 남아 있습니다.

`argvus-config`는 `generated/` 아래의 유일한 작성자입니다. 모든 ARGVUS 소유 소비자 파일은 Hyprland, GTK, Qt6ct, Waybar, Quickshell, Dunst, Hyprlock, Waybar 작업 표시줄 및 위젯-원격 측정 프로필, Yazi 및 Superfile 구성 트리, 터미널 및 Qt 프로필 트리, 글꼴 관리 블록 및 이동식 장치 스타일시트를 포함하여 `config.json`의 투영입니다. 구성 요소는 명시적 사용자 오버라이드를 해결한 다음 생성된 출력, 그 다음 패키지 기본값을 해결합니다. Rofi는 투영이 아닙니다: `argvus-launcher`가 해당 파일을 소유하며, 사용자는 `data/rofi/` 아래에서 파일을 직접 복사합니다. 일반적인 테마/재로드 투영은 생성되거나 패키징된 파일을 사용자 오버라이드로 승격시키지 않습니다. 문서와 도구는 사용자가 생성된 출력을 편집하도록 요구하기보다는 논리적 소스를 식별해야 합니다.

`argvus-appearance`는 따라서 테마와 액센트 오케스트레이션을 소유하며, `argvus-config`는 정규 상태와 구조화된 투영을 소유합니다. `argvus theme-switch`(`argvus config theme-apply` 트랜잭션)와 `argvus config accent`는 가능한 경우 `argvus-config`를 통해 검증, 잠금 및 커밋하며, 직접 `config.json`를 읽을 수 없는 호환성 소비자를 조정합니다. 이러한 어댑터는 기본 Qt6ct 파일, GTK `settings.ini` 및 `gsettings` 힌트, 터미널 및 시스템 모니터 애플리케이션, foot, superfile 자체 구성 디렉토리 및 사전 인증 그리터입니다. 새로운 어댑터가 필요한 도우미는 절대 `generated/` 안이 아니라 그곳에 추가해야 합니다.

런타임 소유권은 의도적으로 책임별로 나누어져 있습니다: Rust는 정규 스토어, 스키마 검증, 마이그레이션, 경로 모델, 락, 원자적 쓰기, 프로젝션 해시 및 구조화된 렌더러를 소유합니다. Shell은 `hyprctl` 및 자산 소유 테마 스크립트와 같은 외부 소비자를 위한 작은 호환성 진입점이나 어댑터로만 남아 있습니다. Python은 테스트, 감사 및 개발 보조 기능에 예약되어 있으며, 중요한 데스크탑 런타임의 일부가 아닙니다.

`argvus config apply`는 재구축 경계입니다(`project`는 호환성 철자로 남습니다). 이는 정규 문서가 변경되지 않아도 누락된 출력을 재생성하고, 스테이징에서 프로젝터를 렌더링하며, 모든 프로젝션이 성공한 후에만 `data/` 트리를 원자적으로 교체합니다. 세션 수명 주기는 `argvus sessionctl`가 여전히 소유하며, 해당 매니페스트를 사용하여 목표로 하는 리로드를 계획합니다.

호출자가 이미 세대를 커밋했을 때 — `apply-theme`, 악센트 변경, `set`/`patch` 또는 위젯-텔레메트리 블록 변경 — `reload`는 그 세대를 다시 투영하는 대신 적용합니다. 이는 매니페스트에서 계획을 읽고 목표 소비자를 복원하며, 매니페스트가 없거나 미처리 작업이 없을 때만 투영합니다. 그 시점에서 다시 투영하는 것은 동일한 트리를 재구성하고 변형이 방금 생성한 계획을 버리는 결과가 될 것입니다.

매니페스트는 또한 운영 세대, 유효 구성 해시, 변경된 도메인 및 생성된 파일 목록을 기록합니다. `data/internal/generation.json`는 일회성 로컬 투영 마커이며 사용자 환경설정이 될 수 없습니다. `argvus config status`는 이러한 기록을 해결된 구성과 비교하고 누락되거나 추가된 생성 파일을 보고합니다.

돌연변이는 동일한 엔진 경계를 따릅니다: `UserConfig`는 `NextUserConfig`로 복제되고, 검증되며, `EffectiveConfig`로 해결되고, 투영된 후에만 지속됩니다. `.config-transaction.json`는 운영 복구 상태입니다; `argvus config recover`는 `data/`와 `config.json` 사이의 게시 중에 발생한 충돌 후 이전 정준 문서를 결정론적으로 다시 투영합니다. 커밋 후에만 다시 로드가 요청되며, 다시 로드 실패는 일관된 지속 구성의 롤백을 발생시키지 않습니다.

## 해결 모델

```text
config.json logical model
    ↓
ARGVUS native compatibility state and per-app override
    ↓
ARGVUS generated runtime file
    ↓
/usr/share/argvus packaged default
    ↓
upstream default
```
정확한 후보는 구성 요소마다 다르지만 원칙은 일관됩니다: 사용자의 재정의가 우선하며, 생성된 출력은 파생 상태이고, 패키지 파일은 읽기 전용 입력으로 남습니다. 세션은 `ARGVUS_CONFIG_HOME` 및 `ARGVUS_SYSTEM_CONFIG`를 내보내므로 헬퍼와 사용자 서비스가 동일한 루트를 해결합니다.

재로드 복구는 `recover → ensure → migrate → apply → runtime reload`를 따르며, 들어오는 매니페스트가 이미 커밋된 세대를 설명하는 경우 투영 단계는 건너뜁니다. `ensure`는 프로필이 없는 경우에만 스키마 기본값에서 `config.json`를 구체화하고, 명시적인 `false`, `0` 또는 사용자 정의 값을 대체하지 않고 결측 필드를 채웁니다. 기존의 잘못된 문서는 조용히 대체되지 않고 오류로 남습니다.

지속적인 변이는 `ChangedDomains`를 통해 해결된 이전 구성과 새로운 구성을 비교합니다. 커밋 후, `argvus sessionctl`를 통해 한 번의 재로드 요청이 전송됩니다; 재로드 실패는 보고되고 기록되지만 커밋된 구성은 롤백되지 않습니다. 세션 시작 시 부트스트랩 구현 및 투영 전에 복구가 수행되므로, 소비자는 포기된 트랜잭션을 기준으로 시작하지 않습니다.

| 상태 | 목적 |
| --- | --- |
| `config.json` | 표준 관리 논리 환경 설정; `generated/` 아래 모든 것의 단일 출처. |
| `.active-theme`, `.accent-color`, `.accent-custom` | 레거시 마이그레이션 입력; 런타임 소스가 아니며 재생성되지 않음. |
| `.wallpaper-custom` | `appearance.wallpaper`용 레거시 마이그레이션 입력; 런타임 소스가 아님. |
| `.spaces`, `.borders` | 레거시 레이아웃 마이그레이션 입력; 표준 값은 config.json의 `layout`에 저장됨. |
|  `config.json` `fonts.conf` | 정규 폰트 선택의 호환성 투영. |
| `generated/effects` | 정규 효과의 호환성 투영; 일회용. |
| `$XDG_STATE_HOME/argvus/config-projection.json` | 투영 매니페스트: 변경된 섹션, 영향받는 소비자 및 재로드 필요 여부. `reload`에서 읽음; 진실의 근원은 아님. |
| `$XDG_STATE_HOME/argvus/widget-telemetry-blocks` | 레거시 텔레메트리 블록 환경 설정. 마이그레이션 폴백으로만 읽음; 정규 값은 `/control_panel/widget_telemetry_blocks`에 있으며 파일은 더 이상 작성되지 않음. |

속성은 `config.json`에서 읽고 거기서부터 투영될 때 정규(canonical) 속성이다. 속성은 투영되면서 마이그레이션 입력으로도 소비될 수 있을 때 파생(derived) 속성이다. 컴포넌트가 런타임에 여전히 작성하는 모든 것 — 캐시, `$XDG_CONFIG_HOME/<app>`에 있는 네이티브 구성 파일, 인사말 상태 파일 — 은 어댑터 대상이며 구성으로 다시 읽어서는 안 된다.

Projection은 Hyprland, GTK/Qt, Waybar, Quickshell, 터미널, 알림, 잠금 화면 및 네이티브 애플리케이션에 대한 소비자를 지원합니다. Hyprland는 선택된 테마에서 활성 및 비활성 테두리 색상을 읽습니다; 명시적인 `appearance.accent_custom=true`만이 활성/그룹 테두리를 사용자 강조 색상으로 덮어쓸 수 있는 유일한 경우입니다. 테마 전환 시 컴포지터 재시작을 통해 이러한 값이 다시 적용되므로 이전 테마의 색상을 유지할 수 없습니다. 사용자 구성이 재생성되면 효과는 기본적으로 애니메이션이 활성화됩니다. `generated/`를 삭제하고 `argvus config apply`를 실행하면 `config.json`에서 바이트 단위로 파일이 재생성됩니다; 생성된 파일을 직접 편집하는 것은 지속적이지 않습니다. 런타임 진단은 전체 실행 튜플을 식별해야 합니다: 바이너리 경로, 버전/릴리스, 자산 루트, 구성 루트, 데이터 루트, 생성된 루트, 내부 루트 및 프로젝션 매니페스트. `status --verbose`는 이 튜플을 노출합니다.
발행된 세대/해시를 현재 해결된 구성과 비교합니다. 개발 및 설치된 패키지 테스트는 별도의 모드이며, 둘 다 관련 없는 실행 파일이나 개발자의 홈으로 조용히 되돌아갈 수 없습니다. `apply`는 또한 레거시 결합 테마 식별자의 조정 경계입니다: `argvus-dark-spaced`와 같은 값은 해결 전에 `theme=argvus-dark` 및 `variant=spaced`가 됩니다. 이 릴리스에서는 패키지된 자산의 식별이 매니페스트에 기록되지 않으므로, 이를 설명하는 진단 파일이 없습니다.
