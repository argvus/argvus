# Theme application latency audit

## Scope and isolation

The selector waits for the theme transaction to finish, then closes on its next
30 ms event poll. It does not introduce a multi-second closing delay. No early
success or detached runtime application was introduced.

Existing session logs showed preparation of 3,750–5,094 ms, projection of
300–350 ms and runtime application of 701–902 ms. Rendering took 6–15 ms within
preparation. Those broad timings cannot identify individual preparation costs.

Detailed debug-binary measurements use the appearance integration fixture,
mocked desktop commands, a temporary runtime directory and an inaccessible
session bus. They do not measure production compositor or wallpaper latency.
No services or theme settings in the real session were changed by this audit.

## Confirmed bottlenecks and changes

The taskbar utility script delegates to the widgets projector. The transaction
then invoked the widgets projector directly, running it twice. The redundant
utility wrapper invocation was removed; the widgets projector still owns drawer
and module visibility generation.

Effective taskbar fields are read repeatedly through child CLI processes. The
transaction now reads the effective document once and passes a serialized
`ARGVUS_THEME_TASKBAR_SNAPSHOT` only to the read-only projector. `config get`
uses it only for effective `/taskbar/` reads. Ordinary commands keep their
existing configuration loading behavior; missing snapshots fall back to normal
reads, and invalid snapshots fail rather than silently using inconsistent data.

Atomic build-file copies now leave identical destination contents untouched.
The existing content-addressed rendering cache and source invalidation remain.
Live transactions leave geometry materialization to the following canonical
projection and runtime reload, instead of executing shell geometry resets twice.
Offline and projection-owned materialization retain the existing helper path.

## Measurements

One isolated initial application measured preparation at 3,252 ms, with taskbar
work taking 2,021 ms. After removing the duplicate projector and sharing effective
configuration, comparable initial applications measured preparation at
2,028/1,996 ms and taskbar work at 966/915 ms. This is an indicative comparison,
not a distribution or hardware latency guarantee.

The latter run measured rendering at 56 ms, icons at 59 ms, file copies at
160 ms, GTK at 8 ms, and finishing at 746 ms. Finishing comprised accent at
99 ms, geometry at 488 ms and effects at 130 ms. The subsequent live-only
geometry deduplication removes that redundant helper path when the canonical
projection will perform it; the offline measurement still includes it.

`session.log` now exposes `theme.icons`, `theme.taskbar`, `theme.qt`, `theme.files`,
`theme.gtk`, `theme.finish`, `theme.finish.accent`, `theme.finish.geometry` and
`theme.finish.effects`, alongside existing render/prepare/project/runtime labels.

## Acceptance and remaining cost

The 500 ms warm end-to-end objective is not demonstrated. The measured taskbar
projector alone still takes about 0.9 s in this fixture. It still spawns CLI
getters and many text-editing processes. Replacing those shell projectors with
one structured Rust projection is the main remaining architectural task.
Finishing, canonical projection and runtime consumers also remain synchronous;
wallpaper decoding and consumer readiness need separate production measurements.

Unchanged control flow preserves profile application, cache invalidation,
selector error handling and completion after the transaction. User-facing
commands, translations and package dependencies have not changed. User guides
were audited; their documented interaction remains current.

A final isolated complete offline transaction measured 2,164 ms initially and
2,115 ms on immediate cached reapplication. Cache reuse therefore does not
remove the remaining projector/finishing cost. Validation passed 269 Rust tests,
Clippy with warnings denied, formatting, diff hygiene and four directed
appearance integrations covering live planning, in-place reload, custom accent
and preservation of taskbar modules/drawer markers. The real session still had
zero failed user units after validation.
