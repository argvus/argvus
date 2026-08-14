import QtQuick
import QtQuick.Layouts
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleUser
    cardIcon:  "»"

    property string fullName: ""
    property string userName: ""
    property string hostName: ""
    property string userIcon: "\uf007"

    Timer {
        interval: 500; running: true; repeat: false
        onTriggered: userProc.running = true
    }

    Process {
        id: userProc
        command: ["bash", "-c",
            "echo \"$(getent passwd $(whoami) | cut -d: -f5 | cut -d, -f1)\"; " +
            "whoami; hostname"
        ]
        stdout: SplitParser {
            property int lineNum: 0
            onRead: data => {
                var line = data.trim()
                if (lineNum === 0) {
                    fullName = line
                } else if (lineNum === 1) {
                    userName = line
                } else if (lineNum === 2) {
                    hostName = line
                }
                lineNum++
            }
        }
    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 10

        Rectangle {
            implicitWidth: 40
            implicitHeight: 40
            radius: 20
            color: Theme.accentDim

            Text {
                anchors.centerIn: parent
                text: userIcon
                font.family: "Font Awesome 6 Free"
                font.pixelSize: 18
                font.weight: Font.Black
                color: Theme.accent
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Text {
                text: fullName && fullName !== userName ? fullName : userName
                color: Theme.fgText
                font.pixelSize: 13
                font.weight: Font.Bold
                font.family: "monospace"
                elide: Text.ElideRight
            }

            Text {
                text: userName + "@" + hostName
                color: Theme.fgDim
                font.pixelSize: 10
                font.family: "monospace"
                elide: Text.ElideRight
            }
        }
    }
}
