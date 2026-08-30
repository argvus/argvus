import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    id: card
    cardTitle: Strings.cardTitleDefaultApps
    cardIcon: "»"

    property var entries: ({})
    property bool ready: false

    // Command that opens the argvus-default-apps graphical selector.
    readonly property string showCmd: "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/default_apps_show.sh"

    // Category order for display (matches argvus-default-apps).
    readonly property var cats: [
        { key: "terminal",        label: Strings.daTerminal,        icon: "\uf120" },
        { key: "file_manager",    label: Strings.daFileManager,     icon: "\uf07b" },
        { key: "text_editor",     label: Strings.daTextEditor,      icon: "\uf044" },
        { key: "terminal_editor", label: Strings.daTerminalEditor,  icon: "\uf120" },
        { key: "browser",         label: Strings.daBrowser,         icon: "\ue271" },
        { key: "image_viewer",    label: Strings.daImageViewer,     icon: "\uf03e" },
        { key: "pdf_viewer",      label: Strings.daPdfViewer,       icon: "\uf1c1" },
        { key: "video_player",    label: Strings.daVideoPlayer,     icon: "\uf03d" },
        { key: "audio_player",    label: Strings.daAudioPlayer,     icon: "\uf001" },
        { key: "archive",         label: Strings.daArchive,         icon: "\uf1c6" },
        { key: "launcher",        label: Strings.daLauncher,        icon: "\ue795" },
    ]

    Timer {
        interval: 1000; running: true; repeat: false
        onTriggered: getProc.running = true
    }

    Process {
        id: getProc
        command: ["sh", "-c",
            "command -v argvus-default-apps >/dev/null 2>&1 && argvus-default-apps get || " +
            "sh ${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/get-default.sh terminal"]
        stdout: SplitParser {
            onRead: data => {
                var lines = data.trim().split("\n")
                for (var i = 0; i < lines.length; i++) {
                    var eq = lines[i].indexOf("=")
                    if (eq > 0) {
                        var k = lines[i].substring(0, eq).trim()
                        var v = lines[i].substring(eq + 1).trim()
                        card.entries[k] = v
                    }
                }
                card.ready = true
            }
        }
    }

    Process {
        id: showProc
        command: ["sh", "-c", card.showCmd]
        onExited: function(code) {
            // Refresh values when the selector closes.
            if (code >= 0) getProc.running = true
        }
    }

    Text {
        text: Strings.daHint
        color: Theme.fgDim
        font.pixelSize: 10
        font.family: "monospace"
        wrapMode: Text.WordWrap
        Layout.fillWidth: true
    }

    ColumnLayout {
        Layout.fillWidth: true
        spacing: 4

        Repeater {
            model: card.cats
            delegate: RowLayout {
                Layout.fillWidth: true
                spacing: 6

                Text {
                    width: 20
                    text: modelData.icon
                    color: Theme.accent
                    font.family: "Font Awesome 7 Free"
                    font.pixelSize: 13
                    font.weight: Font.Black
                }

                Text {
                    text: modelData.label
                    color: Theme.fgText
                    font.pixelSize: 11
                    font.family: "monospace"
                    Layout.minimumWidth: 110
                }

                Item { Layout.fillWidth: true }

                Text {
                    text: card.ready ? (card.entries[modelData.key] || "—") : "…"
                    color: (card.ready && card.entries[modelData.key]) ? Theme.fgText : Theme.fgSubtle
                    font.pixelSize: 11
                    font.family: "monospace"
                    font.bold: true
                    elide: Text.ElideRight
                    Layout.maximumWidth: 110
                }
            }
        }
    }

    GlassButton {
        Layout.fillWidth: true
        implicitHeight: 36
        iconText: "\uf0c2"
        label: Strings.daOpen
        active: true
        onClicked: showProc.running = true
    }
}
