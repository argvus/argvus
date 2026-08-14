import QtQuick

QtObject {
    // Accent ------------------------------------------------------------------
    readonly property color accent:          "#3590bd"
    readonly property color accentDim:       "#223590bd"   // accent 13% opaco
    readonly property color accentMid:       "#553590bd"   // accent 33% opaco
    readonly property color accentFaint:     "#0f3590bd"   // accent 6% opaco
    readonly property color accentLight:     "#3590bd"     // destaque para titulos

    // Foreground --------------------------------------------------------------
    readonly property color fgTitle:         "#3590bd"     // highlight titles
    readonly property color fgText:          "#DFE5EA"     // warm off-white
    readonly property color fgDim:           "#B0BFCB"     // secondary text
    readonly property color fgSubtle:        "#718994"     // muted
    readonly property color fgFaint:         "#4B596A"     // disabled
    readonly property color fgOnAccent:      "#111316"     // text on gold

    // Background --------------------------------------------------------------
    readonly property color bg:              "#111316"
    readonly property color bgPanel:         "#d0111316"   // panel with blur
    readonly property color bgCard:          "#d0262933"   // card bg
    readonly property color bgCardAlt:       "#d0201F27"   // card alt
    readonly property color bgHeader:        "#d01D1D23"   // card header
    readonly property color bgItem:          "#14333647"   // item/row
    readonly property color bgItemHover:     "#22404B5E"   // item hover
    readonly property color bgActive:        "#223590bd"   // active state

    // Borders -----------------------------------------------------------------
    readonly property color border:          "#223590bd"   // card border
    readonly property color borderStrong:    "#553590bd"   // hover/highlight
    readonly property color borderItem:      "#0f3590bd"   // inner item
    readonly property color borderSubtle:    "#333647"     // neutral

    // Scrollbar
    readonly property color scrollbarFg:    "#718994"
    readonly property color scrollbarBg:    "#262933"

    // Status ------------------------------------------------------------------
    readonly property color danger:          "#4C3A3B"
    readonly property color dangerDim:       "#4C3A3B66"
    readonly property color warn:            "#695B57"
    readonly property color ok:              "#5B7683"

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
