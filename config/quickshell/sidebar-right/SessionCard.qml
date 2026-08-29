import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleSession
    cardIcon:  ">"

    property int idleTimeout: 300
    property var idleOptions: [
        { seconds: 60, label: "1m" },
        { seconds: 300, label: "5m" },
        { seconds: 600, label: "10m" },
        { seconds: 900, label: "15m" },
        { seconds: 1800, label: "30m" },
    ]

    function applyIdleTimeout(seconds) {
        if (idleSetProc.running) return
        idleSetProc.command = ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/idle-timeout.sh " + seconds]
        idleSetProc.running = true
    }

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 7

        Text {
            text: Strings.idleLockTitle
            color: Theme.fgText
            font.pixelSize: 13
            font.family: "monospace"
            font.weight: Font.Medium
        }

        RowLayout {
            Layout.fillWidth: true
            spacing: 5

            Repeater {
                model: idleOptions

                Rectangle {
                    required property var modelData
                    Layout.fillWidth: true
                    Layout.preferredHeight: 26
                    radius: Theme.radiusSmall
                    color: idleTimeout === modelData.seconds ? Theme.accentDim : Theme.bgPanel
                    border.width: 1
                    border.color: idleTimeout === modelData.seconds ? Theme.accent : Theme.borderSubtle

                    Text {
                        anchors.centerIn: parent
                        text: parent.modelData.label
                        color: idleTimeout === parent.modelData.seconds ? Theme.accent : Theme.fgSubtle
                        font.family: "monospace"
                        font.pixelSize: 16
                        font.weight: Font.Bold
                    }

                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: applyIdleTimeout(parent.modelData.seconds)
                    }
                }
            }
        }
    }

    Timer {
        interval: 3000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: idleStatusProc.running = true
    }

    Process {
        id: idleSetProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/idle-timeout.sh 300"]
        stdout: SplitParser {
            onRead: data => idleTimeout = Number(data.trim())
        }
    }

    Process {
        id: idleStatusProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/idle-timeout.sh status"]
        stdout: SplitParser {
            onRead: data => idleTimeout = Number(data.trim())
        }
    }
}
