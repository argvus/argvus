# sidebar-right — Quickshell sidebar for Hyprland

Standalone right-hand sidebar with: Calendar, CPU/RAM/GPU, Keyboard, and Power Profile.

## Installation

```bash
# 1. Copy to ~/.config/quickshell/
cp -r sidebar-right ~/.config/quickshell/sidebar-right

# 2. Test manually
qs -c sidebar-right
```

## Waybar integration

Add a button to your `~/.config/waybar/config`:

```json
"custom/sidebar": {
    "format": "󰕰",
    "tooltip": false,
    "on-click": "qs -c sidebar-right ipc call sidebar toggle"
}
```

And in `style.css`:
```css
#custom-sidebar {
    padding: 0 10px;
    font-size: 16px;
    color: #3aa9f0;
    border-radius: 8px;
}
#custom-sidebar:hover {
    background: rgba(58,169,240,0.15);
}
```

## hyprland.lua integration

Add to your `~/.config/hypr/hyprland.lua`:

```lua
-- Right-hand sidebar (Quickshell)
hl.bind("SUPER", "S", "exec",
    "qs -c sidebar-right ipc call sidebar toggle")

-- Blur for the panel (optional, but nice)
hl.rule.layer("noanim,blur,blurpopups", "sidebar-right")
```

> **Note about the layer name:** Quickshell registers the layer with the config name.
> If you need to adjust it, run `hyprctl layers` to see the exact name and use it in the rule above.

## Autostart

To start it together with Hyprland (add to your `hyprland.lua`):

```lua
hl.exec("qs -c sidebar-right")
```

## Dependencies

| Package           | Used for               |
|-------------------|------------------------|
| `quickshell`      | shell framework        |
| `nvidia-smi`      | GPU stats (comes with NVIDIA drivers) |
| `powerprofilesctl`| power profiles         |
| `hyprctl`         | switch keyboard layout |

## Tweaks

### Keyboard layout switching

`KeyboardCard.qml` switches the layout with `hyprctl switchxkblayout all next`,
which cycles through the `br`/`us` groups on every keyboard — so it works on
any machine without editing a device name. The card reads the current keymap
from `hyprctl devices` only to show which layout is active.

### Colors

All the main colors live in `SidebarWindow.qml` and in the cards.
The default accent color is `#3aa9f0` (blue), compatible with your current theme (border `3aa99f`).
