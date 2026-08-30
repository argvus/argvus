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

    readonly property string script: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/spaces-switch.sh"
    readonly property string reloadScript: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/apps/hypr-init.sh --reload"
    readonly property var controls: [
        { key: "waybar", label: Strings.spacesWaybar, valueProp: "waybar", minDef: 0, max: 100 },
        { key: "gaps_in", label: Strings.spacesGapIn, valueProp: "gapsIn", minDef: 3, max: 100 },
        { key: "gaps_out", label: Strings.spacesGapOut, valueProp: "gapsOut", minDef: 1, max: 100 },
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
            delegate: RowLayout {
                Layout.fillWidth: true
                spacing: 6

                Text {
                    Layout.preferredWidth: 32
                    Layout.preferredHeight: 32
                    horizontalAlignment: Text.AlignHCenter
                    verticalAlignment: Text.AlignVCenter
                    text: "\uf065"
                    color: Theme.accent
                    font.family: "Font Awesome 7 Free"
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

                Item { Layout.fillWidth: true }

                // [-] button
                Rectangle {
                    width: 26; height: 26; radius: Theme.radiusSmall
                    color: minusArea.containsMouse ? Theme.accentDim : Theme.bgPanel
                    border.color: Theme.borderSubtle; border.width: 1
                    Layout.alignment: Qt.AlignVCenter

                    Text {
                        anchors.centerIn: parent
                        text: "−"
                        color: minusArea.containsMouse ? Theme.accent : Theme.fgSubtle
                        font.pixelSize: 14; font.family: "monospace"; font.bold: true
                    }

                    MouseArea {
                        id: minusArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            var v = Math.max(Math.round(card[modelData.valueProp]) - 1, modelData.minDef)
                            card[modelData.valueProp] = v
                            setValue(modelData.key, v)
                        }
                    }
                }

                // Value field
                Rectangle {
                    width: 48; height: 26; radius: Theme.radiusSmall
                    color: Theme.bgCard
                    border.color: Theme.borderSubtle; border.width: 1
                    Layout.alignment: Qt.AlignVCenter

                    TextInput {
                        id: valInput
                        anchors.fill: parent
                        anchors.margins: 2
                        horizontalAlignment: TextInput.AlignHCenter
                        verticalAlignment: TextInput.AlignVCenter
                        color: Theme.fgText
                        font.pixelSize: 11
                        font.family: "monospace"
                        font.bold: true
                        inputMethodHints: Qt.ImhDigitsOnly
                        validator: IntValidator { bottom: modelData.minDef; top: modelData.max }
                        clip: true
                        selectByMouse: true

                        property bool programmatic: false
                        property real boundValue: Math.round(card[modelData.valueProp])

                        onBoundValueChanged: {
                            if (!activeFocus) {
                                programmatic = true
                                text = boundValue
                                programmatic = false
                            }
                        }

                        Component.onCompleted: text = boundValue

                        onTextChanged: {
                            if (programmatic) return
                            var v = parseInt(text)
                            if (!isNaN(v)) {
                                v = Math.min(Math.max(v, modelData.minDef), modelData.max)
                                card[modelData.valueProp] = v
                                setValue(modelData.key, v)
                            }
                        }

                        onActiveFocusChanged: {
                            if (!activeFocus) {
                                programmatic = true
                                text = Math.round(card[modelData.valueProp])
                                programmatic = false
                            }
                        }
                    }
                }

                // [+] button
                Rectangle {
                    width: 26; height: 26; radius: Theme.radiusSmall
                    color: plusArea.containsMouse ? Theme.accentDim : Theme.bgPanel
                    border.color: Theme.borderSubtle; border.width: 1
                    Layout.alignment: Qt.AlignVCenter

                    Text {
                        anchors.centerIn: parent
                        text: "+"
                        color: plusArea.containsMouse ? Theme.accent : Theme.fgSubtle
                        font.pixelSize: 14; font.family: "monospace"; font.bold: true
                    }

                    MouseArea {
                        id: plusArea
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: {
                            var v = Math.min(Math.round(card[modelData.valueProp]) + 1, modelData.max)
                            card[modelData.valueProp] = v
                            setValue(modelData.key, v)
                        }
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
