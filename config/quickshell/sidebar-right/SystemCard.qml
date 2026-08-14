import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleSystem
    cardIcon:  "»"

    property real cpuUsage:     0
    property real ramUsed:      0
    property real ramTotal:     0
    property real gpuUsage:     0
    property real gpuVramUsed:  0
    property real gpuVramTotal: 0
    property real gpuTemp:      0

    property var  _cpuPrev: null
    property int  _cpuLine: 0

    Timer {
        interval: 2000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: cpuProc.running = true
    }

    Process {
        id: cpuProc
        command: ["bash", "-c",
            "awk '/^cpu / {print $2,$3,$4,$5,$6,$7,$8}' /proc/stat; " +
            "free -b | awk '/^Mem/ {print $3,$2}'"
        ]
        onStarted: _cpuLine = 0
        stdout: SplitParser {
            onRead: data => {
                var parts = data.trim().split(/\s+/)
                if (_cpuLine === 0 && parts.length >= 7) {
                    var user=+parts[0], nice=+parts[1], sys=+parts[2],
                        idle=+parts[3], iowait=+parts[4], irq=+parts[5], softirq=+parts[6]
                    var total  = user+nice+sys+idle+iowait+irq+softirq
                    var active = total - idle - iowait
                    if (_cpuPrev) {
                        var dt = total  - _cpuPrev.total
                        var da = active - _cpuPrev.active
                        cpuUsage = dt > 0 ? Math.round(da / dt * 100) : 0
                    }
                    _cpuPrev = { total: total, active: active }
                } else if (_cpuLine === 1 && parts.length >= 2) {
                    ramUsed  = +parts[0] / 1073741824
                    ramTotal = +parts[1] / 1073741824
                }
                _cpuLine++
            }
        }
    }

    Timer {
        interval: 3000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: gpuProc.running = true
    }

    Process {
        id: gpuProc
        command: ["nvidia-smi",
            "--query-gpu=utilization.gpu,memory.used,memory.total,temperature.gpu",
            "--format=csv,noheader,nounits"
        ]
        stdout: SplitParser {
            onRead: data => {
                var p = data.split(",").map(s => s.trim())
                if (p.length >= 4) {
                    gpuUsage     = +p[0]
                    gpuVramUsed  = +p[1]
                    gpuVramTotal = +p[2]
                    gpuTemp      = +p[3]
                }
            }
        }
    }

    StatRow { label: "CPU";  value: cpuUsage + "%";
              pct: cpuUsage/100;
              barColor: cpuUsage > 85 ? Theme.danger : cpuUsage > 60 ? Theme.warn : Theme.accent }

    StatRow { label: "RAM";
              value: ramUsed.toFixed(1) + "/" + ramTotal.toFixed(1) + "G"
              pct: ramTotal > 0 ? ramUsed/ramTotal : 0
              barColor: { var r=ramTotal>0?ramUsed/ramTotal:0; return r>0.85?Theme.danger:r>0.65?Theme.warn:Theme.accent } }

    StatRow { label: "GPU";  value: gpuUsage + "%  " + gpuTemp + "°C";
              pct: gpuUsage/100;
              barColor: gpuUsage > 85 ? Theme.danger : gpuUsage > 60 ? Theme.warn : Theme.accent }

    StatRow { label: "VRAM";
              value: gpuVramUsed + "/" + gpuVramTotal + "M"
              pct: gpuVramTotal > 0 ? gpuVramUsed/gpuVramTotal : 0
              barColor: { var r=gpuVramTotal>0?gpuVramUsed/gpuVramTotal:0; return r>0.85?Theme.danger:r>0.65?Theme.warn:Theme.accent } }

    component StatRow: Item {
        property string label:    ""
        property string value:    ""
        property real   pct:      0
        property color  barColor: Theme.accent

        Layout.fillWidth: true
        implicitHeight: 34

        ColumnLayout {
            anchors.fill: parent
            spacing: 3

            RowLayout {
                Layout.fillWidth: true
                Text { text: label; color: Theme.accent; font.pixelSize: 10; font.family: "monospace"; Layout.preferredWidth: 36 }
                Item { Layout.fillWidth: true }
                Text { text: value; color: Theme.fgDim; font.pixelSize: 10; font.family: "monospace" }
            }

            Rectangle {
                Layout.fillWidth: true
                height: 3; radius: 1
                color: Theme.borderSubtle

                Rectangle {
                    width: parent.width * pct
                    height: 3; radius: 1
                    color: barColor
                    Behavior on width { NumberAnimation { duration: 400 } }
                }
            }
        }
    }
}
