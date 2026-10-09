# slint-hmi

Reusable industrial HMI components for [Slint](https://slint.dev): gauges,
rotary controls, equipment status indicators, alarm panels, and live trends.
The library consists of `.slint` source files with a shared light/dark theme,
configurable palettes, vector graphics, and host-owned data models.

| Component | Purpose |
| --- | --- |
| `RadialGauge` | Measurements with configurable scales, warning bands, and unavailable states. |
| `RotaryControl` | Stepped setpoints with pointer, touch, wheel, and keyboard input. |
| `StatusLamp` | Equipment states with text labels and optional blinking. |
| `AlarmPanel` | Scrollable alarm records with filtering and acknowledgement requests. |
| `LiveTrend` | Multiple time series with threshold lines and cursor readings. |
| `HmiAction` | Theme-aware action buttons used by the components and examples. |

Device communication, alarm persistence, and control validation belong to the
host application. The examples use simulated or static data.

## Contents

- [Run the examples](#run-the-examples)
- [Use the library](#use-the-library)
- [Appearance and common properties](#appearance-and-common-properties)
- [RadialGauge](#radialgauge)
- [RotaryControl](#rotarycontrol)
- [StatusLamp](#statuslamp)
- [AlarmPanel](#alarmpanel)
- [LiveTrend](#livetrend)
- [HmiAction](#hmiaction)
- [Embedded use and rendering limits](#embedded-use-and-rendering-limits)
- [Development](#development)
- [License](#license)

## Run the examples

The Rust examples require Rust **1.92 or newer** and use Slint **1.18.1**.
From the repository root:

```sh
cargo run --locked -p hmi-dashboard
cargo run --locked -p hmi-compact
```

The first build downloads and compiles dependencies. A desktop display is
required to run the examples.

### Dashboard

[`examples/hmi-dashboard`](examples/hmi-dashboard) is a cooling-loop dashboard
with simulated telemetry updated once per second. It demonstrates all components
in a scrolling workspace with Overview, Controls, Trends, and Alarms navigation.
It includes a process schematic, summary metrics, four gauges, temperature and
pump-speed controls, thermal and hydraulic trends, and an alarm console.

Use the dashboard to:

- Adjust temperature and pump speed using the rotary controls.
- Apply Eco (55 °C / 50%), Balanced (65 °C / 75%), or Boost (78 °C / 95%) presets.
- Pause telemetry, disconnect the controller, or inject and clear a pump fault.
- Filter alarms and acknowledge events without clearing their underlying conditions.
- Switch between light and dark themes and inspect trend readings by hovering or dragging.

A pump fault locks the setpoint controls and reduces simulated flow to zero.
Clearing a fault preserves alarm history. Cooling demand and energy figures are
illustrative simulation outputs. The dashboard retains up to 121 samples per
channel and 32 alarm records.

### Compact integration

[`examples/hmi-compact`](examples/hmi-compact) shows all five HMI components in a
smaller composition, with a warm palette, an unavailable sensor, irregular trend
timestamps, and a host-confirmed setpoint. The Rust host accepts requests up to
60 °C and rejects higher requests; the dial keeps displaying the confirmed value.
This example is a reference for integrating external validation and local styling.

## Use the library

The reusable library is the complete [`ui/`](ui) directory. It has no Rust crate
of its own and adds no charting, font, image, or device-protocol dependencies.
The Rust workspace builds the examples only. Slint's Rust, C++, JavaScript, and
Python integrations can compile the same UI sources; configure the library path
in the application's compiler integration.

### Add the sources

Copy the complete `ui/` directory into your application as
`vendor/slint-hmi/ui/`, and include `LICENSE` in `vendor/slint-hmi/`. Alternatively,
keep a local checkout of this project and map its `ui/` directory. Preserve the `hmi/` subdirectory: the entry point imports it
using relative paths.

Map the library name `slint-hmi` to that directory. Imports then use:

```slint
import {
    HmiTheme, HmiPalette, HmiMath, RadialGauge, RotaryControl,
    StatusLamp, EquipmentState, AlarmPanel, Alarm, AlarmSeverity, AlarmFilter,
    LiveTrend, TrendSeries, TrendSample, HmiAction
} from "@slint-hmi/hmi.slint";
```

Alternatively, import `hmi.slint` by its relative filesystem path from your
application's `.slint` file if no library mapping is configured.

### Rust integration

Add these dependencies to the consuming application's `Cargo.toml`:

```toml
[dependencies]
slint = "=1.18.1"

[build-dependencies]
slint-build = "=1.18.1"
```

For the vendored layout above, use this `build.rs`:

```rust
use std::{collections::HashMap, env, path::PathBuf};

fn main() {
    let app_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let config = slint_build::CompilerConfiguration::new().with_library_paths(
        HashMap::from([(
            "slint-hmi".to_string(),
            app_dir.join("vendor/slint-hmi/ui"),
        )]),
    );
    slint_build::compile_with_config("ui/main.slint", config).unwrap();
}
```

Create `ui/main.slint`:

```slint
import { HmiTheme, RadialGauge, RotaryControl }
    from "@slint-hmi/hmi.slint";

export { HmiTheme }

export component MainWindow inherits Window {
    title: "Process controls";
    width: 560px;
    height: 320px;
    in property <float> pressure: 7.2;
    in-out property <float> setpoint: 65;
    callback setpoint-changed(float);

    HorizontalLayout {
        padding: 16px;
        spacing: 16px;
        RadialGauge {
            label: "Circuit pressure";
            unit: "bar";
            minimum: 0;
            maximum: 10;
            value: root.pressure;
        }
        RotaryControl {
            label: "Temperature setpoint";
            unit: "°C";
            minimum: 30;
            maximum: 90;
            value <=> root.setpoint;
            value-changed(v) => { root.setpoint-changed(v); }
        }
    }
}
```

Create `src/main.rs`:

```rust
use slint::ComponentHandle;

slint::include_modules!();

fn main() -> Result<(), slint::PlatformError> {
    let ui = MainWindow::new()?;
    ui.set_pressure(7.2);
    ui.on_setpoint_changed(|value| println!("Requested setpoint: {value}"));
    ui.run()
}
```

For device-confirmed control, use `request-only` and update the confirmed value
only after host acceptance; see [RotaryControl](#rotarycontrol) and the compact
example. Update UI models on the event-loop thread.

The checked-in VS Code settings map `slint-hmi` to `./ui` for this repository.
In a consuming project, configure `slint.libraryPaths` to match its vendored path
when using the Slint editor extension.

## API reference

Properties in the following tables are inputs unless marked `in-out` or `out`.
Every component accepts the shared `palette` input described below.

## Appearance and common properties

Every HMI component accepts `palette: HmiPalette`, defaulting to
`HmiTheme.palette`. Inherited Slint geometry properties control its size and
placement. Use layouts to stretch components; the gauge and dial preserve their
square drawing area. Captions and severity labels accompany colors.

`HmiPalette` contains eleven `color` fields: `canvas`, `surface`, `elevated`,
`border`, `text`, `muted`, `accent`, `success`, `warning`, `critical`, and `track`.
Supply a complete struct for a local override, or bind its fields to the theme
while replacing selected colors, as in `examples/hmi-compact/ui/main.slint`.

| Global property | Type / default | Effect |
| --- | --- | --- |
| `HmiTheme.dark` | in-out bool / `false` | Light mode by default; set true for dark mode. |
| `HmiTheme.reduced-motion` | in-out bool / `false` | Default policy for lamp blinking. |
| `HmiTheme.palette` | out HmiPalette | The current shared palette. |

Set `HmiTheme.dark = false` or `HmiTheme.reduced-motion = true` in Slint. To access
the global from the host, also add `export { HmiTheme }` to your application's
main `.slint` file.

In Rust use `ui.global::<HmiTheme>().set_dark(false)` with `slint::ComponentHandle` imported.

The compact example starts in light mode using this host API.

`HmiMath.fraction(value, minimum, maximum)` returns a clamped 0–1 fraction,
or zero for an invalid range. `HmiMath.reading(value, decimals)` returns a rounded
numeric string; precision is capped at 0–3 decimal places and trailing zeroes
are not padded. All supplied measurements and range values should be finite.

## RadialGauge

```slint
RadialGauge {
    label: "Circuit pressure";
    unit: "bar";
    value: 7.2;
    minimum: 0;
    maximum: 10;
    warning-value: 8;
    critical-value: 9;
}
```

| Property | Type / default | Meaning |
| --- | --- | --- |
| `label`, `unit` | string / `"Process value"`, empty | Caption and engineering unit. |
| `value` | float / `0` | Actual reading, retained in the digital display even outside the scale. |
| `minimum`, `maximum` | float / `0`, `100` | Scale endpoints. |
| `decimals` | int / `1` | Rounded display precision. |
| `tick-count` | int / `11` | Tick dots, capped to 2–41. |
| `available` | bool / `true` | False displays an em dash and “No signal.” |
| `show-bands` | bool / `true` | Show upper warning/critical bands and severity text. |
| `warning-value` | float / 75% of range | Beginning of the warning band. |
| `critical-value` | float / 90% of range | Beginning of the critical band. |
| `accent` | color / palette accent | Normal reading/progress color. |
| `range-valid` | out bool | Whether maximum exceeds minimum. |
| `fraction` | out float | Normalized, clamped reading for other bindings. |

Thresholds are clamped to the scale and critical is raised to at least warning.
The arc stops at the scale endpoints; an out-of-range value is labeled explicitly.
Equal or reversed endpoints show “Invalid range” and an em dash. Bands detect
high values; low-value alarms belong in the host's alarm logic.

## RotaryControl

```slint
RotaryControl {
    label: "Temperature setpoint";
    unit: "°C";
    minimum: 30;
    maximum: 90;
    step: 0.5;
    decimals: 1;
    value: 65;
    value-changed(proposed) => { debug(proposed); }
}
```

| Property | Type / default | Meaning |
| --- | --- | --- |
| `label`, `unit` | string / `"Setpoint"`, empty | Caption and engineering unit. |
| `value` | in-out float / `50` | Current setpoint. Bind two ways to application state in normal mode. |
| `minimum`, `maximum` | float / `0`, `100` | Permitted UI range. |
| `step` | float / `1` | Increment, snapped relative to minimum. Nonpositive steps use 1. |
| `decimals` | int / `0` | Display precision. |
| `enabled` | bool / `true` | False dims the control and disables all input. |
| `read-only` | bool / `false` | Show the value while preventing changes. |
| `request-only` | bool / `false` | Emit a proposal without mutating the confirmed `value`. |
| `drag-distance` | length / `180px` | Vertical travel across the full range; at least 1px internally. |
| `ring-input` | bool / `true` | Enable clicking and turning the outer ring to select values. |
| `accent` | color / palette accent | Progress and focus color. |
| `range-valid` | out bool | Whether maximum exceeds minimum. |
| `has-focus` | out bool | Whether the dial's keyboard scope has focus. |

`value-changed(float)` fires for user proposals that differ from the current
value. Ordinary host writes to `value` do not emit the callback. In normal mode,
the value is updated before the callback; in request-only mode, the callback must
update the host's confirmed value after acceptance. Host writes are displayed
as supplied, allowing the application to show an unexpected actual setpoint.

Click the outer ring to set a value directly, then drag around the ring to turn
the dial. The 90-degree bottom gap maps to its nearest endpoint. Press and drag
the center upward to increase and downward to decrease, without an initial value
jump. Set `ring-input: false` to use vertical dragging everywhere. The wheel changes one step
per scroll event. Tab to the dial and use arrows for one step, Shift+arrow for
ten steps, or Home/End for exact endpoints. The +/− buttons also accept Tab and
Enter/Space. All user proposals are clamped, including ranges not divisible by
step. Invalid ranges disable input. There are no device writes inside the pack.

For externally confirmed values:

```slint
RotaryControl {
    value: root.confirmed-setpoint;
    request-only: true;
    value-changed(v) => { root.setpoint-requested(v); }
}
```

The compact example rejects proposals above its host-side operating limit of
60 °C while leaving the confirmed value unchanged.

## StatusLamp

```slint
StatusLamp {
    label: "Primary pump";
    state: EquipmentState.running;
}
StatusLamp {
    label: "Interlock";
    state: EquipmentState.fault;
    state-text: "Tripped";
    blink: true;
}
```

`EquipmentState` is `offline`, `idle`, `running`, `warning`, or `fault`.

| Property | Type / default | Meaning |
| --- | --- | --- |
| `label` | string / `"Equipment"` | Equipment name. |
| `state` | EquipmentState / `idle` | Semantic state and default color/caption. |
| `state-text` | string / empty | Optional replacement for the state caption. |
| `lamp-color` | color / derived from state | Local color override. |
| `lamp-size` | length / `14px` | Diameter, capped to 8–32px. |
| `blink` | bool / `false` | Enable blinking except for offline equipment. |
| `blink-interval` | duration / `600ms` | Each bright/dim phase; at least 100ms. |
| `reduced-motion` | bool / global setting | Force a steady lamp while preserving semantic text. |
| `caption` | out string | Effective state caption. |

Running uses success, warning uses warning, fault uses critical, and idle/offline
use muted. Offline lamps have a hollow-looking track fill. Blinking dims the lamp
but never removes the caption. Only lamps configured to blink run a timer.

## AlarmPanel

```slint
AlarmPanel {
    alarms: root.visible-alarms;
    filter <=> root.alarm-filter;
    filter-changed(f) => { root.filter-requested(f); }
    acknowledge-requested(id) => { root.acknowledge-requested(id); }
}
```

`Alarm` fields:

| Field | Type | Meaning |
| --- | --- | --- |
| `id` | string | Stable, unique host identifier, passed to acknowledgement callbacks. |
| `title`, `source`, `timestamp` | string | Description, equipment/source, host-formatted event time. |
| `severity` | AlarmSeverity | `info`, `warning`, or `critical`. |
| `acknowledged` | bool | Host-confirmed acknowledgement. |
| `active` | bool | Whether the underlying condition is still present. |

`AlarmFilter` is `all`, `unacknowledged`, `warning`, or `critical`.

| Property | Type / default | Meaning |
| --- | --- | --- |
| `title` | string / `"Alarm console"` | Panel heading. |
| `alarms` | [Alarm] / empty | **Visible, already filtered** model supplied by the host. |
| `filter` | in-out AlarmFilter / `all` | Selected filter; “New” means unacknowledged. |
| `show-filters` | bool / `true` | Show the four filter buttons. |
| `acknowledgement-enabled` | bool / `true` | Allow acknowledgement requests. |
| `empty-text` | string / `"No matching alarms"` | Empty-model message. |
| `row-height` | length / `88px` | Scrollable row height, at least 80px. |
| `scrollbar-gutter` | length / `24px` | Right padding reserved for overlay scrollbars, at least 20px. |

`filter-changed(AlarmFilter)` requests a new visible model. The panel updates its
selected filter first; the host handles storage/querying and updates `alarms`.
Setting `filter` programmatically does not emit a callback, so update the model
at the same time. This keeps sorting, filtering, paging, and history policies
under the application's control. Rows preserve the host's ordering.

`acknowledge-requested(string)` requests acknowledgement by ID. The panel does
not mutate records: update the corresponding host record and refresh the visible
model after confirmation. Acknowledging an active alarm does not clear it.
Cleared and acknowledged are displayed separately. ListView provides scrolling
and row virtualization; filter and acknowledgement buttons support keyboard use.
Acknowledgement buttons have a fixed 44px height; the reserved right gutter keeps
the scrollbar clear of their hit targets. Horizontal scrolling is disabled.

The examples share `examples/hmi-common/alarms.rs`, a bounded, in-memory host store
that filters records, acknowledges by ID, and retains cleared alarms in history.

## LiveTrend

```slint
LiveTrend {
    title: "Supply temperature";
    unit: "°C";
    minimum: 20;
    maximum: 110;
    time-minimum: 0;
    time-maximum: 60;
    series: [{
        label: "Supply", color: HmiTheme.palette.accent,
        samples: [{ time: 0, value: 40 }, { time: 10, value: 48 },
                  { time: 25, value: 44 }, { time: 60, value: 51 }]
    }];
    show-threshold: true;
    threshold: 85;
    threshold-label: "High temperature";
}
```

`TrendSample` has `time: float` and `value: float`. Time is in seconds in the
application's chosen epoch. Prefer elapsed/relative seconds to preserve float
precision over a short display interval. `TrendSeries` has `label: string`, `color: color`,
and `samples: [TrendSample]`. Series can have different sampling times; samples
within each series must be finite and strictly increasing in time.

| Property | Type / default | Meaning |
| --- | --- | --- |
| `title`, `unit` | string / `"Live trend"`, empty | Heading and engineering unit. |
| `series` | [TrendSeries] / empty | Host-owned channels and samples. |
| `minimum`, `maximum` | float / `0`, `100` | Fixed Y scale. |
| `time-minimum`, `time-maximum` | float / `0`, `60` | Visible time interval. |
| `time-start-label`, `time-end-label` | string / `"−60 s"`, `"Now"` | Host-formatted endpoint labels. |
| `decimals` | int / `1` | Y-axis/legend display precision. |
| `grid-divisions` | int / `4` | Y intervals, capped to 1–8. |
| `line-width` | length / `2px` | Series thickness, capped to 1–8px. |
| `show-legend` | bool / `true` | Channel names and latest/cursor readings. |
| `show-threshold` | bool / `false` | Draw a horizontal limit if it lies within the Y scale. |
| `threshold` | float / `80` | Limit value. |
| `threshold-label` | string / `"High limit"` | Limit annotation. |
| `threshold-color` | color / palette warning | Limit color. |
| `available` | bool / `true` | False hides signal data and shows “No signal.” |
| `has-data` | bool / `series.length > 0` | Set false when configured channels have no readings yet. |
| `cursor-enabled` | bool / `true` | Enable hover/touch inspection. |
| `range-valid` | out bool | Whether both scales have increasing endpoints. |
| `cursor-visible` | out bool | Whether an available, valid chart is being inspected. |
| `cursor-time` | out float | Time under the pointer, clamped to the visible interval. |

The chart draws straight line segments at real time coordinates and clips values
outside the plot. One-sample channels show a point. Hover or touch/drag shows a
crosshair and changes each legend reading to the last actual sample at or before
the cursor; before a channel's first sample the legend shows an em dash. Readings
are held samples, not interpolated values. `cursor-moved(float)` reports the
cursor time on click/drag. Bind to `cursor-time`/`cursor-visible` for hover state.
Invalid time or value ranges show “Invalid range.” An empty model shows
“Waiting for samples”; set `has-data` explicitly when creating named empty channels.

In Rust, mutable sample models should be `Rc<VecModel<TrendSample>>` converted
to `ModelRc`, as in the dashboard. Mutate them on the UI thread; use
`slint::invoke_from_event_loop` when forwarding readings from a worker thread.
Advance both time endpoints to scroll, and format endpoint labels in the host.

## HmiAction

```slint
HmiAction {
    text: "Apply";
    clicked => { root.apply-requested(); }
}
```

| Property | Type / default | Meaning |
| --- | --- | --- |
| `text` | string / empty | Button caption and accessibility label. |
| `enabled` | bool / `true` | Allow pointer, keyboard, and accessibility activation. |
| `selected` | bool / `false` | Highlight a selected action. |

`clicked()` fires on activation. Tab focuses the button; Enter or Space activates
it. The minimum target size is 44 × 44px. `HmiAction` is available from the public
entry point; drawing helpers in `hmi/primitives.slint` are implementation details.

## Embedded use and rendering limits

The pack uses Rectangle, Text, and Path; it includes no raster assets, external
fonts, or plotting dependencies. Interactive targets are at least 44px; input
controls expose accessibility roles and focus indications. Both examples are
desktop reference compositions. Place individual components in layouts suitable
for your device's display and select the Slint backend/features appropriate to
your target.

Trend rendering creates a path per segment and a marker/legend candidate per
sample. Keep the supplied model bounded and downsample large histories in the
host. The dashboard uses three channels with at most 121 samples each, sampled once
per second; hardware performance must be measured on the target. Alarm example
history is capped at 32 records. A production application's retention and
persistence policies may differ.

## Development

```text
ui/
  hmi.slint                 Public library entry point
  hmi/                      Components, theme, and drawing helpers
examples/
  hmi-dashboard/            Simulated cooling-loop application
  hmi-compact/              Compact host-validation example
  hmi-common/alarms.rs      Shared example alarm store
```

Public components, types, and globals are re-exported from `ui/hmi.slint`.
When changing that API, update the reference above and the examples that exercise
it. Keep application-specific telemetry and persistence outside `ui/`.

Run the following checks before contributing:

```sh
cargo fmt --all -- --check
cargo build --workspace --locked
cargo clippy --workspace --all-targets --locked -- -D warnings
```

## License

slint-hmi, including its examples, is licensed under the [MIT License](LICENSE).
Slint and other dependencies retain their own licenses; see the Slint crate's
[licensing information](https://github.com/slint-ui/slint/blob/v1.18.1/LICENSE.md).
