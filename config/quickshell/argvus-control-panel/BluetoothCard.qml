import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    id: card

    cardTitle: Strings.cardTitleBluetooth
    cardIcon:  "»"

    property bool available: false
    property bool powered: false
    property int connectedCount: 0
    property bool managerAvailable: false
    property string adapter: ""
    property string devices: ""
    property string statusName: "unavailable"
    readonly property string script: "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/bluetooth-control.sh"

    function parseStatus(text) {
        var lines = text.trim().split("\n")
        var obj = {}

        lines.forEach(function(line) {
            var idx = line.indexOf("=")
            if (idx > -1)
                obj[line.slice(0, idx)] = line.slice(idx + 1)
        })

        available = obj.available === "yes"
        powered = obj.powered === "yes"
        connectedCount = Number(obj.connected || 0)
        managerAvailable = obj.manager === "yes"
        adapter = obj.adapter || ""
        devices = obj.devices || ""
        statusName = obj.status || "unavailable"
    }

    Timer {
        interval: 3000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: {
            if (!statusProc.running)
                statusProc.running = true
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 10

        Rectangle {
            width: 44; height: 24
            radius: Theme.radius
            color: powered ? Theme.accent : Theme.borderSubtle
            Layout.alignment: Qt.AlignVCenter

            Behavior on color { ColorAnimation { duration: 150 } }

            Rectangle {
                width: 18; height: 18
                radius: Math.max(2, Theme.radius)
                x: powered ? parent.width - width - 3 : 3
                y: (parent.height - height) / 2
                color: Theme.bgHeader

                Behavior on x { NumberAnimation { duration: 180; easing.type: Easing.OutCubic } }
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    if (!toggleProc.running)
                        toggleProc.running = true
                }
            }
        }

        ColumnLayout {
            spacing: 1
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter

            Text {
                text: Strings.btTitle
                color: Theme.fgText
                font.pixelSize: 13
                font.family: Theme.fontMono
                font.weight: Font.Medium
            }

            Text {
                text: {
                    if (!available && statusName === "unavailable") return Strings.btUnavailable
                    if (!available) return Strings.btNoController
                    if (powered && connectedCount > 0) return connectedCount + " " + Strings.btConnected
                    return powered ? Strings.btEnabled : Strings.btDisabled
                }
                color: powered ? Theme.accent : Theme.danger
                font.pixelSize: 13
                font.family: Theme.fontMono
            }
        }

        Text {
            text: powered ? "\uf294" : "\uf05e"
            color: powered ? Theme.accent : Theme.danger
            font.family: Theme.fontIcon
            font.pixelSize: 18
            font.weight: Font.Black
            Layout.alignment: Qt.AlignVCenter
        }
    }

    RowLayout {
        Layout.fillWidth: true
        visible: available
        spacing: 6

        Text {
            text: "\uf2db"
            color: Theme.accent
            font.family: Theme.fontIcon
            font.pixelSize: 14
            font.weight: Font.Black
        }

        Text {
            text: adapter !== "" ? adapter : "Controller"
            color: Theme.fgText
            font.pixelSize: 13
            font.family: Theme.fontMono
            elide: Text.ElideRight
            Layout.fillWidth: true
        }
    }

    Text {
        Layout.fillWidth: true
        visible: powered && connectedCount > 0
        text: devices
        color: Theme.fgDim
        font.pixelSize: 12
        font.family: Theme.fontMono
        wrapMode: Text.Wrap
    }

    GlassButton {
        Layout.fillWidth: true
        implicitHeight: 38
        iconText: "\uf294"
        label: managerAvailable ? Strings.btOpenManager : Strings.btManagerOptional
        active: false
        onClicked: {
            if (!managerProc.running)
                managerProc.running = true
        }
    }

    Process {
        id: statusProc
        command: ["sh", "-c", card.script + " status"]
        stdout: StdioCollector {
            onStreamFinished: parseStatus(this.text)
        }
    }

    Process {
        id: toggleProc
        command: ["sh", "-c", card.script + " toggle"]
        stdout: StdioCollector {
            onStreamFinished: parseStatus(this.text)
        }
    }

    Process {
        id: managerProc
        command: ["sh", "-c", card.script + " manager"]
    }
}
