import QtQuick

QtObject {
    // Accent ------------------------------------------------------------------
    readonly property color accent:          "#181818"
    readonly property color accentDim:       "#22181818"   // accent 13% opaco
    readonly property color accentMid:       "#55181818"   // accent 33% opaco
    readonly property color accentFaint:     "#0f181818"   // accent 6% opaco
    readonly property color accentLight:     "#181818"     // destaque para titulos

    // Foreground --------------------------------------------------------------
    readonly property color fgTitle:         "#181818"     // gold titles
    readonly property color fgText:          "#181818"     // warm off-white
    readonly property color fgDim:           "#303030"     // secondary text
    readonly property color fgSubtle:        "#5a5a5a"     // muted
    readonly property color fgFaint:         "#8a8a8a"     // disabled
    readonly property color fgOnAccent:      "#f7f7f7"     // text on gold

    // Background --------------------------------------------------------------
    readonly property color bg:              "#f7f7f7"
    readonly property color bgPanel:         "#f0f7f7f7"   // panel with blur
    readonly property color bgCard:          "#f0ebebeb"   // card bg
    readonly property color bgCardAlt:       "#f0f2f2f2"   // card alt
    readonly property color bgHeader:        "#f0e7e7e7"   // card header
    readonly property color bgItem:          "#14c7c7c7"   // item/row
    readonly property color bgItemHover:     "#22d8d8d8"   // item hover
    readonly property color bgActive:        "#22181818"   // active state

    // Borders -----------------------------------------------------------------
    readonly property color border:          "#22181818"   // card border
    readonly property color borderStrong:    "#55181818"   // hover/highlight
    readonly property color borderItem:      "#0f181818"   // inner item
    readonly property color borderSubtle:    "#c7c7c7"     // neutral

    // Scrollbar
    readonly property color scrollbarFg:    "#5a5a5a"
    readonly property color scrollbarBg:    "#ebebeb"

    // Status ------------------------------------------------------------------
    readonly property color danger:          "#9b2f3b"
    readonly property color dangerDim:       "#9b2f3b66"
    readonly property color warn:            "#725a35"
    readonly property color ok:              "#2f6f49"

    // Tipography --------------------------------------------------------------
    readonly property string fontMono:       "monospace"
    readonly property string fontIcon:       "Font Awesome 6 Free"

    // Form --------------------------------------------------------------------
    readonly property int radius:            0
    readonly property int radiusPill:        0
    readonly property int radiusSmall:       0

    // Animations --------------------------------------------------------------
    readonly property int animFast:          150
    readonly property int animNormal:        220

    // Position margins
    readonly property int marginTop:          1
    readonly property int marginBottom:       1
    readonly property int sidebarWidth:       350
    readonly property int marginRight:        1
}
