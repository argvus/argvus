import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleNotifications
    cardIcon:  "»"

    property var notifications: []
    property int unreadCount: 0
    property int currentPage: 0

    readonly property int pageSize: 3
    readonly property int pageCount: Math.ceil(notifications.length / pageSize)
    readonly property var pageNotifications: notifications.slice(currentPage * pageSize, (currentPage + 1) * pageSize)

    onNotificationsChanged: currentPage = 0

    Timer {
        interval: 3000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: historyProc.running = true
    }

    Process {
        id: historyProc
        command: ["dunstctl", "history"]
        stdout: StdioCollector {
            onStreamFinished: {
                try {
                    var data = JSON.parse(this.text)
                    var items = []
                    var entries = data.data[0] || []
                    for (var i = 0; i < entries.length && i < 9; i++) {
                        var n = entries[i]
                        items.push({
                            app:     n.appname  ? n.appname.data  : "sistema",
                            summary: n.summary  ? n.summary.data  : "",
                            body:    n.body     ? n.body.data     : ""
                        })
                    }
                    notifications = items
                    unreadCount   = items.length
                } catch(e) {
                    notifications = []
                    unreadCount   = 0
                }
            }
        }
    }

    Process {
        id: clearProc
        command: ["dunstctl", "history-clear"]
        onExited: function(code) {
            if (code === 0) {
                notifications = []
                unreadCount   = 0
            }
        }
    }

    // Header: count + nav + clear button
    RowLayout {
        Layout.fillWidth: true

        Text {
            text: unreadCount > 0 ? unreadCount + " " + Strings.notifRecent : Strings.notifNone
            color: Theme.fgText
            font.pixelSize: 10
            font.family: "monospace"
            Layout.fillWidth: true
        }

        RowLayout {
            visible: pageCount > 1
            spacing: 2

            NavBtn {
                text: "\uf053"
                enabled: currentPage > 0
                onClicked: currentPage--
            }

            Text {
                text: (currentPage + 1) + "/" + pageCount
                color: Theme.accent
                font.pixelSize: 9
                font.family: "monospace"
                horizontalAlignment: Text.AlignHCenter
                Layout.preferredWidth: 28
            }

            NavBtn {
                text: "\uf054"
                enabled: currentPage < pageCount - 1
                onClicked: currentPage++
            }
        }

        GlassButton {
            visible: unreadCount > 0
            implicitWidth: 70
            implicitHeight: 22
            iconText: "\uf2ed"
            label: Strings.notifClear
            active: false
            radius: 3
            onClicked: clearProc.running = true
        }
    }

    // List of notifications (current page)
    Repeater {
        model: pageNotifications
        delegate: Rectangle {
            Layout.fillWidth: true
            implicitHeight: notifCol.implicitHeight + 12
            radius: 4
            color: Theme.bgPanel
            border.color: Theme.borderSubtle
            border.width: 1

            ColumnLayout {
                id: notifCol
                anchors { fill: parent; margins: 8 }
                spacing: 2

                RowLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Text {
                        text: "\uf192"
                        font.family: "Font Awesome 6 Free"
                        font.pixelSize: 8
                        font.weight: Font.Black
                        color: Theme.accent
                        opacity: 1
                    }
                    Text {
                        text: modelData.app
                        color: Theme.accent
                        font.pixelSize: 9
                        font.family: "monospace"
                        opacity: 1
                        Layout.fillWidth: true
                        elide: Text.ElideRight
                    }
                }

                Text {
                    Layout.fillWidth: true
                    text: modelData.summary
                    color: Theme.fgText
                    font.pixelSize: 10
                    font.family: "monospace"
                    font.weight: Font.Medium
                    wrapMode: Text.WordWrap
                    visible: modelData.summary !== ""
                }

                Text {
                    Layout.fillWidth: true
                    text: modelData.body
                    color: Theme.fgText
                    font.pixelSize: 9
                    font.family: "monospace"
                    wrapMode: Text.WordWrap
                    visible: modelData.body !== ""
                    maximumLineCount: 2
                    elide: Text.ElideRight
                }
            }
        }
    }

    // Empty state
    Text {
        visible: notifications.length === 0
        Layout.fillWidth: true
        text: "\uf00d  " + Strings.notifAllClear
        font.pixelSize: 10
        font.family: "monospace"
        color: Theme.accent
        opacity: 1
        horizontalAlignment: Text.AlignHCenter
    }

    component NavBtn: Rectangle {
        property string text: ""
        signal clicked()

        width: 22; height: 22; radius: 4
        color: ma.containsMouse ? Theme.accentDim : "transparent"
        border.color: ma.containsMouse ? Theme.accent : "transparent"
        border.width: 1

        Behavior on color { ColorAnimation { duration: 150 } }

        Text {
            anchors.centerIn: parent
            text: parent.text
            color: parent.enabled ? (ma.containsMouse ? Theme.accent : Theme.fgSubtle) : Theme.borderSubtle
            font.family: "Font Awesome 6 Free"
            font.pixelSize: 10
            font.weight: Font.Black
        }

        MouseArea {
            id: ma
            anchors.fill: parent
            hoverEnabled: true
            enabled: parent.enabled
            cursorShape: Qt.PointingHandCursor
            onClicked: parent.clicked()
        }
    }
}
