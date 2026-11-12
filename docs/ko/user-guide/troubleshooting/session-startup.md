---
title: 세션 시작 문제 해결
description: 인사 담당자, Hyprland 및 systemd 사용자 세션 실패를 진단하십시오.
---
달리다:

```sh
argvus sessionctl status
argvus sessionctl logs
systemctl --user status argvus-session.target
```
그리터 `argvus start`, Hyprland 및 관련 사용자 유닛을 개별적으로 확인하십시오. 패키지 빌드가 성공했다고 해서 설치된 페이로드나 활성 사용자 재정의가 최신이라는 것을 증명하지는 않습니다.

다음 순서로 확인을 수행하십시오:

1. 그리터가 설치된 ARGVUS greetd 설정을 사용하고 있는지 확인하십시오.
2. 세션 로그를 읽고 누락되었거나 잘못된 Hyprland Lua 구성이 있는지 확인하십시오.
3. `systemctl --user status argvus-session.target`를 사용하여 준비 상태와 대상 상태를 확인하십시오.
4. 모든 서비스를 한 번에 재시작하기보다는 실패하는 구성 요소를 검사하십시오.

로딩 오버레이가 활성 상태일 때 `argvus-session.target`가 비활성 상태이면, 서비스를 재시작하기 전에 투영 오류를 확인하십시오:

```sh
argvus config project
journalctl --user -u argvus-session.target -u argvus-session-prepare.service -b --no-pager
```
`argvus-config`는 `data/generated/` 아래에서 유일한 작성자이므로, `project`는 진단 중에 세대를 재생산하는 권위 있는 방법입니다. 세션 시작 시 커밋된 계획이 아직 없기 때문에 이를 실행하며, `${XDG_STATE_HOME:-$HOME/.local/state}/argvus/config-projection.json`에 기록하는 계획은 이후 `argvus sessionctl reload`가 다시 투영하지 않고 적용하는 것입니다.

세션은 치명적인 준비 오류나 대상 시작 실패 시 로딩 오버레이를 중지합니다. 시각적 소비자 실패는 문서화된 대체 방법을 사용하거나 명시적으로 보고해야 하며, 성공적인 세션으로 숨겨져서는 안 됩니다.

## 느린 로그인

만약 `greeter.handoff-takeover-failed`가 `handoff.master-acquired` 없이 나타난다면, 가져오기 오류를 확인하기 위해 `handoff.master-unavailable card=<name> errno=<n>`를 검사하십시오. 다른 컴포지터가 마스터일 때 관찰자가 열리면, 그 컴포지터가 마스터를 해제한 후에도 errno 13(`EACCES`)을 반환할 수 있습니다. 헬퍼는 마스터를 암시적으로 획득하기 위해 동일한 GPU를 다시 엽니다; `handoff.master-reopened`는 이 복구를 기록하고 해당 디스크립터에 새로운 프레임버퍼가 준비됩니다. 그리터는 실패한 헬퍼(`greeter.handoff-cancelled`)를 취소하므로, 사용자 컴포지터가 시작되기 전에 다른 대기를 추가할 수 없습니다. 가져오기 진행 중에 `release`를 보내는 세션은 `handoff.release-before-master`를 받습니다: 헬퍼가 확인하기 전에 자신의 DRM 디스크립터를 닫습니다. Hyprland/Aquamarine에서 그리터 컴포지터 충돌은 별개의 발견 사항입니다; 헬퍼를 취소한다고 해서 드라이버 또는 컴포지터 문제가 해결되었다는 것을 의미하지 않습니다.

`greeter.vt-switch-returned active=<tty>`는 logind의 switch 호출이 반환될 때 활성 VT를 기록합니다. 그림 시간 초과도 오류에 `active-vt=<tty>`를 기록합니다. 이것들은 스냅샷일 뿐 완료된 DRM 인계 증거는 아닙니다.

비교를 위해, 작업을 저장하고 동일한 기계, 테마 및 모니터 설정에서 버전별로 다섯 번 로그인하세요. 다음 로그아웃 전에 `/run/argvus-greeter/timing.log`, 세션 로그 및 부팅 저널을 `/tmp/argvus/` 아래의 별도 샘플 디렉토리에 복사하세요; 다음 그리터 실행은 그 타이밍 로그를 대체합니다. 각 샘플과 함께 부팅 ID를 기록하고 해당 부팅 내에서만 타임스탬프를 비교하세요. 인증에서 첫 프레임 전달, 세션 래퍼 진입, 최종 Hyprland 실행 및 로딩 스피너 정지까지를 측정하세요. 인증에서 스피너 정지까지의 중앙값과 최악의 시간을 비교하고, 검은 화면, 깜박임 또는 멈춘 스피너가 있는지 시각적으로 확인하세요. 데스크탑-서피스 지속 시간만으로는 끝까지 걸리는 로그인 시간이 아닙니다. 실패 및 복구 테스트는 작업 중이 아닌 임시 세션에서 반복하세요. 저장되지 않은 작업이 있는 세션을 종료하지 마세요.

