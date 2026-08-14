import QtQuick
import QtQuick.Layouts

BaseCard {
    cardTitle: Strings.cardTitleCalendar
    cardIcon:  "»"

    property var today: new Date()
    property int displayYear:  today.getFullYear()
    property int displayMonth: today.getMonth()

    readonly property var monthNames: Strings.monthNames
    readonly property var dayNames: Strings.dayNames

    // ── month navigator ──────────────────────────────────────────
    RowLayout {
        Layout.fillWidth: true

        NavBtn { text: "\uf053"; onClicked: prevMonth() }

        Text {
            Layout.fillWidth: true
            text: monthNames[displayMonth] + "  " + displayYear
            color: Theme.fgSubtle
            font.pixelSize: 12
            font.weight: Font.Medium
            font.family: "monospace"
            horizontalAlignment: Text.AlignHCenter
        }

        NavBtn { text: "\uf054"; onClicked: nextMonth() }
    }

    // ── day-of-week headers ──────────────────────────────────────
    GridLayout {
        Layout.fillWidth: true
        columns: 7
        columnSpacing: 0; rowSpacing: 2

        Repeater {
            model: dayNames
            Text {
                Layout.fillWidth: true
                text: modelData
                color: Theme.accent
                font.pixelSize: 9
                font.family: "monospace"
                horizontalAlignment: Text.AlignHCenter
                opacity: 1
            }
        }

        // day cells
        Repeater {
            model: calendarModel()
            delegate: Item {
                Layout.fillWidth: true
                implicitHeight: 26

                readonly property bool isToday:
                    modelData.day === today.getDate() &&
                    displayMonth === today.getMonth() &&
                    displayYear  === today.getFullYear()
                readonly property bool inMonth: modelData.inMonth

                Rectangle {
                    anchors.centerIn: parent
                    width: 22; height: 22; radius: 3
                    color: isToday ? Theme.accent : "transparent"
                    border.color: isToday ? Theme.accent : "transparent"
                }

                Text {
                    anchors.centerIn: parent
                    text: modelData.day > 0 ? modelData.day : ""
                    color: isToday ? Theme.bg : (inMonth ? Theme.fgText : Theme.borderSubtle)
                    font.pixelSize: 11
                    font.family: "monospace"
                    font.weight: isToday ? Font.Bold : Font.Normal
                }
            }
        }
    }

    function prevMonth() {
        if (displayMonth === 0) { displayMonth = 11; displayYear-- }
        else displayMonth--
    }
    function nextMonth() {
        if (displayMonth === 11) { displayMonth = 0; displayYear++ }
        else displayMonth++
    }

    function calendarModel() {
        var cells = []
        var first = new Date(displayYear, displayMonth, 1).getDay()
        var days  = new Date(displayYear, displayMonth + 1, 0).getDate()
        for (var i = 0; i < first; i++) cells.push({ day: 0, inMonth: false })
        for (var d = 1; d <= days; d++) cells.push({ day: d, inMonth: true })
        while (cells.length % 7 !== 0) cells.push({ day: 0, inMonth: false })
        return cells
    }

    component NavBtn: Rectangle {
        property string text: ""
        signal clicked()
        width: 28; height: 28; radius: 6
        color: ma.containsMouse ? Theme.accentDim : Theme.bgPanel
        border.color: ma.containsMouse ? Theme.accent : Theme.borderSubtle
        border.width: 1

        Behavior on color { ColorAnimation { duration: 150 } }

        Text {
            anchors.centerIn: parent
            text: parent.text
            color: ma.containsMouse ? Theme.accent : Theme.fgSubtle
            font.family: "Font Awesome 6 Free"
            font.pixelSize: 12
            font.weight: Font.Black
        }

        MouseArea {
            id: ma
            anchors.fill: parent
            hoverEnabled: true
            cursorShape: Qt.PointingHandCursor
            onClicked: parent.clicked()
        }
    }
}
