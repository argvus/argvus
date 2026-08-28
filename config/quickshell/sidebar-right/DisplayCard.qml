import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    id: card
    cardTitle: Strings.cardTitleDisplay
    cardIcon: "»"

    readonly property string script: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/monitor-switch.sh"

    property var monitors: []

    Timer {
        interval: 2000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: statusProc.running = true
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
                            current = { name: val, res: "", pos: "", scale: "", power: "" }
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

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 10

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
                        font.family: "Font Awesome 6 Free"
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
