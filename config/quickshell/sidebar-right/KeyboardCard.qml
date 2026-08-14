import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleKeyboard
    cardIcon:  "»"

    property string currentLayout: "br"
    property string kbDevice: "usb-usb-keyboard"

    Timer {
        interval: 500; running: true; repeat: false
        onTriggered: detectProc.running = true
    }

    Process {
        id: detectProc
        command: ["hyprctl", "devices", "-j"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    var data = JSON.parse(this.text)
                    var kbs = data.keyboards || []
                    for (var i = 0; i < kbs.length; i++) {
                        var kb = kbs[i]
                        if (kb.name === kbDevice || kb.name.includes("keyboard")) {
                            var km = (kb.active_keymap || "").toLowerCase()
                            currentLayout = km.includes("us") ? "us" : "br"
                            break
                        }
                    }
                } catch(e) {}
            }
        }
    }

    Process {
        id: switchProc
        command: ["hyprctl", "switchxkblayout", kbDevice, "next"]
        onExited: function(code) {
            if (code === 0)
                currentLayout = (currentLayout === "br") ? "us" : "br"
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 8

        Text {
            text: "\uf11c"
            font.family: "Font Awesome 6 Free"
            font.pixelSize: 18
            font.weight: Font.Black
            color: Theme.accent
            opacity: 1
        }

        LayoutBtn {
            Layout.fillWidth: true
            label: "BR  ABNT2"
            flag:  "🇧🇷"
            active: currentLayout === "br"
            onClicked: { if (currentLayout !== "br") switchProc.running = true }
        }

        LayoutBtn {
            Layout.fillWidth: true
            label: "US  QWERTY"
            flag:  "🇺🇸"
            active: currentLayout === "us"
            onClicked: { if (currentLayout !== "us") switchProc.running = true }
        }
    }

    component LayoutBtn: Rectangle {
        property string label:  ""
        property string flag:   ""
        property bool   active: false
        signal clicked()

        Layout.fillWidth: true
        implicitHeight: 50
        radius: 6

        color: {
            if (active) return Theme.bgCard
            if (ma.containsMouse) return Theme.accentDim
            return Theme.bgPanel
        }
        border.color: {
            if (active) return Theme.accent
            if (ma.containsMouse) return Theme.accent
            return Theme.borderSubtle
        }
        border.width: 1

        Behavior on color { ColorAnimation { duration: 150 } }

        ColumnLayout {
            anchors.centerIn: parent
            spacing: 3

            Text {
                Layout.alignment: Qt.AlignHCenter
                text: flag; font.pixelSize: 18
            }

            Text {
                Layout.alignment: Qt.AlignHCenter
                text: label
                color: active ? Theme.accent : (ma.containsMouse ? Theme.accent : Theme.fgSubtle)
                font.pixelSize: 9
                font.family: "monospace"
                font.weight: active ? Font.Medium : Font.Normal
            }
        }

        MouseArea {
            id: ma
            anchors.fill: parent
            hoverEnabled: true
            onClicked: parent.clicked()
            cursorShape: Qt.PointingHandCursor
        }
    }
}
