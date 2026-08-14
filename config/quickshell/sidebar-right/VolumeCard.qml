import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleVolume
    cardIcon:  "»"

    property real volume: 0
    property bool muted:  false
    property bool dragging: false

    Timer {
        interval: 1000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: if (!dragging) readProc.running = true
    }

    Process {
        id: readProc
        command: ["bash", "-c",
            "wpctl get-volume @DEFAULT_AUDIO_SINK@"
        ]
        stdout: SplitParser {
            onRead: data => {
                var parts = data.trim().split(/\s+/)
                if (parts.length >= 2) {
                    volume = Math.min(parseFloat(parts[1]) || 0, 1.0)
                    muted  = data.includes("[MUTED]")
                }
            }
        }
    }

    Process {
        id: setProc
        property string cmd: ""
        command: ["bash", "-c", cmd]
    }

    function setVolume(v) {
        volume = Math.min(Math.max(v, 0), 1.0)
        setProc.cmd = "wpctl set-volume @DEFAULT_AUDIO_SINK@ " + volume.toFixed(2)
        setProc.running = true
    }

    function toggleMute() {
        muted = !muted
        setProc.cmd = "wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"
        setProc.running = true
    }

    // ── Mute button + slider + value ──
    RowLayout {
        Layout.fillWidth: true
        spacing: 10

        GlassButton {
            implicitHeight: 32
            implicitWidth: 32
            iconText: {
                if (muted) return "\uf6a9"
                if (volume > 0.6) return "\uf028"
                if (volume > 0.2) return "\uf027"
                return "\uf026"
            }
            label: ""
            active: !muted
            accentColor: muted ? Theme.danger : Theme.accent
            onClicked: toggleMute()
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
                    width: track.width * (muted ? 0 : volume)
                    height: 4; radius: 2
                    color: muted ? Theme.dangerDim : Theme.accent
                    Behavior on width { NumberAnimation { duration: 80 } }
                }
            }

            MouseArea {
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor

                onPressed: function(mouse) {
                    dragging = true
                    setVolume(mouse.x / width)
                }
                onPositionChanged: function(mouse) {
                    if (pressed) setVolume(mouse.x / width)
                }
                onReleased: {
                    dragging = false
                }
            }
        }

        Text {
            text: muted ? "mut" : Math.round(volume * 100) + "%"
            color: muted ? Theme.danger : Theme.fgDim
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
                onClicked: setVolume(volume + modelData.delta)
            }
        }
    }
}
