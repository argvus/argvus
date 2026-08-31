import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleSession
    cardIcon:  ">"

    property int idleTimeout: 300
    property bool lockDpms: true
    property var idleOptions: [
        { seconds: 60, label: "1m" },
        { seconds: 300, label: "5m" },
        { seconds: 600, label: "10m" },
        { seconds: 900, label: "15m" },
        { seconds: 1800, label: "30m" },
    ]

    function applyIdleTimeout(seconds) {
        if (idleSetProc.running) return
        idleSetProc.command = ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/idle-timeout.sh " + seconds]
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

    Item { Layout.preferredHeight: 4 }

    RowLayout {
        Layout.fillWidth: true
        spacing: 10

        Rectangle {
            id: lockDpmsToggleBtn
            width: 32; height: 32; radius: 6

            color: {
                if (!lockDpms) return Theme.bgCardAlt
                if (lockDpmsToggleArea.containsMouse) return Theme.accentDim
                return Theme.bgCard
            }
            border.color: lockDpms ? Theme.accent : Theme.danger
            border.width: 1
            Layout.alignment: Qt.AlignVCenter

            Behavior on color { ColorAnimation { duration: 150 } }

            Text {
                anchors.centerIn: parent
                text: lockDpms ? "\uf108" : "\uf00d"
                color: lockDpms ? Theme.accent : Theme.danger
                font.family: "Font Awesome 7 Free"
                font.pixelSize: 16
                font.weight: Font.Black
            }

            MouseArea {
                id: lockDpmsToggleArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: lockDpmsToggleProc.running = true
            }
        }

        ColumnLayout {
            spacing: 1
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter

            Text {
                text: Strings.lockDpmsTitle
                color: Theme.fgText
                font.pixelSize: 13
                font.family: "monospace"
                font.weight: Font.Medium
            }

            Text {
                text: lockDpms ? Strings.lockDpmsEnabled : Strings.lockDpmsDisabled
                color: lockDpms ? Theme.accent : Theme.danger
                font.pixelSize: 13
                font.family: "monospace"
                opacity: 1
            }
        }

        Text {
            text: lockDpms ? "ON" : "OFF"
            color: lockDpms ? Theme.accent : Theme.danger
            font.pixelSize: 16
            font.family: "monospace"
            font.weight: Font.Bold
            font.letterSpacing: 2
            Layout.alignment: Qt.AlignVCenter
        }
    }

    Timer {
        interval: 3000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: {
            idleStatusProc.running = true
            lockDpmsStatusProc.running = true
        }
    }

    Process {
        id: idleSetProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/idle-timeout.sh 300"]
        stdout: SplitParser {
            onRead: data => idleTimeout = Number(data.trim())
        }
    }

    Process {
        id: idleStatusProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/idle-timeout.sh status"]
        stdout: SplitParser {
            onRead: data => idleTimeout = Number(data.trim())
        }
    }

    Process {
        id: lockDpmsToggleProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/lock-dpms-toggle.sh toggle"]
        stdout: SplitParser {
            onRead: data => lockDpms = data.trim() === "enabled"
        }
    }

    Process {
        id: lockDpmsStatusProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/lock-dpms-toggle.sh status"]
        stdout: SplitParser {
            onRead: data => lockDpms = data.trim() === "enabled"
        }
    }
}
