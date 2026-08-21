import QtQuick

QtObject {
    // Accent ------------------------------------------------------------------
    readonly property color accent:          "#eeeeee"
    readonly property color accentDim:       "#22eeeeee"   // accent 13% opaco
    readonly property color accentMid:       "#55eeeeee"   // accent 33% opaco
    readonly property color accentFaint:     "#0feeeeee"   // accent 6% opaco
    readonly property color accentLight:     "#eeeeee"     // destaque para titulos

    // Foreground --------------------------------------------------------------
    readonly property color fgTitle:         "#eeeeee"     // highlight titles
    readonly property color fgText:          "#eeeeee"     // primary text
    readonly property color fgDim:           "#aaaaaa"     // secondary text
    readonly property color fgSubtle:        "#666666"     // muted
    readonly property color fgFaint:         "#444444"     // disabled
    readonly property color fgOnAccent:      "#000000"     // text on accent

    // Background --------------------------------------------------------------
    readonly property color bg:              "#000000"
    readonly property color bgPanel:         "#d0000000"   // panel with blur
    readonly property color bgCard:          "#d00a0a0a"   // card bg
    readonly property color bgCardAlt:       "#d00f0f0f"   // card alt
    readonly property color bgHeader:        "#d0050505"   // card header
    readonly property color bgItem:          "#141a1a1a"   // item/row
    readonly property color bgItemHover:     "#22222222"   // item hover
    readonly property color bgActive:        "#22eeeeee"   // active state

    // Borders -----------------------------------------------------------------
    readonly property color border:          "#22eeeeee"   // card border
    readonly property color borderStrong:    "#55eeeeee"   // hover/highlight
    readonly property color borderItem:      "#0feeeeee"   // inner item
    readonly property color borderSubtle:    "#1a1a1a"     // neutral

    // Scrollbar
    readonly property color scrollbarFg:    "#666666"
    readonly property color scrollbarBg:    "#0a0a0a"

    // Status ------------------------------------------------------------------
    readonly property color danger:          "#6b3030"
    readonly property color dangerDim:       "#6b303066"
    readonly property color warn:            "#6b5a30"
    readonly property color ok:              "#306b4a"

    // Tipography --------------------------------------------------------------
    readonly property string fontMono:       "monospace"
    readonly property string fontIcon:       "Font Awesome 6 Free"

    // Form --------------------------------------------------------------------
    readonly property int radius:            8
    readonly property int radiusPill:        18
    readonly property int radiusSmall:       4

    // Animations --------------------------------------------------------------
    readonly property int animFast:          150
    readonly property int animNormal:        220

    // Position margins
    readonly property int marginTop:          15
    readonly property int marginBottom:       15
    readonly property int sidebarWidth:       350
    readonly property int marginRight:        15
}
