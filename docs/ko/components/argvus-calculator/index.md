---
title: ARGVUS 계산기
description: ARGVUS 계산기 — GTK4 / ARGVUS
---

# ARGVUS 계산기

`argvus calculator` · launcher: **ARGVUS 계산기**

- GTK4 / Relm4; ARGVUS theme (`time`), IBM Plex Mono.
- `+`, `-`, `*`, `/`, `%`, `±`; `50% = 0.5`.
- **공학용**: `sqrt`, `log`, `ln`, `exp`, `sin`, `cos`, `tan`, `asin`, `acos`, `atan`, `pi`, `e`, `^`, `(`, `)`.
- **라디안**: 선택 해제: 도.
- `-2^2 = -4`; `2^3^2 = 512`; `2*pi`; `1.5 = 1,5`.
- Enter: `=`; Escape: **지우기**; Backspace: **문자 삭제**; Tab / Shift+Tab.
- **결과 복사**; **기록**: 100; **기록 지우기**.
- IEEE 754 / f64; 15 digits; 4096 bytes; 64 levels.

## 기록

종료 후에도 최대 100개의 기록을 보관합니다. 기록 지우기는 저장된 항목도 삭제합니다. 설정은 ↑/↓로 탐색하고 Enter 또는 클릭으로 실행하며 Escape로 돌아갑니다.

`$XDG_DATA_HOME/argvus/calculator/history.json` (`~/.local/share/argvus/calculator/history.json`)

항상 중앙의 작은 플로팅 창(370 × 420)으로 열립니다. 공학용 모드에서는 확대할 수 있습니다.

기본 화면에는 수식, 결과, 키패드만 표시됩니다. 메뉴에서 공학용 모드, 각도 단위, 복사 및 별도 기록 화면에 접근합니다. 뒤로 또는 Escape로 계산에 돌아가고 기록 결과를 선택하면 재사용합니다. 기본 모드로 돌아오면 수식을 유지하며 작은 창으로 복원됩니다.

`SUPER + C` → `argvus calculator`

`.desktop`: `/usr/share/applications/argvus-calculator.desktop` · `Name=ARGVUS Calculator` · `Exec=argvus calculator`

ARGVUS: `focus_on_activate = true` (`org.argvus.*`, `sh.argvus.*`, `com.argvus.*`, `argvus-*`, `argvus.*`).

창을 늘리면 버튼도 커집니다. 새로 실행할 때마다 기본 크기로 열립니다. 보조 화면 위쪽에 뒤로 화살표가 있습니다.

`s Settings` → `settings` · Escape / ←

Min: **370 × 420** / **440 × 640** · `0–9`: **16 px**.

`2+2` → Enter / `=` → `4` (input) · `4*3` → Enter → `12`.

선택한 기본 또는 공학용 모드는 즉시 저장되며 다음 실행 시 복원됩니다.

`$XDG_CONFIG_HOME/argvus/calculator/settings.json` (`~/.config/argvus/calculator/settings.json`).
