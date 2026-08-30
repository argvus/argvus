import QtQuick
import QtQuick.Layouts
import QtCore
import Quickshell.Io

BaseCard {
    id: root
    cardTitle: Strings.cardTitleWeather
    cardIcon:  "»"

    property string location: ""
    property string cityName:    "—"
    property string condition:   "—"
    property string tempC:       "—"
    property string feelsLike:   "—"
    property string humidity:    "—"
    property string windKmh:     "—"
    property string weatherIcon: "~"
    property bool   loading:     true
    property bool   hasError:    false
    property bool   refreshPending: false

    function refreshWeather() {
        if (fetchProc.running) {
            refreshPending = true
            return
        }
        refreshPending = false
        fetchProc.running = true
    }

    FileView {
        id: locationFile
        path: StandardPaths.writableLocation(StandardPaths.HomeLocation) +
              "/.config/argvus/.weather-location"
        onTextChanged: {
            root.location = text().trim()
            root.refreshWeather()
        }
    }

    // Atualiza a cada 15 minutos
    Timer {
        interval: 900000; running: true; repeat: true; triggeredOnStart: true
        onTriggered: root.refreshWeather()
    }

    Timer {
        id: deferredRefresh
        interval: 1
        repeat: false
        onTriggered: root.refreshWeather()
    }

    Process {
        id: fetchProc
        property string url: root.location !== ""
            ? "https://wttr.in/" + encodeURIComponent(root.location) + "?format=j1"
            : "https://wttr.in/?format=j1"
        command: ["curl", "-sf", "--max-time", "10", url]
        onStarted: { loading = true; hasError = false }
        stdout: StdioCollector {
            onStreamFinished: {
                loading = false
                try {
                    var d    = JSON.parse(this.text)
                    var cur  = d.current_condition[0]
                    var area = d.nearest_area[0]

                    cityName  = area.areaName[0].value + ", " + area.country[0].value
                    tempC     = cur.temp_C
                    feelsLike = cur.FeelsLikeC
                    humidity  = cur.humidity
                    windKmh   = cur.windspeedKmph
                    condition = cur.weatherDesc[0].value

                    var code = parseInt(cur.weatherCode)
                    if      (code === 113)                          weatherIcon = "\uf185"
                    else if (code === 116)                          weatherIcon = "\uf6c4"
                    else if (code === 119 || code === 122)          weatherIcon = "\uf0c2"
                    else if (code >= 176 && code <= 281)            weatherIcon = "\uf043"
                    else if (code >= 293 && code <= 308)            weatherIcon = "\uf043"
                    else if (code >= 311 && code <= 377)            weatherIcon = "\uf2dc"
                    else if (code >= 386 && code <= 395)            weatherIcon = "\uf0e7"
                    else                                            weatherIcon = "\uf185"

                    hasError = false
                } catch(e) {
                    hasError = true
                }
            }
        }
        onExited: function(code) {
            loading = false
            if (code !== 0) hasError = true
            if (root.refreshPending) {
                root.refreshPending = false
                deferredRefresh.restart()
            }
        }
    }

    Process {
        id: configureProc
        command: ["sh", "-c", "${ARGVUS_SYSTEM_CONFIG:-/usr/share/argvus}/argvus/sh/weather-location.sh"]
        onExited: {
            locationFile.reload()
            root.refreshWeather()
        }
    }

    // Estado: loading
    Text {
        visible: loading
        Layout.fillWidth: true
        text: Strings.weatherLoading
        color: Theme.accent; font.pixelSize: 16; font.family: "monospace"
        opacity: 1; horizontalAlignment: Text.AlignHCenter
    }

    // Estado: erro
    Text {
        visible: !loading && hasError
        Layout.fillWidth: true
        text: Strings.weatherError
        color: Theme.danger; font.pixelSize: 16; font.family: "monospace"
        horizontalAlignment: Text.AlignHCenter
    }

    // Conteudo principal
    ColumnLayout {
        visible: !loading && !hasError
        Layout.fillWidth: true
        spacing: 6

        // Cidade
        Text {
            text: cityName
            color: Theme.fgDim; font.pixelSize: 13; font.family: "monospace"
            Layout.fillWidth: true; elide: Text.ElideRight
            opacity: 1
        }

        // Icon + temperature
        RowLayout {
            Layout.fillWidth: true
            spacing: 10

            Text {
                text: weatherIcon
                font.family: "Font Awesome 7 Free"
                font.weight: Font.Black
                color: Theme.fgText
                font.pixelSize: 30
            }

            ColumnLayout {
                spacing: 1
                Text {
                    text: tempC + "°C"
                    color: Theme.fgSubtle
                    font.pixelSize: 26
                    font.weight: Font.Light
                    font.family: "monospace"
                }
                Text {
                    text: condition
                    color: Theme.fgText
                    font.pixelSize: 13
                    font.family: "monospace"
                    opacity: 1
                }
            }
        }

        // Divider
        Rectangle {
            Layout.fillWidth: true; height: 1
            color: Theme.accent; opacity: 0.10
        }

        // Details: feels like, humidity, wind
        GridLayout {
            Layout.fillWidth: true
            columns: 3
            columnSpacing: 14; rowSpacing: 8

            Repeater {
                model: [
                    { label: Strings.weatherFeelsLike, value: feelsLike + "°C" },
                    { label: Strings.weatherHumidity,  value: humidity  + "%"  },
                    { label: Strings.weatherWind,      value: windKmh   + " km/h" },
                ]
                delegate: ColumnLayout {
                    Layout.fillWidth: true
                    spacing: 4
                    Text {
                        text: modelData.label
                        color: Theme.accent; font.pixelSize: 16; font.family: "monospace"
                        opacity: 1; Layout.alignment: Qt.AlignHCenter
                    }
                    Text {
                        text: modelData.value
                        color: Theme.fgText; font.pixelSize: 16; font.family: "monospace"
                        Layout.alignment: Qt.AlignHCenter
                    }
                }
            }
        }

    }

    RowLayout {
        Layout.fillWidth: true
        spacing: 6

        Item { Layout.fillWidth: true }

        GlassButton {
            implicitWidth: 100
            implicitHeight: 30
            iconText: "\uf3c5"
            label: Strings.weatherConfigure
            active: false
            onClicked: {
                if (!configureProc.running) configureProc.running = true
            }
        }

        GlassButton {
            implicitWidth: 92
            implicitHeight: 30
            iconText: "\uf021"
            label: Strings.weatherRefresh
            active: false
            onClicked: root.refreshWeather()
        }
    }
}
