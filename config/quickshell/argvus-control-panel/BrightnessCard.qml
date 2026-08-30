import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    id: card
    cardTitle: Strings.cardTitleBrightness
    cardIcon:  "»"

    property bool supported: false
    property real brightness: 0
    property bool dragging: false

    readonly property string script: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/scripts/argvus/brightness-switch.sh"

    visible: supported

    // Detect a controllable backlight (brightnessctl or DDC/CI). Re-checks
    // periodically so hotplugged monitors are picked up.
    Timer {
        interval: 5000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: detectProc.running = true
    }

    Process {
        id: detectProc
        command: ["bash", "-c", card.script + " --status"]
        stdout: SplitParser {
            onRead: data => {
                var backend = data.trim()
                card.supported = (backend === "brightnessctl" || backend === "ddcutil")
                if (card.supported) readProc.running = true
            }
        }
    }

    Timer {
        interval: 1000; running: supported; repeat: true
        onTriggered: if (!dragging) readProc.running = true
    }

    Process {
        id: readProc
        command: ["bash", "-c", card.script + " --get"]
        stdout: SplitParser {
            onRead: data => {
                var pct = parseFloat(data.trim()) || 0
                card.brightness = Math.min(pct / 100, 1.0)
            }
        }
    }

    Process {
        id: setProc
        property string cmd: ""
        command: ["bash", "-c", cmd]
    }

    function setBrightness(v) {
        brightness = Math.min(Math.max(v, 0), 1.0)
        setProc.cmd = card.script + " --set " + Math.round(brightness * 100) + "%"
        setProc.running = true
    }

    // ── Brightness slider + value ──
    RowLayout {
        Layout.fillWidth: true
        spacing: 10

        Text {
            Layout.preferredWidth: 32
            Layout.preferredHeight: 32
            horizontalAlignment: Text.AlignHCenter
            verticalAlignment: Text.AlignVCenter
            text: "\uf185"
            color: brightness > 0.5 ? Theme.accent : Theme.fgDim
            font.family: "Font Awesome 7 Free"
            font.pixelSize: 16
            font.weight: Font.Black
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
                    width: track.width * brightness
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
                    dragging = true
                    setBrightness(mouse.x / width)
                }
                onPositionChanged: function(mouse) {
                    if (pressed) setBrightness(mouse.x / width)
                }
                onReleased: {
                    dragging = false
                }
            }
        }

        Text {
            text: Math.round(brightness * 100) + "%"
            color: Theme.fgDim
            font.pixelSize: 10
            font.family: "monospace"
            Layout.preferredWidth: 32
            horizontalAlignment: Text.AlignRight
        }
    }

    // ── Step buttons ──
    RowLayout {
        Layout.fillWidth: true
        spacing: 4

        Repeater {
            model: [
                { label: "-10", delta: -0.10 },
                { label: "-5",  delta: -0.05 },
                { label: "+5",  delta:  0.05 },
                { label: "+10", delta:  0.10 },
            ]
            delegate: GlassButton {
                Layout.fillWidth: true
                implicitHeight: 22
                iconText: ""
                label: modelData.label
                active: false
                radius: 3
                onClicked: setBrightness(brightness + modelData.delta)
            }
        }
    }
}
