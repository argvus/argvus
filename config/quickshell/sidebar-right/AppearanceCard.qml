import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleAppearance
    cardIcon:  "»"

    property bool sysinfoEnabled: true
    property int idleTimeout: 300
    property var accentColors: ["#996548", "#3590bd", "#7391a5", "#17d174", "#cb17d1", "#d1174f", "#d1ce17", "#9617d1", "#595959"]
    property var idleOptions: [
        { seconds: 60, label: "1m" },
        { seconds: 300, label: "5m" },
        { seconds: 600, label: "10m" },
        { seconds: 900, label: "15m" },
        { seconds: 1800, label: "30m" },
    ]

    function applyAccent(color) {
        if (accentProc.running) return
        accentProc.command = ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/accent-switch.sh '" + color + "'"]
        accentProc.running = true
    }

    function applyIdleTimeout(seconds) {
        if (idleSetProc.running) return
        idleSetProc.command = ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/idle-timeout.sh " + seconds]
        idleSetProc.running = true
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 6

        GlassButton {
            Layout.fillWidth: true
            implicitHeight: 52
            iconText: "\uf03e"
            label: Strings.btnWallpaper
            onClicked: wallpaperProc.running = true
        }

        GlassButton {
            Layout.fillWidth: true
            implicitHeight: 52
            iconText: "\uf53f"
            label: Strings.btnTheme
            onClicked: themeProc.running = true
        }
    }

    Item { Layout.preferredHeight: 4 }

    GlassButton {
        Layout.fillWidth: true
        implicitHeight: 44
        iconText: "\uf53f"
        label: Strings.btnAccent
        accentColor: Theme.accent
        onClicked: {
            if (accentProc.running) return
            accentProc.command = ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/accent-switch.sh"]
            accentProc.running = true
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 5

        Repeater {
            model: accentColors

            Rectangle {
                required property string modelData
                Layout.fillWidth: true
                Layout.preferredHeight: 20
                radius: 3
                color: modelData
                border.width: Theme.accent.toString().toLowerCase() === modelData ? 2 : 1
                border.color: Theme.accent.toString().toLowerCase() === modelData ? Theme.fgText : Theme.borderSubtle

                MouseArea {
                    anchors.fill: parent
                    hoverEnabled: true
                    cursorShape: Qt.PointingHandCursor
                    onClicked: applyAccent(parent.modelData)
                }
            }
        }
    }

    Item { Layout.preferredHeight: 4 }

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
            id: sysinfoToggleBtn
            width: 32; height: 32; radius: 6

            color: {
                if (!sysinfoEnabled) return Theme.bgCardAlt
                if (sysinfoToggleArea.containsMouse) return Theme.accentDim
                return Theme.bgCard
            }
            border.color: sysinfoEnabled ? Theme.accent : Theme.danger
            border.width: 1
            Layout.alignment: Qt.AlignVCenter

            Behavior on color { ColorAnimation { duration: 150 } }

            Text {
                anchors.centerIn: parent
                text: sysinfoEnabled ? "\uf0ca" : "\uf00d"
                color: sysinfoEnabled ? Theme.accent : Theme.danger
                font.family: "Font Awesome 6 Free"
                font.pixelSize: 16
                font.weight: Font.Black
            }

            MouseArea {
                id: sysinfoToggleArea
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: toggleProc.running = true
            }
        }

        ColumnLayout {
            spacing: 1
            Layout.fillWidth: true
            Layout.alignment: Qt.AlignVCenter

            Text {
                text: Strings.sysinfoTitle
                color: Theme.fgText
                font.pixelSize: 13
                font.family: "monospace"
                font.weight: Font.Medium
            }

            Text {
                text: sysinfoEnabled ? Strings.sysinfoEnabled : Strings.sysinfoDisabled
                color: sysinfoEnabled ? Theme.accent : Theme.danger
                font.pixelSize: 13
                font.family: "monospace"
                opacity: 1
            }
        }

        Text {
            text: sysinfoEnabled ? "ON" : "OFF"
            color: sysinfoEnabled ? Theme.accent : Theme.danger
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
            checkProc.running = true
            idleStatusProc.running = true
        }
    }

    Process {
        id: wallpaperProc
        command: ["bash", "-c", "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/hypr/scripts/wallpaper-pick.sh"]
    }

    Process {
        id: themeProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/theme-switch.sh"]
        onExited: {
            Theme.reloadActiveTheme()
            Theme.reloadAccent()
        }
    }

    Process {
        id: accentProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/accent-switch.sh"]
        onExited: Theme.reloadAccent()
    }

    Process {
        id: toggleProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/waybar/scripts/sysinfo-toggle.sh toggle"]
        stdout: SplitParser {
            onRead: data => sysinfoEnabled = data.trim() === "enabled"
        }
    }

    Process {
        id: checkProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/waybar/scripts/sysinfo-toggle.sh status"]
        stdout: SplitParser {
            onRead: data => sysinfoEnabled = data.trim() === "enabled"
        }
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
