import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    id: card
    cardTitle: Strings.cardTitleTaskbar
    cardIcon: "»"

    property string currentPos: "top"
    property string selectedPos: "top"

    readonly property string script: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/spaces-switch.sh"

    Timer {
        interval: 2000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: statusProc.running = true
    }

    Process {
        id: statusProc
        command: ["bash", "-c", card.script + " --status"]
        stdout: SplitParser {
            onRead: data => {
                var lines = data.trim().split("\n")
                for (var i = 0; i < lines.length; i++) {
                    var parts = lines[i].split("=")
                    if (parts.length === 2 && parts[0] === "waybar_pos") {
                        card.currentPos = parts[1]
                        card.selectedPos = parts[1]
                        break
                    }
                }
            }
        }
    }

    Process {
        id: setProc
        property string cmd: ""
        command: ["sh", "-c", cmd]
    }

    function applyPosition() {
        setProc.cmd = card.script + " --set waybar_pos " + card.selectedPos
        setProc.running = true
    }

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 10

        Text {
            text: Strings.taskbarPositionLabel
            color: Theme.fgText
            font.family: "monospace"
            font.pixelSize: 11
            Layout.alignment: Qt.AlignLeft
        }

        // Top / Bottom selection
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            GlassButton {
                Layout.fillWidth: true
                implicitHeight: 40
                iconText: "\uf077"
                label: Strings.taskbarTop
                active: card.selectedPos === "top"
                accentColor: Theme.accent
                onClicked: card.selectedPos = "top"
            }
            GlassButton {
                Layout.fillWidth: true
                implicitHeight: 40
                iconText: "\uf078"
                label: Strings.taskbarBottom
                active: card.selectedPos === "bottom"
                accentColor: Theme.accent
                onClicked: card.selectedPos = "bottom"
            }
        }

        // Apply (persists position and restarts waybar)
        GlassButton {
            Layout.fillWidth: true
            implicitHeight: 36
            iconText: "\uf00c"
            label: Strings.btnApply
            active: true
            onClicked: applyPosition()
        }
    }
}