각 시작 단계는 세션 로그에 지속 시간을 기록합니다. 마지막 로그인의 단계를 확인하려면, 가장 느린 것부터:

```sh
grep timing ~/.local/state/argvus/session.log | tail -30 | sort -k4 -rn
systemd-analyze --user critical-chain argvus-session.target
```
인사 화면에서 데스크톱으로 전환하는 시간을 맞추려면, 인사 화면 마크와 전달 마크를 저널과 함께 읽으세요. 모두 `CLOCK_MONOTONIC`를 사용하므로, 동시에 맞춰집니다:

```sh
cat /run/argvus-greeter/timing.log
journalctl -b -o short-monotonic --since -2min | grep -E 'greetd|Hyprland|coredump|argvus-greeter-handoff|session-loading'
grep monotonic_ns ~/.local/state/argvus/session.log | tail -5
```

`timing.log`는 마지막 로그인만 보유합니다: 인사 프로그램, DRM 핸드오프 표면(`handoff.master-acquired`, `handoff.output-found`, `handoff.first-frame`, `handoff.release-received`, `handoff.replaced-by-compositor`) 및 `greeter.*` 마크. `handoff.crtc` 라인은 핸드오프가 DRM 마스터를 기다리는 동안 모든 CRTC 변경을 기록합니다; `fb=0` 또는 `mode_valid=false`, 혹은 `handoff.output-found ... framebuffer=0`가 포함된 라인은 인사 프로그램 컴포지터가 핸드오프가 가져가기 전에 출력을 끈 것을 의미하며, 따라서 모니터는 다시 동기화해야 했습니다. `greeter.watchdog-stop-requested`는 인사 프로그램 세션이 자신의 컴포지터를 기다리지 않고 종료되었음을 의미하며, `greeter.compositor-exit-deferred`는 핸드오프 표면이 사라지면 컴포지터가 종료되도록 요청되었음을 의미합니다; `greeter.compositor-exit-requested` 단독으로는 즉시 종료가 요청되었음을 의미합니다. 저널의 greetd 및 logind 라인에 인사 프로그램 세션 종료와 사용자 세션 사이에 약 0.5초의 간격이 나타나면 시작 시, 로그인 동안 인사 담당 컴포지터의 분해가 실행되었습니다. 마크가 `compositor-in-foreground=true`를 표시하는 동안에는 예상된 상황입니다: 인사 담당 터미널의 중단으로 인해 지연 종료 전에 컴포지터가 종료됩니다. 이는 인사 담당자가 컴포지터 종료 전에 VT를 떠날 수 없을 때 발생합니다: `greeter.vt-switch-requested`를 확인한 후 `greeter.handoff-drawing`를 확인하세요; 두 번째 항목이 없으면 `greeter.handoff-takeover-failed error=<reason>`이 이유를 알려줍니다. `handoff.page-flip`는 핸드오프가 모드 변경 없이 인사 담당자의 마지막 프레임을 교체했음을 의미합니다; `handoff.page-flip-failed errno=<n>` 다음에 `handoff.modeset`는 커널이 페이지 플립을 거부했음을 의미합니다. `handoff.overlay plane=<id>` 그리고 나서 `handoff.retired`는 오버레이가 세션 스피너가 화면에 나타날 때까지 Hyprland의 첫 프레임을 숨겼음을 뜻합니다; `handoff.overlay-unavailable`는 CRTC에 사용 가능한 오버레이 플레인이 없음을 의미하며, `handoff.retire-timeout`는 세션 스피너가 전송되지 않았음을 의미합니다. `retire`. 세션 스피너(`argvus-greeter-handoff`)는 `first layer-shell frame callback` 및 `stop requested`를 저널에 기록합니다.

외관 변경, 테마 파일 변경, 새 악센트 추가 또는 이전 시작 스탬프 마이그레이션 후에는 `project.theme-switch` 또는 0이 아닌 `prepare.startup-theme` 항목이 예상됩니다. 동일한 파일을 재설치하면 콘텐츠 기반 시작 캐시가 유지됩니다. 모든 로그인 시 나타나면, 실체화된 테마 파일이 누락된 것이므로 `argvus theme-switch <theme>`를 한 번 실행하십시오.

세션 로그는 `$XDG_STATE_HOME/argvus/session.log`(보통 `~/.local/state/argvus/session.log`) 아래에 보관됩니다. 사용자 Hyprland 오버라이드를 변경한 후에는 로그아웃 후 다시 로그인하여 세션 진입점이 검증되고 다시 선택되도록 하십시오. 일반적인 시작 흐름은 [Graphical session](/docs/argvus-session/graphical-session/)를 참조하십시오.
