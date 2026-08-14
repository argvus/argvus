import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    id: card
    cardTitle: Strings.cardTitleSpaces
    cardIcon: "»"

    property real waybar: 0
    property real gapsIn: 0
    property real gapsOut: 0
    property bool draggingWaybar: false
    property bool draggingIn: false
    property bool draggingOut: false

    readonly property string script: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/spaces-switch.sh"
    readonly property string reloadScript: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/hypr/scripts/init.sh --reload"
    readonly property var controls: [
        { key: "waybar", label: Strings.spacesWaybar, valueProp: "waybar", draggingProp: "draggingWaybar", minDef: 0, max: 100 },
        { key: "gaps_in", label: Strings.spacesGapIn, valueProp: "gapsIn", draggingProp: "draggingIn", minDef: 3, max: 100 },
        { key: "gaps_out", label: Strings.spacesGapOut, valueProp: "gapsOut", draggingProp: "draggingOut", minDef: 1, max: 100 },
    ]

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
                    if (parts.length === 2) {
                        var key = parts[0]
                        var val = parseFloat(parts[1])
                        for (var j = 0; j < controls.length; j++) {
                            if (controls[j].key === key) {
                                card[controls[j].valueProp] = val
                                break
                            }
                        }
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

    Process {
        id: reloadProc
        command: ["sh", "-c", card.reloadScript]
    }

    function setValue(key, value) {
        setProc.cmd = card.script + " --set " + key + " " + Math.round(value)
        setProc.running = true
    }

    function applyChanges() {
        reloadProc.running = true
    }

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 10

        Repeater {
            model: controls
            delegate: ColumnLayout {
                Layout.fillWidth: true
                spacing: 6

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 10

                    Text {
                        Layout.preferredWidth: 32
                        Layout.preferredHeight: 32
                        horizontalAlignment: Text.AlignHCenter
                        verticalAlignment: Text.AlignVCenter
                        text: "\uf065"
                        color: Theme.accent
                        font.family: "Font Awesome 6 Free"
                        font.pixelSize: 16
                        font.weight: Font.Black
                    }

                    Text {
                        text: modelData.label
                        color: Theme.fgText
                        font.pixelSize: 11
                        font.family: "monospace"
                        font.weight: Font.Medium
                        Layout.alignment: Qt.AlignVCenter
                    }

                    Item {
                        Layout.fillWidth: true
                        implicitHeight: 32

                        Rectangle {
                            id: track
                            anchors.verticalCenter: parent.verticalCenter
                            width: parent.width
                            height: 4; radius: 2
                            color: Theme.borderSubtle

                            Rectangle {
                                width: track.width * (card[modelData.valueProp] / modelData.max)
                                height: 4; radius: 2
                                color: Theme.accent
                                Behavior on width { NumberAnimation { duration: 80 } }
                            }
                        }

                        MouseArea {
                            anchors.fill: parent
                            hoverEnabled: true
                            cursorShape: Qt.PointingHandCursor

                            onPressed: function(mouse) {
                                card[modelData.draggingProp] = true
                                var v = Math.min(Math.max(mouse.x / width * modelData.max, modelData.minDef), modelData.max)
                                card[modelData.valueProp] = v
                                setValue(modelData.key, v)
                            }
                            onPositionChanged: function(mouse) {
                                if (pressed) {
                                    var v = Math.min(Math.max(mouse.x / width * modelData.max, modelData.minDef), modelData.max)
                                    card[modelData.valueProp] = v
                                    setValue(modelData.key, v)
                                }
                            }
                            onReleased: {
                                card[modelData.draggingProp] = false
                            }
                        }
                    }

                    Text {
                        text: Math.round(card[modelData.valueProp])
                        color: Theme.fgDim
                        font.pixelSize: 10
                        font.family: "monospace"
                        Layout.preferredWidth: 32
                        horizontalAlignment: Text.AlignRight
                    }
                }
            }
        }

        // Apply button (triggers full reload: SUPER+SHIFT+R)
        GlassButton {
            Layout.fillWidth: true
            implicitHeight: 36
            iconText: "\uf00c"
            label: Strings.btnApply
            active: true
            onClicked: applyChanges()
        }
    }
}