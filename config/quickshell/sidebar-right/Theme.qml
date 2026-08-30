pragma Singleton
import QtQuick
import QtCore
import Quickshell
import Quickshell.Io

Singleton {
    id: root

    property string themeName: "argvus-dark-aether"
    property string gtkMode: "dark"
    readonly property string configHome: StandardPaths.writableLocation(StandardPaths.GenericConfigLocation)
    readonly property string stateHome: Quickshell.env("XDG_STATE_HOME") || StandardPaths.writableLocation(StandardPaths.HomeLocation) + "/.local/state"
    readonly property string legacyConfig: stateHome + "/argvus/config"
    FileView {
        id: themeNameFile
        path: root.configHome + "/argvus/.active-theme"
        onTextChanged: {
            var n = text().trim()
            if (n !== "") root.themeName = n
        }
    }

    FileView {
        id: gtkModeFile
        path: root.configHome + "/argvus/.gtk-mode"
        onTextChanged: {
            var m = text().trim()
            if (m === "light" || m === "dark") root.gtkMode = m
        }
    }

    // Mirrors the top waybar's horizontal margin so the sidebar stays
    // x-aligned with it (source of truth: waybar/config.jsonc, rewritten
    // by theme-switch.sh and spaces-switch.sh).
    FileView {
        id: waybarMarginFile
        path: root.configHome + "/waybar/config.jsonc"
        onTextChanged: {
            var m = text().match(/"margin-right":\s*(-?\d+)/)
            if (m) root._waybarMarginRight = parseInt(m[1], 10)
        }
    }

    property var themeObj: null

    FileView {
        id: themeFile
        blockLoading: true
        onTextChanged: {
            var qml = text().trim()
            if (qml === "") return
            var obj = Qt.createQmlObject(qml, root, "themeLoader")
            if (obj) {
                if (root.themeObj && root.themeObj !== obj) root.themeObj.destroy()
                root.themeObj = obj
            }
        }
    }

    function loadTheme() {
        // 1. User config (~/.config) — where accent-switch writes accent colors
        themeFile.path = root.configHome + "/quickshell/sidebar-right/themes/" +
            themeName + "/Theme.qml"
        themeFile.reload()
        if (themeFile.text().trim() !== "") return

        // 2. Legacy generated config (~/.local/state/argvus/config)
        themeFile.path = root.legacyConfig + "/quickshell/sidebar-right/themes/" +
            themeName + "/Theme.qml"
        themeFile.reload()
        if (themeFile.text().trim() !== "") return

        // 3. System default
        themeFile.path = "/usr/share/argvus/quickshell/sidebar-right/themes/" +
            themeName + "/Theme.qml"
        themeFile.reload()
    }

    function reloadActiveTheme() {
        themeNameFile.reload()
    }

    function reloadAccent() { themeFile.reload() }

    Timer {
        interval: 2000
        running: true
        repeat: true
        onTriggered: {
            themeNameFile.reload()
            gtkModeFile.reload()
            waybarMarginFile.reload()
        }
    }

    onThemeNameChanged: loadTheme()
    Component.onCompleted: loadTheme()

    // modeColors — non-null when theme declares a `light` QtObject and gtkMode is "light"
    readonly property var modeColors: gtkMode === "light" && themeObj && themeObj.light
        ? themeObj.light : null

    readonly property color accent:          themeObj ? themeObj.accent          : "#3590bd"
    readonly property color accentDim:       themeObj ? themeObj.accentDim       : "#223590bd"
    readonly property color accentMid:       themeObj ? themeObj.accentMid       : "#553590bd"
    readonly property color accentFaint:     themeObj ? themeObj.accentFaint     : "#0f3590bd"
    readonly property color accentLight:     themeObj ? themeObj.accentLight     : "#3590bd"
    readonly property color fgTitle:         themeObj ? themeObj.fgTitle         : "#3590bd"
    readonly property color fgText:          modeColors ? modeColors.fgText          : (themeObj ? themeObj.fgText         : "#cdd6f4")
    readonly property color fgDim:           modeColors ? modeColors.fgDim           : (themeObj ? themeObj.fgDim          : "#bac2de")
    readonly property color fgSubtle:        modeColors ? modeColors.fgSubtle        : (themeObj ? themeObj.fgSubtle       : "#a6adc8")
    readonly property color fgFaint:         modeColors ? modeColors.fgFaint         : (themeObj ? themeObj.fgFaint        : "#6c7086")
    readonly property color fgOnAccent:      themeObj ? themeObj.fgOnAccent      : "#111316"
    readonly property color bg:              modeColors ? modeColors.bg              : (themeObj ? themeObj.bg             : "#1e1e2e")
    readonly property color bgPanel:         modeColors ? modeColors.bgPanel         : (themeObj ? themeObj.bgPanel        : "#b01e1e2e")
    readonly property color bgCard:          modeColors ? modeColors.bgCard          : (themeObj ? themeObj.bgCard         : "#b0313244")
    readonly property color bgCardAlt:       modeColors ? modeColors.bgCardAlt       : (themeObj ? themeObj.bgCardAlt      : "#b045475a")
    readonly property color bgHeader:        modeColors ? modeColors.bgHeader        : (themeObj ? themeObj.bgHeader       : "#b011111b")
    readonly property color bgItem:          modeColors ? modeColors.bgItem          : (themeObj ? themeObj.bgItem         : "#0acdd6f4")
    readonly property color bgItemHover:     modeColors ? modeColors.bgItemHover     : (themeObj ? themeObj.bgItemHover    : "#14cdd6f4")
    readonly property color bgActive:        themeObj ? themeObj.bgActive        : "#223590bd"
    readonly property color border:          themeObj ? themeObj.border          : "#223590bd"
    readonly property color borderStrong:    themeObj ? themeObj.borderStrong    : "#553590bd"
    readonly property color borderItem:      themeObj ? themeObj.borderItem      : "#0f3590bd"
    readonly property color borderSubtle:    modeColors ? modeColors.borderSubtle    : (themeObj ? themeObj.borderSubtle   : "#45475a")
    readonly property color scrollbarFg:     modeColors ? modeColors.scrollbarFg     : (themeObj ? themeObj.scrollbarFg    : "#cdd6f4")
    readonly property color scrollbarBg:     modeColors ? modeColors.scrollbarBg     : (themeObj ? themeObj.scrollbarBg    : "#45475a")
    readonly property color danger:          modeColors ? modeColors.danger          : (themeObj ? themeObj.danger         : "#f38ba8")
    readonly property color dangerDim:       modeColors ? modeColors.dangerDim       : (themeObj ? themeObj.dangerDim      : "#f38ba866")
    readonly property color warn:            modeColors ? modeColors.warn            : (themeObj ? themeObj.warn           : "#f9e2af")
    readonly property color ok:              modeColors ? modeColors.ok              : (themeObj ? themeObj.ok             : "#a6e3a1")
    readonly property string fontMono:       modeColors ? modeColors.fontMono        : (themeObj ? themeObj.fontMono       : "monospace")
    readonly property string fontIcon:       modeColors ? modeColors.fontIcon        : (themeObj ? themeObj.fontIcon       : "Font Awesome 7 Free")
    readonly property int radius:            modeColors ? modeColors.radius          : (themeObj ? themeObj.radius         : 8)
    readonly property int radiusPill:        modeColors ? modeColors.radiusPill      : (themeObj ? themeObj.radiusPill     : 18)
    readonly property int radiusSmall:       modeColors ? modeColors.radiusSmall     : (themeObj ? themeObj.radiusSmall    : 4)
    readonly property int animFast:          modeColors ? modeColors.animFast        : (themeObj ? themeObj.animFast       : 150)
    readonly property int animNormal:        modeColors ? modeColors.animNormal      : (themeObj ? themeObj.animNormal     : 220)
    property int _waybarMarginRight: 0
    readonly property int waybarMarginRight: _waybarMarginRight
    readonly property int marginTop:         modeColors ? modeColors.marginTop       : (themeObj ? themeObj.marginTop      : 15)
    readonly property int marginBottom:      modeColors ? modeColors.marginBottom    : (themeObj ? themeObj.marginBottom   : 15)
    readonly property int marginRight:       modeColors ? modeColors.marginRight     : (themeObj ? themeObj.marginRight    : 15)
    readonly property int sidebarWidth:      modeColors ? modeColors.sidebarWidth    : (themeObj ? themeObj.sidebarWidth   : 350)
}
