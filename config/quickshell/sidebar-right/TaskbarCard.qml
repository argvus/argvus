import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    id: card
    cardTitle: Strings.cardTitleTaskbar
    cardIcon: "»"

    // Position currently active in the system (from spaces-switch).
    property string appliedPos: "top"
    // Position the user has selected locally (survives polling).
    property string selectedPos: "top"
    // True once the user manually changes the selection.
    property bool dirty: false

    readonly property string script: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/spaces-switch.sh"

    Timer {
        interval: 2000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: statusProc.running = true
    }

    // Poll the persisted position. Only updates the local selection when the
    // user has not made an unapplied change.
    Process {
        id: statusProc
        command: ["bash", "-c", card.script + " --status"]
        stdout: SplitParser {
            onRead: data => {
                var lines = data.trim().split("\n")
                var pos = null
                for (var i = 0; i < lines.length; i++) {
                    var parts = lines[i].split("=")
                    if (parts.length === 2 && parts[0] === "waybar_pos") {
                        pos = parts[1]
                        break
                    }
                }
                if (pos === null) return
                card.appliedPos = pos
                if (!card.dirty) card.selectedPos = pos
            }
        }
    }

    Process {
        id: setProc
        property string cmd: ""
        command: ["sh", "-c", cmd]
        onExited: {
            // After applying, the selection is no longer pending.
            card.dirty = false
            card.appliedPos = card.selectedPos
        }
    }

    function select(pos) {
        card.selectedPos = pos
        card.dirty = (pos !== card.appliedPos)
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

        // Top / Bottom selection — explicit radio-style tiles.
        RowLayout {
            Layout.fillWidth: true
            spacing: 8

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 40
                radius: Theme.radiusSmall
                color: posTileTop.containsMouse ? Theme.accentDim : Theme.bgPanel
                border.color: card.selectedPos === "top" ? Theme.accent : Theme.borderSubtle
                border.width: card.selectedPos === "top" ? 2 : 1
                Behavior on color { ColorAnimation { duration: 150 } }

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 6
                    Rectangle {
                        width: 12; height: 12; radius: 6
                        border.color: card.selectedPos === "top" ? Theme.accent : Theme.fgSubtle
                        border.width: 2
                        color: "transparent"
                        Rectangle {
                            anchors.centerIn: parent
                            width: 6; height: 6; radius: 3
                            color: card.selectedPos === "top" ? Theme.accent : "transparent"
                        }
                    }
                    Text {
                        text: "\uf077"
                        color: card.selectedPos === "top" ? Theme.accent : Theme.fgSubtle
                        font.family: "Font Awesome 6 Free"
                        font.pixelSize: 14
                        font.weight: Font.Black
                    }
                    Text {
                        text: Strings.taskbarTop
                        color: card.selectedPos === "top" ? Theme.accent : Theme.fgText
                        font.pixelSize: 11
                        font.family: "monospace"
                        font.weight: card.selectedPos === "top" ? Font.Medium : Font.Normal
                    }
                }

                MouseArea {
                    id: posTileTop
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: card.select("top")
                }
            }

            Rectangle {
                Layout.fillWidth: true
                implicitHeight: 40
                radius: Theme.radiusSmall
                color: posTileBottom.containsMouse ? Theme.accentDim : Theme.bgPanel
                border.color: card.selectedPos === "bottom" ? Theme.accent : Theme.borderSubtle
                border.width: card.selectedPos === "bottom" ? 2 : 1
                Behavior on color { ColorAnimation { duration: 150 } }

                RowLayout {
                    anchors.centerIn: parent
                    spacing: 6
                    Rectangle {
                        width: 12; height: 12; radius: 6
                        border.color: card.selectedPos === "bottom" ? Theme.accent : Theme.fgSubtle
                        border.width: 2
                        color: "transparent"
                        Rectangle {
                            anchors.centerIn: parent
                            width: 6; height: 6; radius: 3
                            color: card.selectedPos === "bottom" ? Theme.accent : "transparent"
                        }
                    }
                    Text {
                        text: "\uf078"
                        color: card.selectedPos === "bottom" ? Theme.accent : Theme.fgSubtle
                        font.family: "Font Awesome 6 Free"
                        font.pixelSize: 14
                        font.weight: Font.Black
                    }
                    Text {
                        text: Strings.taskbarBottom
                        color: card.selectedPos === "bottom" ? Theme.accent : Theme.fgText
                        font.pixelSize: 11
                        font.family: "monospace"
                        font.weight: card.selectedPos === "bottom" ? Font.Medium : Font.Normal
                    }
                }

                MouseArea {
                    id: posTileBottom
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: card.select("bottom")
                }
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
