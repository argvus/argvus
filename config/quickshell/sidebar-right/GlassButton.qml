import QtQuick
import QtQuick.Layouts

Rectangle {
    id: root

    property string iconText: ""
    property string label: ""
    property bool   active: false
    property color  accentColor: Theme.accent
    signal clicked()

    implicitHeight: 34
    radius: Theme.radiusSmall

    color: {
        if (active) return Theme.bgCard
        if (ma.containsMouse) return Theme.accentDim
        return Theme.bgPanel
    }
    border.color: {
        if (active) return accentColor
        if (ma.containsMouse) return accentColor
        return Theme.borderSubtle
    }
    border.width: 1

    Behavior on color { ColorAnimation { duration: 150 } }

    RowLayout {
        anchors.centerIn: parent
        spacing: 5

        Text {
            visible: iconText !== ""
            text: iconText
            color: active || ma.containsMouse ? accentColor : Theme.fgSubtle
            font.family: "Font Awesome 6 Free"
            font.pixelSize: 13
            font.weight: Font.Black
        }

        Text {
            visible: label !== ""
            text: label
            color: active ? accentColor : (ma.containsMouse ? accentColor : Theme.fgSubtle)
            font.pixelSize: 9
            font.family: "monospace"
            font.weight: active ? Font.Medium : Font.Normal
        }
    }

    MouseArea {
        id: ma
        anchors.fill: parent
        hoverEnabled: true
        cursorShape: Qt.PointingHandCursor
        onClicked: root.clicked()
    }
}
