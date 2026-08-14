import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleNetwork
    cardIcon:  "»"

    property string iface:     "—"
    property string ip:        "—"
    property string downSpeed: "0 B/s"
    property string upSpeed:   "0 B/s"
    property string ssid:      ""
    property bool   connected: false
    property bool   networkingEnabled: true

    property var _prevRx: ({})
    property var _prevTx: ({})

    // ── Check network state via nmcli ──
    Timer {
        interval: 3000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: nmStateProc.running = true
    }

    Process {
        id: nmStateProc
        command: ["nmcli", "-t", "networking"]
        stdout: SplitParser {
            onRead: data => {
                networkingEnabled = data.trim() === "enabled"
            }
        }
    }

    Process {
        id: toggleNetProc
        property bool turnOn: true
        command: ["nmcli", "networking", turnOn ? "on" : "off"]
        onExited: {
            if (exitCode === 0) {
                networkingEnabled = turnOn
                netProc.running = true
            }
        }
    }

    // ── Network stats ──
    Timer {
        interval: 2000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: netProc.running = true
    }

    Process {
        id: netProc
        command: ["bash", "-c", [
            "IFACE=$(ip route show default 2>/dev/null | awk '/default/ {print $5; exit}')",
            "echo \"iface:$IFACE\"",
            "IP=$(ip -4 addr show $IFACE 2>/dev/null | awk '/inet / {print $2}' | cut -d/ -f1 | head -1)",
            "echo \"ip:${IP:-none}\"",
            "RX=$(cat /sys/class/net/$IFACE/statistics/rx_bytes 2>/dev/null || echo 0)",
            "TX=$(cat /sys/class/net/$IFACE/statistics/tx_bytes 2>/dev/null || echo 0)",
            "echo \"rx:$RX\"",
            "echo \"tx:$TX\"",
            "SSID=$(iwgetid -r $IFACE 2>/dev/null || echo '')",
            "echo \"ssid:$SSID\"",
        ].join("; ")]
        stdout: StdioCollector {
            onStreamFinished: {
                var lines = this.text.trim().split("\n")
                var obj = {}
                lines.forEach(function(line) {
                    var idx = line.indexOf(":")
                    if (idx >= 0) obj[line.slice(0, idx)] = line.slice(idx + 1)
                })

                if (!obj.iface || obj.iface === "") {
                    connected = false; return
                }

                iface     = obj.iface
                ip        = (obj.ip && obj.ip !== "none") ? obj.ip : "—"
                ssid      = obj.ssid || ""
                connected = ip !== "—"

                var now = Date.now()
                var rx  = parseFloat(obj.rx) || 0
                var tx  = parseFloat(obj.tx) || 0

                var prevRx = _prevRx[iface] || null
                var prevTx = _prevTx[iface] || null
                var prevTs = _prevRx["_ts"]  || null

                if (prevRx !== null && prevTs !== null) {
                    var dt = (now - prevTs) / 1000
                    if (dt > 0) {
                        downSpeed = fmtSpeed((rx - prevRx) / dt)
                        upSpeed   = fmtSpeed((tx - prevTx) / dt)
                    }
                }

                _prevRx = { [iface]: rx, _ts: now }
                _prevTx = { [iface]: tx }
            }
        }
    }

    function fmtSpeed(bps) {
        if (bps < 0)        bps = 0
        if (bps < 1024)     return Math.round(bps) + " B/s"
        if (bps < 1048576)  return (bps / 1024).toFixed(1) + " KB/s"
        return (bps / 1048576).toFixed(1) + " MB/s"
    }

    // ── Network toggle ──
    RowLayout {
        Layout.fillWidth: true
        spacing: 10

        Rectangle {
            id: toggleBtn
            width: 32; height: 32; radius: 6

            color: {
                if (!networkingEnabled) return Theme.bgCardAlt
                if (toggleArea.containsMouse) return Theme.accentDim
                return Theme.bgCard
            }
            border.color: networkingEnabled ? Theme.accent : Theme.danger
            border.width: 1
            Layout.alignment: Qt.AlignVCenter

            Behavior on color { ColorAnimation { duration: 150 } }

            Text {
                anchors.centerIn: parent
                text: networkingEnabled ? "\uf0ac" : "\uf127"
                color: networkingEnabled ? Theme.accent : Theme.danger
                font.family: "Font Awesome 6 Free"
                font.pixelSize: 14
                font.weight: Font.Black
            }

            MouseArea {
                id: toggleArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    toggleNetProc.turnOn = !networkingEnabled
                    toggleNetProc.running = true
                }
            }
        }

        ColumnLayout {
            spacing: 1
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter

            Text {
                text: Strings.netTitle
                color: Theme.fgText
                font.pixelSize: 11
                font.family: "monospace"
                font.weight: Font.Medium
            }

            Text {
                text: {
                    if (!networkingEnabled) return Strings.netDisabled
                    if (connected) return ssid !== "" ? ssid : Strings.netConnected
                    return Strings.netNoConnection
                }
                color: {
                    if (!networkingEnabled) return Theme.danger
                    if (connected) return Theme.accent
                    return Theme.danger
                }
                font.pixelSize: 9
                font.family: "monospace"
                opacity: 1
            }
        }

        Text {
            text: networkingEnabled ? "ON" : "OFF"
            color: networkingEnabled ? Theme.accent : Theme.danger
            font.pixelSize: 8
            font.family: "monospace"
            font.weight: Font.Bold
            font.letterSpacing: 2
            Layout.alignment: Qt.AlignVCenter
        }
    }

    // ── IP ──
    RowLayout {
        Layout.fillWidth: true
        visible: connected && networkingEnabled
        spacing: 4

        Text {
            text: "\uf0ac"
            color: Theme.accent
            font.family: "Font Awesome 6 Free"
            font.pixelSize: 10
            font.weight: Font.Black
            opacity: 1
        }
        Text {
            text: ip
            color: Theme.fgText
            font.pixelSize: 10
            font.family: "monospace"
            Layout.fillWidth: true
        }
        Text {
            text: iface
            color: Theme.accent
            font.pixelSize: 9
            font.family: "monospace"
            opacity: 1
        }
    }

    // ── Speed ↓ / ↑ ──
    RowLayout {
        Layout.fillWidth: true
        visible: connected && networkingEnabled
        spacing: 12

        RowLayout {
            spacing: 4
            Text { text: "↓"; color: Theme.fgText; font.pixelSize: 11; font.family: "monospace" }
            Text {
                text: downSpeed
                color: Theme.fgText; font.pixelSize: 10; font.family: "monospace"
                Layout.preferredWidth: 80
            }
        }

        RowLayout {
            spacing: 4
            Text { text: "↑"; color: Theme.fgText; font.pixelSize: 11; font.family: "monospace" }
            Text {
                text: upSpeed
                color: Theme.fgText; font.pixelSize: 10; font.family: "monospace"
            }
        }
    }
}
