import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Quickshell.Io

BaseCard {
    id: card
    cardTitle: Strings.cardTitleDisplay
    cardIcon: "»"

    readonly property string script: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/monitor-switch.sh"

    property var monitors: []

    Timer {
        interval: 2000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: {
            statusProc.running = true
            brightProbe.running = true
            if (card.brightSupported) brightGet.running = true
        }
    }

    // Read current monitor state (name/res/pos/scale/power) from hyprctl.
    Process {
        id: statusProc
        command: ["bash", "-c", card.script + " --status"]
        stdout: SplitParser {
            onRead: data => {
                var lines = data.trim().split("\n")
                var current = null
                var list = []
                for (var i = 0; i < lines.length; i++) {
                    var parts = lines[i].split("=")
                    if (parts.length === 2) {
                        var key = parts[0]
                        var val = parts[1]
                        if (key === "name") {
                            current = { name: val, res: "", pos: "", scale: "", dpi: "", power: "" }
                            list.push(current)
                        } else if (current) {
                            current[key] = val
                        }
                    }
                }
                card.monitors = list
            }
        }
    }

    Process {
        id: setProc
        property string cmd: ""
        command: ["sh", "-c", cmd]
    }

    function setMonitor(monitor, key, value) {
        setProc.cmd = card.script + " --set " + monitor + " " + key + " " + value
        setProc.running = true
    }

    // ── Global brightness (backlight) reuse ──
    readonly property string brightScript: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/brightness-switch.sh"
    property bool brightSupported: false
    property real brightness: 0

    Process {
        id: brightProbe
        command: ["bash", "-c", card.brightScript + " --status"]
        stdout: SplitParser {
            onRead: data => {
                var backend = data.trim()
                card.brightSupported = (backend === "brightnessctl" || backend === "ddcutil")
            }
        }
    }
    Process {
        id: brightGet
        command: ["bash", "-c", card.brightScript + " --get"]
        stdout: SplitParser {
            onRead: data => {
                var pct = parseFloat(data.trim()) || 0
                card.brightness = Math.min(Math.max(pct / 100, 0), 1)
            }
        }
    }
    Process {
        id: brightSet
        property string cmd: ""
        command: ["bash", "-c", cmd]
    }

    function setBrightness(v) {
        var pct = Math.round(v * 100)
        brightSet.cmd = card.brightScript + " --set " + pct
        brightSet.running = true
    }

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 10

        // Global brightness (backlight for panel/internal)
        RowLayout {
            Layout.fillWidth: true
            visible: card.brightSupported
            spacing: 6
            Text {
                text: Strings.displayBrightness
                color: Theme.fgText
                font.family: "monospace"
                font.pixelSize: 11
                Layout.alignment: Qt.AlignVCenter
            }
            Slider {
                id: brightSlider
                Layout.fillWidth: true
                Layout.alignment: Qt.AlignVCenter
                from: 0
                to: 1
                value: card.brightness
                onMoved: card.setBrightness(value)
            }
            Text {
                text: Math.round(card.brightness * 100) + "%"
                color: Theme.fgText
                font.family: "monospace"
                font.pixelSize: 11
                Layout.preferredWidth: 32
                Layout.alignment: Qt.AlignVCenter
            }
        }

        Repeater {
            model: card.monitors
            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: 6

                // Monitor name + resolution
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Text {
                        text: modelData.name
                        color: Theme.accent
                        font.family: "monospace"
                        font.pixelSize: 12
                        font.weight: Font.Bold
                    }
                    Text {
                        text: modelData.res
                        color: Theme.fgDim
                        font.family: "monospace"
                        font.pixelSize: 11
                    }
                    Item { Layout.fillWidth: true }
                    Text {
                        text: modelData.power === "off" ? "\uf06e" : "\uf070"
                        color: modelData.power === "off" ? Theme.danger : Theme.fgSubtle
                        font.family: "Font Awesome 7 Free"
                        font.pixelSize: 13
                    }
                }

                // Scale slider
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Text {
                        text: Strings.displayScale
                        color: Theme.fgText
                        font.family: "monospace"
                        font.pixelSize: 11
                        Layout.alignment: Qt.AlignVCenter
                    }
                    Slider {
                        id: scaleSlider
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter
                        from: 0.5
                        to: 3.0
                        stepSize: 0.25
                        snapMode: Slider.SnapOnRelease
                        value: modelData.scale
                        onMoved: setMonitor(modelData.name, "scale", value)
                    }
                    Text {
                        text: modelData.scale
                        color: Theme.fgText
                        font.family: "monospace"
                        font.pixelSize: 11
                        Layout.preferredWidth: 30
                        Layout.alignment: Qt.AlignVCenter
                    }
                }

                // DPI slider (text/UI scaling per monitor)
                RowLayout {
                    Layout.fillWidth: true
                    spacing: 6
                    Text {
                        text: Strings.displayDpi
                        color: Theme.fgText
                        font.family: "monospace"
                        font.pixelSize: 11
                        Layout.alignment: Qt.AlignVCenter
                    }
                    Slider {
                        id: dpiSlider
                        Layout.fillWidth: true
                        Layout.alignment: Qt.AlignVCenter
                        from: 32
                        to: 300
                        stepSize: 4
                        snapMode: Slider.SnapOnRelease
                        value: modelData.dpi || 96
                        onMoved: setMonitor(modelData.name, "dpi", value)
                    }
                    Text {
                        text: modelData.dpi || 96
                        color: Theme.fgText
                        font.family: "monospace"
                        font.pixelSize: 11
                        Layout.preferredWidth: 30
                        Layout.alignment: Qt.AlignVCenter
                    }
                }

                // Power toggle
                GlassButton {
                    Layout.fillWidth: true
                    implicitHeight: 28
                    iconText: modelData.power === "off" ? "\uf06e" : "\uf070"
                    label: modelData.power === "off" ? Strings.displayPowerOn : Strings.displayPowerOff
                    active: false
                    onClicked: setMonitor(modelData.name, "power",
                        modelData.power === "off" ? "on" : "off")
                }
            }
        }

        // Open nwg-displays for advanced layout configuration
        GlassButton {
            Layout.fillWidth: true
            implicitHeight: 36
            iconText: "\uf2d0"
            label: Strings.displayAdvanced
            active: true
            onClicked: nwgProc.running = true
        }
    }

    Process {
        id: nwgProc
        command: ["nwg-displays"]
    }
}
