import QtQuick
import QtQuick.Layouts
import QtQuick.Controls
import Quickshell.Io

BaseCard {
    cardTitle: Strings.cardTitleUser
    cardIcon:  "»"

    // ── State ──
    property bool editing: false
    property int activeTab: 0  // 0=Avatar, 1=Name, 2=Password

    // ── User data ──
    property string fullName: ""
    property string userName: ""
    property string hostName: ""
    property string userIcon: "\uf007"
    property string avatarPath: ""

    // ── Edit fields ──
    property string editNameValue: ""
    property string editStatus: ""
    property color  editStatusColor: Theme.ok

    // ── Tab labels ──
    readonly property var tabIcons: ["\uf03e", "\uf406", "\uf084"]
    readonly property var tabLabels: [Strings.userTabAvatar, Strings.userTabName, Strings.userTabPassword]

    Component.onCompleted: userProc.running = true

    Timer {
        interval: 3000; running: true; repeat: true
        onTriggered: avatarCheckProc.running = true
    }

    // ── Read user info ──
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
                if (lineNum === 0)      fullName = line
                else if (lineNum === 1) userName = line
                else if (lineNum === 2) hostName = line
                lineNum++
            }
        }
    }

    // ── Check avatar existence ──
    Process {
        id: avatarCheckProc
        command: ["bash", "-c", "test -f $HOME/.face && echo $HOME/.face || echo "]
        stdout: SplitParser {
            onRead: data => {
                var p = data.trim()
                avatarPath = p.length > 0 ? p : ""
            }
        }
    }

    // ── Set avatar via argvus-accounts ──
    Process {
        id: setAvatarProc
        command: ["argvus-accounts", "self", "avatar", ""]
        onExited: function(code) {
            if (code === 0) {
                editStatus = Strings.userAvatarOk
                editStatusColor = Theme.ok
                avatarCheckProc.running = true
            } else {
                editStatus = Strings.userAvatarError
                editStatusColor = Theme.danger
            }
            statusTimer.restart()
        }
    }

    // ── Remove avatar ──
    Process {
        id: removeAvatarProc
        command: ["argvus-accounts", "self", "avatar", "--remove"]
        onExited: function(code) {
            if (code === 0) {
                editStatus = Strings.userAvatarRemoved
                editStatusColor = Theme.ok
                avatarPath = ""
            } else {
                editStatus = Strings.userAvatarError
                editStatusColor = Theme.danger
            }
            statusTimer.restart()
        }
    }

    // ── Set display name (polkit) ──
    Process {
        id: setNameProc
        property string newName: ""
        command: ["pkexec", "argvus-accounts", "name", userName, newName]
        onExited: function(code) {
            if (code === 0) {
                editStatus = Strings.user_name_ok
                editStatusColor = Theme.ok
                fullName = newName
            } else {
                editStatus = Strings.user_name_error
                editStatusColor = Theme.danger
            }
            statusTimer.restart()
        }
    }

    // ── Change password (polkit) ──
    Process {
        id: setPasswdProc
        property string user: ""
        property string pass: ""
        command: ["bash", "-c", ""]
        onRunningChanged: {
            if (running) {
                var escaped = pass.replace(/'/g, "'\\''")
                command = ["bash", "-c",
                    "printf '%s:%s\\n' '" + user + "' '" + escaped + "' | pkexec chpasswd"]
            }
        }
        onExited: function(code) {
            if (code === 0) {
                editStatus = Strings.userPasswordOk
                editStatusColor = Theme.ok
                newPassField.text = ""
                confirmPassField.text = ""
            } else {
                editStatus = Strings.userPasswordError
                editStatusColor = Theme.danger
            }
            pass = ""
            statusTimer.restart()
        }
    }

    // ── Pick avatar file via kdialog ──
    Process {
        id: pickAvatarProc
        command: ["kdialog", "--getopenfilename", "$HOME", "Images (*.png *.jpg *.jpeg *.webp)"]
        stdout: SplitParser {
            onRead: data => {
                var path = data.trim()
                if (path.length > 0) {
                    setAvatarProc.command = ["argvus-accounts", "self", "avatar", path]
                    setAvatarProc.running = true
                }
            }
        }
    }

    Timer {
        id: statusTimer
        interval: 3000; running: false; repeat: false
        onTriggered: editStatus = ""
    }

    // ════════════════════════════════════════════════════════════════
    //  CONTENT
    // ════════════════════════════════════════════════════════════════

    // ── User info row (always visible) ──
    RowLayout {
        Layout.fillWidth: true
        spacing: 10

        // Avatar circle
        Rectangle {
            implicitWidth: 40
            implicitHeight: 40
            radius: 20
            color: Theme.accentDim
            clip: true

            Image {
                anchors.fill: parent
                source: avatarPath.length > 0 ? "file://" + avatarPath : ""
                fillMode: Image.PreserveAspectCrop
                visible: status === Image.Ready
                smooth: true
                asynchronous: true
            }

            Text {
                anchors.centerIn: parent
                text: userIcon
                font.family: "Font Awesome 6 Free"
                font.pixelSize: 22
                font.weight: Font.Black
                color: Theme.accent
                visible: avatarPath.length === 0 || parent.children[0].status !== Image.Ready
            }
        }

        ColumnLayout {
            Layout.fillWidth: true
            spacing: 2

            Text {
                text: fullName && fullName !== userName ? fullName : userName
                color: Theme.fgText
                font.pixelSize: 15
                font.weight: Font.Bold
                font.family: "monospace"
                elide: Text.ElideRight
            }

            Text {
                text: userName + "@" + hostName
                color: Theme.fgDim
                font.pixelSize: 12
                font.family: "monospace"
                elide: Text.ElideRight
            }
        }

        // Edit toggle button
        Rectangle {
            implicitWidth: 28
            implicitHeight: 28
            radius: Theme.radiusSmall
            color: editBtnMa.containsMouse ? Theme.accentDim : "transparent"
            border.color: editing ? Theme.accent : Theme.borderSubtle
            border.width: 1

            Text {
                anchors.centerIn: parent
                text: editing ? "\uf00d" : "\uf303"
                font.family: "Font Awesome 6 Free"
                font.pixelSize: 14
                font.weight: Font.Black
                color: editing ? Theme.accent : Theme.fgSubtle
            }

            MouseArea {
                id: editBtnMa
                anchors.fill: parent
                hoverEnabled: true
                cursorShape: Qt.PointingHandCursor
                onClicked: {
                    editing = !editing
                    if (editing) {
                        editNameValue = fullName
                        activeTab = 0
                        editStatus = ""
                    }
                }
            }
        }
    }

    // ── Settings panel (visible when editing) ──
    ColumnLayout {
        visible: editing
        Layout.fillWidth: true
        spacing: 8

        Item { Layout.preferredHeight: 4 }

        // Tab bar
        RowLayout {
            Layout.fillWidth: true
            spacing: 4

            Repeater {
                model: tabIcons.length
                Rectangle {
                    Layout.fillWidth: true
                    Layout.preferredHeight: 30
                    radius: Theme.radiusSmall
                    color: activeTab === index ? Theme.accentDim : Theme.bgPanel
                    border.color: activeTab === index ? Theme.accent : Theme.borderSubtle
                    border.width: 1

                    RowLayout {
                        anchors.centerIn: parent
                        spacing: 4

                        Text {
                            text: tabIcons[index]
                            font.family: "Font Awesome 6 Free"
                            font.pixelSize: 12
                            font.weight: Font.Black
                            color: activeTab === index ? Theme.accent : Theme.fgSubtle
                        }

                        Text {
                            text: tabLabels[index]
                            font.pixelSize: 11
                            font.family: "monospace"
                            font.weight: Font.Bold
                            color: activeTab === index ? Theme.accent : Theme.fgSubtle
                        }
                    }

                    MouseArea {
                        anchors.fill: parent
                        hoverEnabled: true
                        cursorShape: Qt.PointingHandCursor
                        onClicked: activeTab = index
                    }
                }
            }
        }

        // Divider
        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.accent
            opacity: 0.18
        }

        // ── Tab: Avatar ──
        ColumnLayout {
            visible: activeTab === 0
            Layout.fillWidth: true
            spacing: 8

            // Current avatar preview
            Rectangle {
                Layout.alignment: Qt.AlignHCenter
                implicitWidth: 80
                implicitHeight: 80
                radius: 40
                color: Theme.accentDim
                border.color: Theme.accent
                border.width: 1
                clip: true

                Image {
                    anchors.fill: parent
                    source: avatarPath.length > 0 ? "file://" + avatarPath : ""
                    fillMode: Image.PreserveAspectCrop
                    visible: status === Image.Ready
                    smooth: true
                    asynchronous: true
                }

                Text {
                    anchors.centerIn: parent
                    text: "\uf007"
                    font.family: "Font Awesome 6 Free"
                    font.pixelSize: 32
                    font.weight: Font.Black
                    color: Theme.accent
                    visible: avatarPath.length === 0 || parent.children[0].status !== Image.Ready
                }
            }

            Text {
                Layout.fillWidth: true
                text: avatarPath.length > 0 ? Strings.userAvatarActive : Strings.userAvatarNone
                color: Theme.fgDim
                font.pixelSize: 11
                font.family: "monospace"
                horizontalAlignment: Text.AlignHCenter
            }

            // Buttons
            RowLayout {
                Layout.fillWidth: true
                spacing: 6

                GlassButton {
                    Layout.fillWidth: true
                    iconText: "\uf030"
                    label: Strings.userAvatarChange
                    onClicked: pickAvatarProc.running = true
                }

                GlassButton {
                    Layout.fillWidth: true
                    iconText: "\uf2ed"
                    label: Strings.userAvatarRemove
                    accentColor: Theme.danger
                    visible: avatarPath.length > 0
                    onClicked: removeAvatarProc.running = true
                }
            }
        }

        // ── Tab: Name ──
        ColumnLayout {
            visible: activeTab === 1
            Layout.fillWidth: true
            spacing: 8

            Text {
                text: Strings.user_name_label
                color: Theme.fgDim
                font.pixelSize: 11
                font.family: "monospace"
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: Theme.radiusSmall
                color: Theme.bgPanel
                border.color: nameInput.activeFocus ? Theme.accent : Theme.borderSubtle
                border.width: 1

                TextInput {
                    id: nameInput
                    anchors.fill: parent
                    anchors.margins: 8
                    color: Theme.fgText
                    font.pixelSize: 13
                    font.family: "monospace"
                    clip: true
                    verticalAlignment: Text.AlignVCenter
                    text: editNameValue
                    onTextChanged: editNameValue = text
                    Keys.onReturnPressed: saveNameBtn.clicked()
                    Keys.onEnterPressed: saveNameBtn.clicked()
                }
            }

            Text {
                text: Strings.user_name_hint
                color: Theme.fgFaint
                font.pixelSize: 10
                font.family: "monospace"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            GlassButton {
                id: saveNameBtn
                Layout.fillWidth: true
                iconText: "\uf0c7"
                label: Strings.user_name_save
                accentColor: Theme.ok
                enabled: editNameValue.length > 0 && editNameValue !== fullName
                onClicked: {
                    setNameProc.newName = editNameValue
                    setNameProc.running = true
                }
            }
        }

        // ── Tab: Password ──
        ColumnLayout {
            visible: activeTab === 2
            Layout.fillWidth: true
            spacing: 8

            Text {
                text: Strings.userPasswordDesc
                color: Theme.fgDim
                font.pixelSize: 11
                font.family: "monospace"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }

            // New password field
            Text {
                text: Strings.userPasswordNew
                color: Theme.fgDim
                font.pixelSize: 11
                font.family: "monospace"
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: Theme.radiusSmall
                color: Theme.bgPanel
                border.color: newPassField.activeFocus ? Theme.accent : Theme.borderSubtle
                border.width: 1

                TextInput {
                    id: newPassField
                    anchors.fill: parent
                    anchors.margins: 8
                    color: Theme.fgText
                    font.pixelSize: 13
                    font.family: "monospace"
                    clip: true
                    verticalAlignment: Text.AlignVCenter
                    echoMode: TextInput.Password
                    onAccepted: confirmPassField.forceActiveFocus()
                }
            }

            // Confirm password field
            Text {
                text: Strings.userPasswordConfirm
                color: Theme.fgDim
                font.pixelSize: 11
                font.family: "monospace"
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: Theme.radiusSmall
                color: Theme.bgPanel
                border.color: confirmPassField.activeFocus ? Theme.accent : Theme.borderSubtle
                border.width: 1

                TextInput {
                    id: confirmPassField
                    anchors.fill: parent
                    anchors.margins: 8
                    color: Theme.fgText
                    font.pixelSize: 13
                    font.family: "monospace"
                    clip: true
                    verticalAlignment: Text.AlignVCenter
                    echoMode: TextInput.Password
                    onAccepted: changePassBtn.clicked()
                }
            }

            // Mismatch warning
            Text {
                visible: confirmPassField.text.length > 0 && newPassField.text !== confirmPassField.text
                text: Strings.userPasswordMismatch
                color: Theme.danger
                font.pixelSize: 10
                font.family: "monospace"
                Layout.fillWidth: true
            }

            GlassButton {
                id: changePassBtn
                Layout.fillWidth: true
                iconText: "\uf084"
                label: Strings.userPasswordChange
                accentColor: Theme.ok
                enabled: newPassField.text.length > 0 && newPassField.text === confirmPassField.text
                onClicked: {
                    setPasswdProc.user = userName
                    setPasswdProc.pass = newPassField.text
                    setPasswdProc.running = true
                }
            }

            Text {
                text: Strings.userPasswordHint
                color: Theme.fgFaint
                font.pixelSize: 10
                font.family: "monospace"
                wrapMode: Text.WordWrap
                Layout.fillWidth: true
            }
        }

        // ── Status message ──
        Text {
            visible: editStatus.length > 0
            Layout.fillWidth: true
            text: editStatus
            color: editStatusColor
            font.pixelSize: 11
            font.family: "monospace"
            horizontalAlignment: Text.AlignHCenter
        }
    }
}
