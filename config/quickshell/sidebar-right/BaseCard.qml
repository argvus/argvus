import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root

    property string cardTitle: ""
    property string cardIcon:  ""

    implicitHeight: innerCol.implicitHeight + 24
    radius: Theme.radius

    // Glassmorphism: fundo escuro semi-transparente
    // O blur do Hyprland age na layer toda — este alpha cria o efeito "vidro fosco"
    color: Theme.bgHeader

    border.color: Theme.accentMid
    border.width: 1

    // Accent line no topo
    Rectangle {
        anchors { top: parent.top; left: parent.left; right: parent.right }
        anchors.leftMargin: parent.radius
        anchors.rightMargin: parent.radius
        height: 1
        color: Theme.accent
        opacity: 0.6
        z: 1
    }

    ColumnLayout {
        id: innerCol
        anchors {
            fill: parent
            topMargin: 12; bottomMargin: 12
            leftMargin: 14; rightMargin: 14
        }
        spacing: 8

        // card header
        RowLayout {
            spacing: 6

            Text {
                text: "\uf054"
                font.family: "Font Awesome 6 Free"
                font.pixelSize: 11
                font.weight: Font.Black
                color: Theme.accent
                opacity: 0.9
            }

            Text {
                text: root.cardTitle
                color: Theme.accentLight          // teal um pouco mais claro para realçar
                font.pixelSize: 11
                font.weight: Font.Bold
                font.letterSpacing: 1.5
                font.family: "monospace"
            }

            Item { Layout.fillWidth: true }
        }

        // divider
        Rectangle {
            Layout.fillWidth: true
            height: 1
            color: Theme.accent
            opacity: 0.18
        }

        // body content
        ColumnLayout {
            id: contentCol
            Layout.fillWidth: true
            spacing: 6
        }
    }

    default property alias content: contentCol.data
}
