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

    // Caminho absoluto de `argvus-accounts`, resolvido via shell de login.
    // Necessário porque processos iniciados pelo Hyprland (exec-once →
    // Quickshell) NÃO herdam o $PATH do seu shell interativo (.bashrc/.profile) —
    // só o ambiente mínimo exportado na sessão. Por isso o comando roda liso
    // no terminal mas falha silenciosamente quando disparado pela sidebar.
    // Resolvendo com "bash -lc" uma única vez, pegamos o mesmo PATH que o
    // terminal do usuário usa (incluindo ~/.cargo/bin, ~/.local/bin, etc.).
    property string accountsBin: "argvus-accounts"

    // ── Edit fields ──
    property string editNameValue: ""
    property string editStatus: ""
    property color  editStatusColor: Theme.ok

    // ── Tab labels ──
    readonly property var tabIcons: ["\uf03e", "\uf406", "\uf084"]
    readonly property var tabLabels: [Strings.userTabAvatar, Strings.userTabName, Strings.userTabPassword]

    Component.onCompleted: {
        userProc.running = true
        resolveBinProc.running = true
    }

    // ── Resolve o caminho absoluto de argvus-accounts via shell de login ──
    Process {
        id: resolveBinProc
        command: ["bash", "-lc", "command -v argvus-accounts"]
        stdout: SplitParser {
            onRead: data => {
                var p = data.trim()
                if (p.length > 0) accountsBin = p
            }
        }
        onExited: function(code) {
            if (code !== 0)
                console.warn("[UserCard] argvus-accounts não encontrado no PATH (nem via shell de login). " +
                    "Verifique se está instalado e no PATH, ex.: /usr/bin ou ~/.cargo/bin.")
        }
    }

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
        command: [accountsBin, "self", "avatar", ""]
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
        command: [accountsBin, "self", "avatar", "--remove"]
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

    // ── Set display name (argvus-accounts already handles polkit internally) ──
    Process {
        id: setNameProc
        property string newName: ""
        property string errOutput: ""
        command: ["true"]
        stderr: SplitParser {
            onRead: data => setNameProc.errOutput += data + "\n"
        }
        onRunningChanged: if (running) errOutput = ""
        onExited: function(code) {
            if (code === 0) {
                editStatus = Strings.user_name_ok
                editStatusColor = Theme.ok
                fullName = newName
            } else {
                // Mostra o motivo real (ex.: "Not authorized", polkit
                // recusado, etc.) em vez de um erro genérico, já que
                // várias causas diferentes levam a esse mesmo code != 0.
                var reason = errOutput.trim()
                if (reason.length === 0 && accountsBin === "argvus-accounts")
                    reason = "argvus-accounts não encontrado no PATH da sidebar (veja o console)"
                console.warn("[UserCard] argvus-accounts name falhou (code " + code + "): " + reason)
                editStatus = reason.length > 0
                    ? Strings.user_name_error + ": " + reason
                    : Strings.user_name_error
                editStatusColor = Theme.danger
            }
            statusTimer.restart()
        }
    }

    // ── Change password (argvus-accounts already handles polkit internally) ──
    Process {
        id: setPasswdProc
        property string oldPass: ""
        property string newPass: ""
        property string confirmPass: ""
        property string errOutput: ""
        command: ["true"]
        stderr: SplitParser {
            onRead: data => setPasswdProc.errOutput += data + "\n"
        }
        onRunningChanged: {
            if (running) {
                errOutput = ""
                // argv puro, sem shell: argvus-accounts passwd OLD NEW CONFIRM
                command = [accountsBin, "passwd", oldPass, newPass, confirmPass]
            }
        }
        onExited: function(code) {
            if (code === 0) {
                editStatus = Strings.userPasswordOk
                editStatusColor = Theme.ok
                oldPassField.text = ""
                newPassField.text = ""
                confirmPassField.text = ""
            } else {
                var reason = errOutput.trim()
                if (reason.length === 0 && accountsBin === "argvus-accounts")
                    reason = "argvus-accounts não encontrado no PATH da sidebar (veja o console)"
                console.warn("[UserCard] argvus-accounts passwd falhou (code " + code + "): " + reason)
                editStatus = reason.length > 0
                    ? Strings.userPasswordError + ": " + reason
                    : Strings.userPasswordError
                editStatusColor = Theme.danger
            }
            oldPass = ""
            newPass = ""
            confirmPass = ""
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
                    setAvatarProc.command = [accountsBin, "self", "avatar", path]
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
                    // Chama o binário diretamente (sem shell, sem pkexec):
                    // argvus-accounts já resolve o polkit internamente, e
                    // como argv puro não há necessidade de escapar aspas.
                    setNameProc.command = [accountsBin, "name", userName, editNameValue]
                    // Solta o foco do campo de texto antes de disparar o
                    // processo, para o diálogo de autenticação do polkit
                    // (se houver) conseguir assumir o foco de teclado.
                    nameInput.focus = false
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

            // Current password field
            Text {
                text: Strings.userPasswordCurrent
                color: Theme.fgDim
                font.pixelSize: 11
                font.family: "monospace"
            }

            Rectangle {
                Layout.fillWidth: true
                Layout.preferredHeight: 32
                radius: Theme.radiusSmall
                color: Theme.bgPanel
                border.color: oldPassField.activeFocus ? Theme.accent : Theme.borderSubtle
                border.width: 1

                TextInput {
                    id: oldPassField
                    anchors.fill: parent
                    anchors.margins: 8
                    color: Theme.fgText
                    font.pixelSize: 13
                    font.family: "monospace"
                    clip: true
                    verticalAlignment: Text.AlignVCenter
                    echoMode: TextInput.Password
                    onAccepted: newPassField.forceActiveFocus()
                }
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
                enabled: oldPassField.text.length > 0
                    && newPassField.text.length > 0
                    && newPassField.text === confirmPassField.text
                onClicked: {
                    setPasswdProc.oldPass = oldPassField.text
                    setPasswdProc.newPass = newPassField.text
                    setPasswdProc.confirmPass = confirmPassField.text
                    // Solta o foco antes de disparar, mesma razão do nome:
                    // deixa o diálogo de polkit (se houver) assumir o teclado.
                    confirmPassField.focus = false
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
