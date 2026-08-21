-- ===================================
--  Hyprland 0.56+
--  File: ~/.config/hypr/hyprland.lua
--  By: William C. Canin
-- ===================================

-- Theme loader --------------------------------------------------------------------------------------------------------
local _home = os.getenv("HOME") or ""
local _config_home = os.getenv("ARGVUS_CONFIG_HOME")
  or os.getenv("XDG_CONFIG_HOME")
  or (_home .. "/.config")
local _system_config = os.getenv("ARGVUS_SYSTEM_CONFIG") or "/usr/share/argvus"
local _debug_session = os.getenv("ARGVUS_DEBUG") == "1"
local _state_home = _config_home .. "/argvus"
local _generated_config = (os.getenv("XDG_STATE_HOME") or (_home .. "/.local/state")) .. "/argvus/config"

local function _path_exists(path)
  local file = io.open(path, "r")
  if file then
    file:close()
    return true
  end
  return false
end

local function _first_existing(paths)
  for _, path in ipairs(paths) do
    if _path_exists(path) then
      return path
    end
  end
  return paths[1]
end

local function _config_path(relative_path)
  return _first_existing({
    _config_home .. "/" .. relative_path,
    _generated_config .. "/" .. relative_path,
    _system_config .. "/" .. relative_path,
  })
end

local function _load_user_override(relative_path)
  local path = _config_home .. "/argvus/hypr/" .. relative_path
  if _path_exists(path) then
    dofile(path)
  end
end

local function _sh(path)
  return "sh " .. string.format("%q", path)
end

local function _read_first_line(paths)
  for _, path in ipairs(paths) do
    local file = io.open(path, "r")
    if file then
      local line = file:read("*l")
      file:close()
      if line and line ~= "" then
        return line
      end
    end
  end
  return nil
end

local _theme_name = "argvus-dark"
local _active_theme = _read_first_line({
  _state_home .. "/.active-theme",
  _config_home .. "/.active-theme",
})
if _active_theme then
  _theme_name = _active_theme
end

local _theme_path = _first_existing({
  _config_home .. "/hypr/themes/" .. _theme_name .. "/theme.lua",
  _system_config .. "/hypr/themes/" .. _theme_name .. "/theme.lua",
})
local theme = dofile(_theme_path)

local _accent = "3590bd"
local _allowed_accents = {
  ["996548"] = true,
  ["3590bd"] = true,
  ["181818"] = true,
  ["7391a5"] = true,
  ["17d174"] = true,
  ["cb17d1"] = true,
  ["d1174f"] = true,
  ["d1ce17"] = true,
  ["9617d1"] = true,
  ["595959"] = true,
  ["d3d3d3"] = true,
  ["eeeeee"] = true,
}
local _accent_line = _read_first_line({
  _state_home .. "/.accent-color",
  _config_home .. "/.accent-color",
})
if _accent_line then
  local _line = _accent_line:lower():gsub("#", "")
  if _allowed_accents[_line] then _accent = _line end
end
theme.border_active = "rgba(" .. _accent .. "ff)"
theme.groupbar_active = "rgba(" .. _accent .. "ff)"

local _spaces_path = _first_existing({
  _state_home .. "/.spaces",
  _config_home .. "/.spaces",
})
local _spaces_file = io.open(_spaces_path)
if _spaces_file then
  for _line in _spaces_file:lines() do
    local _key, _val = _line:match("^([%w_]+)=(%d+)$")
    if _key == "gaps_in" then theme.gaps_in = tonumber(_val) end
    if _key == "gaps_out" then theme.gaps_out = tonumber(_val) end
  end
  _spaces_file:close()
end
-- End theme loader ----------------------------------------------------------------------------------------------------

-- Monitor -------------------------------------------------------------------------------------------------------------
hl.monitor({
  output = "", -- "" = any monitor
  mode = "preferred", -- "preferred" = any mode
  position = "auto", -- "auto" = automatic
  scale = 1,
})

-- Environment variables -----------------------------------------------------------------------------------------------

-- Cursor size
hl.env("HYPRCURSOR_SIZE", "24")
hl.env("XCURSOR_SIZE", "24")
-- Forces Qt apps to use Kvantum as their theme engine
-- hl.env("QT_STYLE_OVERRIDE", "kvantum")
-- Use qt5ct to configure Qt (font, icons, style)
hl.env("QT_QPA_PLATFORMTHEME", "qt5ct")
-- Use Hyprland's Qt Quick Controls style for Hypr* Qt/QML apps
hl.env("QT_QUICK_CONTROLS_STYLE", "org.hyprland.style")
-- Forces Firefox to run natively on Wayland
hl.env("MOZ_ENABLE_WAYLAND", "1")
-- XDGs
hl.env("XDG_CURRENT_DESKTOP", "Hyprland")
hl.env("XDG_SESSION_TYPE", "wayland")
hl.env("XDG_SESSION_DESKTOP", "Hyprland")
hl.env("XDG_CONFIG_DIRS", _system_config .. ":" .. (os.getenv("XDG_CONFIG_DIRS") or "/etc/xdg"))
-- Theme
-- hl.env("GTK2_RC_FILES", "/dev/null")
-- hl.env("GTK_THEME", "Hyprland-Dark-Teal")

-- Variables -----------------------------------------------------------------------------------------------------------
local mod = "SUPER"
local kitty_config = string.format("%q", _config_path("kitty/kitty.conf"))
local terminal = "kitty --config " .. kitty_config
-- Default File Manager: superfile (spf)
local file_manager = terminal .. " -e spf"
local rofi_config = string.format("%q", _config_path("rofi/config.rasi"))

-- Global configuration ------------------------------------------------------------------------------------------------
hl.config({
  general = {
    gaps_in = theme.gaps_in,
    gaps_out = theme.gaps_out,
    border_size = theme.border_size,

    col = {
      active_border = theme.border_active,
      inactive_border = theme.border_inactive,
    },
    layout = "dwindle",
    allow_tearing = false,
  },

  group = {
    merge_groups_on_drag = true,
    col = {
      border_active = theme.border_active,
      border_inactive = theme.border_inactive,
      border_locked_active = theme.border_active,
      border_locked_inactive = theme.border_inactive,
    },

    groupbar = {
      enabled = true,
      font_size = 14,
      render_titles = false,
      text_color = "rgba(ffffffff)",
      col = {
        active = theme.groupbar_active,
        inactive = theme.groupbar_inactive,
        locked_active = theme.groupbar_active,
        locked_inactive = theme.groupbar_inactive,
      },
    },
  },

  decoration = {
    active_opacity = 1.0,
    inactive_opacity = 1.0,
    rounding = theme.rounding,
    rounding_power = theme.rounding_power,
    fullscreen_opacity = 1.0,
    dim_inactive = false,
    dim_strength = 0.08,

    shadow = {
      enabled = true,
      range = 6,
      render_power = 2,
      color = theme.shadow_color,
      color_inactive = theme.shadow_color_inactive,
    },

    blur = {
      enabled = true,
      size = 3,
      passes = 1,
      new_optimizations = true,
      xray = false,
      noise = 0.0,
      contrast = 0.9,
      brightness = 0.8,
      vibrancy = 0.1,
      ignore_opacity = false,
      popups = false,
    },
  },

  animations = {
    enabled = true,
  },

  dwindle = {
    preserve_split = true,
  },

  master = {
    new_status = "master",
  },

  misc = {
    force_default_wallpaper = 0,
    disable_hyprland_logo = true,
    disable_splash_rendering = true,
  },

  debug = {
    disable_logs = not _debug_session,
    enable_stdout_logs = _debug_session,
    colored_stdout_logs = false,
  },

  cursor = {
    no_hardware_cursors = 2,
    use_cpu_buffer = 2,
  },

  render = {
    new_render_scheduling = true,
  },

  -- XWayland enabled/disabled
  xwayland = { enabled = true },

  input = {
    kb_layout = "br,us",
    kb_variant = "abnt2",
    kb_options = "grp:alt_shift_toggle",
    numlock_by_default = true,
    follow_mouse = 1,
    -- Mouse acceleration (disable)
    sensitivity = 0,
    accel_profile = "flat",
    --
    touchpad = {
      natural_scroll = false,
      tap_to_click = true,
      disable_while_typing = true,
      middle_button_emulation = true,
      drag_lock = true,
    },
  },
})

-- Gestures ----------------------------------------------------------------------------------------------------------
hl.gesture({ fingers = 3, direction = "horizontal", action = "workspace" })

-- XWayland ------------------------------------------------------------------------------------------------------------
-- -- Prevent invisible XWayland ghost windows from stealing focus
-- -- Use with: xwayland = { enabled = true }
hl.window_rule({
  match = { class = "^$", title = "^$", xwayland = true, float = true, fullscreen = false, pin = false },
  no_focus = true,
})

-- Animations ----------------------------------------------------------------------------------------------------------
hl.curve("myBezier", { type = "bezier", points = { { 0.05, 0.9 }, { 0.1, 1.05 } } })
hl.curve("smoothOut", { type = "bezier", points = { { 0.36, 0 }, { 0.66, -0.56 } } })
hl.curve("smoothIn", { type = "bezier", points = { { 0.25, 1 }, { 0.5, 1 } } })
hl.curve("linear", { type = "bezier", points = { { 0, 0 }, { 1, 1 } } })

hl.animation({ leaf = "global", enabled = true, speed = 1, bezier = "default" })
hl.animation({
  leaf = "windows",
  enabled = true,
  speed = 5,
  bezier = "myBezier",
})
hl.animation({
  leaf = "windowsIn",
  enabled = true,
  speed = 5,
  bezier = "myBezier",
  style = "popin 80%",
})
hl.animation({
  leaf = "windowsOut",
  enabled = true,
  speed = 4,
  bezier = "smoothOut",
  style = "popin 80%",
})
hl.animation({ leaf = "border", enabled = true, speed = 10, bezier = "default" })
hl.animation({ leaf = "fade", enabled = true, speed = 5, bezier = "smoothIn" })
hl.animation({
  leaf = "fadeOut",
  enabled = true,
  speed = 4,
  bezier = "smoothOut",
})
hl.animation({
  leaf = "workspaces",
  enabled = true,
  speed = 5,
  bezier = "myBezier",
  style = "slide",
})

-- Blur ----------------------------------------------------------------------------------------------------------------
hl.layer_rule({ match = { namespace = "waybar" }, blur = true })
hl.layer_rule({ match = { namespace = "quickshell" }, blur = true })
hl.layer_rule({ match = { namespace = "rofi" }, blur = true })
hl.layer_rule({ match = { namespace = "dunst" }, blur = true })

-- Window Rules  -------------------------------------------------------------------------------------------------------
hl.window_rule({
  match = { class = "org.gnome.Nautilus" },
  float = false,
  size = "1399 920",
  center = true,
  opacity = theme.file_manager_opacity,
})
hl.window_rule({
  match = { class = "hyprfm" },
  float = false,
  size = "1399 920",
  center = true,
  opacity = theme.file_manager_opacity,
})
hl.window_rule({
  match = { class = ".*pwvucontrol.*" },
  float = true,
  size = "700 450",
  center = true,
})
hl.window_rule({ match = { class = ".*pavucontrol.*" }, float = true })
hl.window_rule({ match = { class = "org.gnome.FileRoller" }, float = true })
hl.window_rule({ match = { class = "org.gnome.Calculator" }, float = true })
hl.window_rule({ match = { class = "nm-connection-editor" }, float = true })
hl.window_rule({
  match = { class = "kitty", title = ".*nmtui.*" },
  float = true,
  size = "900 900",
  center = true,
})
hl.window_rule({
  match = { class = "kitty", title = ".*nvim.*" },
  opacity = theme.term_opacity,
})
hl.window_rule({ match = { class = "blueman-manager" }, float = true })
hl.window_rule({ match = { class = "xdg-desktop-portal-gtk" }, float = true })
hl.window_rule({
  match = { class = "firefox", title = ".*Picture-in-Picture.*" },
  float = true,
  pin = true,
  size = "420 320",
  center = true,
  keep_aspect_ratio = true,
})
hl.window_rule({ match = { class = "mpv" }, float = true })

-- Transparency at the terminals ---------------------------------------------------------------------------------------
hl.window_rule({ match = { class = "kitty" }, opacity = theme.term_opacity })
hl.window_rule({ match = { class = "foot" }, opacity = theme.term_opacity })
hl.window_rule({ match = { class = "Alacritty" }, opacity = theme.term_opacity })

-- ================ Keybindings ================

-- Moving between windows (Using: snappy-switcher) ---------------------------------------------------------------------
hl.bind("ALT + Tab", hl.dsp.exec_cmd("snappy-switcher next --mod alt"))
hl.bind("ALT + SHIFT + Tab", hl.dsp.exec_cmd("snappy-switcher prev --mod alt"))

-- All cheatsheets -----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + slash", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/cheatsheets.sh"))))

-- Cheatsheets Kitty ---------------------------------------------------------------------------------------------------
hl.bind(mod .. " + CTRL + slash", hl.dsp.exec_cmd(_sh(_config_path("kitty/scripts/cheatsheets.sh"))))

-- Open Terminal -------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + Return", hl.dsp.exec_cmd(terminal))

-- File Manager --------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + Space", hl.dsp.exec_cmd(file_manager))

-- Removable storage ---------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + S", hl.dsp.exec_cmd("argvus-storage menu"))

-- Sidebar Settings ----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + comma", hl.dsp.exec_cmd("qs -c sidebar-right ipc call sidebar toggle"))
hl.bind("mouse:274", hl.dsp.exec_cmd("qs -c sidebar-right ipc call sidebar toggle"))

-- Wallpaper Picker ----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + Y", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/wallpaper-pick.sh"))))

-- Theme switcher ------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + T", hl.dsp.exec_cmd(_sh(_config_path("argvus/sh/theme-switch.sh"))))

-- Accent color --------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + A", hl.dsp.exec_cmd(_sh(_config_path("argvus/sh/accent-switch.sh"))))

-- Brightness -----------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + B", hl.dsp.exec_cmd(_sh(_config_path("argvus/sh/brightness-switch.sh"))))

-- Weather location ----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + W", hl.dsp.exec_cmd(_sh(_config_path("argvus/sh/weather-location.sh"))))

-- GTK Theme Dark/Light ------------------------------------------------------------------------------------------------
hl.bind(mod .. " + F5", hl.dsp.exec_cmd(_sh(_config_path("argvus/sh/toggle-mode.sh"))))

-- Finder --------------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + D", hl.dsp.exec_cmd('rofi -config ' .. rofi_config .. ' -show drun -display-drun "drun"'))
-- hl.bind(mod .. " + D", hl.dsp.exec_cmd("wofi"))

-- Maximize Window -----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + S", hl.dsp.window.fullscreen({ mode = "maximized", action = "toggle" }))

-- Closed Window -------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + Q", hl.dsp.window.close())

-- Enable/Disable Floating Window --------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + space", function()
  hl.dispatch(hl.dsp.window.float({ action = "toggle" }))
  hl.exec_scheduled_prop_refresh_immediately()

  local win = hl.get_active_window()

  if win and win.floating then
    hl.dispatch(hl.dsp.window.resize({
      x = 1399,
      y = 920,
    }))

    hl.dispatch(hl.dsp.window.center())
  end
end)

-- Window fullscreen ---------------------------------------------------------------------------------------------------
hl.bind(mod .. " + F", hl.dsp.window.fullscreen())

-- Split vertical/horizontal -------------------------------------------------------------------------------------------
hl.bind(mod .. " + E", hl.dsp.layout("togglesplit"))

-- Tabbed windows ------------------------------------------------------------------------------------------------------
-- Groups all windows in the current workspace into tabs.
hl.bind(mod .. " + W", function()
  local active = hl.get_active_window()
  if not active then
    return
  end

  local ws = active.workspace.id

  hl.dispatch(hl.dsp.group.toggle())

  for _, w in ipairs(hl.get_windows()) do
    if w.workspace.id == ws and w.address ~= active.address then
      hl.dispatch(hl.dsp.focus({
        window = "address:" .. w.address,
      }))

      for _, dir in ipairs({ "l", "r", "u", "d" }) do
        pcall(function()
          hl.dispatch(hl.dsp.window.move({
            into_group = dir,
          }))
        end)
      end
    end
  end

  hl.dispatch(hl.dsp.focus({
    window = "address:" .. active.address,
  }))
end)

-- Navigate between tabs -----------------------------------------------------------------------------------------------
hl.bind(mod .. " + Tab", hl.dsp.group.next())

-- Navigate between windows --------------------------------------------------------------------------------------------
hl.bind(mod .. " + left", hl.dsp.focus({ direction = "left" }))
hl.bind(mod .. " + right", hl.dsp.focus({ direction = "right" }))
hl.bind(mod .. " + up", hl.dsp.focus({ direction = "up" }))
hl.bind(mod .. " + down", hl.dsp.focus({ direction = "down" }))

-- Cycle focus between all windows in current workspace (including floating) -------------------------------------------
hl.bind(mod .. " + CTRL + right", function()
  local wins = hl.get_windows()
  local active = hl.get_active_window()
  if not active then
    return
  end

  -- Filter only current workspace
  local ws_wins = {}
  for _, w in ipairs(wins) do
    if w.workspace.id == active.workspace.id then
      table.insert(ws_wins, w)
    end
  end

  for i, w in ipairs(ws_wins) do
    if w.address == active.address then
      local next = ws_wins[i + 1] or ws_wins[1]
      hl.dispatch(hl.dsp.focus({ window = "address:" .. next.address }))
      break
    end
  end
end)

hl.bind(mod .. " + CTRL + left", function()
  local wins = hl.get_windows()
  local active = hl.get_active_window()
  if not active then
    return
  end

  local ws_wins = {}
  for _, w in ipairs(wins) do
    if w.workspace.id == active.workspace.id then
      table.insert(ws_wins, w)
    end
  end

  for i, w in ipairs(ws_wins) do
    if w.address == active.address then
      local prev = ws_wins[i - 1] or ws_wins[#ws_wins]
      hl.dispatch(hl.dsp.focus({ window = "address:" .. prev.address }))
      break
    end
  end
end)

-- Cycle workspaces in loop (like GNOME) -------------------------------------------------------------------------------
local function get_sorted_workspaces()
  local workspaces = hl.get_workspaces()
  local ws_ids = {}

  for _, ws in ipairs(workspaces) do
    if ws.id > 0 then
      table.insert(ws_ids, ws.id)
    end
  end

  table.sort(ws_ids)
  return ws_ids
end

local function cycle_workspace(offset)
  local active = hl.get_active_workspace()
  if not active then
    return
  end

  local ws_ids = get_sorted_workspaces()
  if #ws_ids == 0 then
    return
  end

  for i, id in ipairs(ws_ids) do
    if id == active.id then
      local target_index = ((i - 1 + offset) % #ws_ids) + 1

      hl.dispatch(hl.dsp.focus({
        workspace = ws_ids[target_index],
      }))

      return
    end
  end
end

local function workspace_next()
  cycle_workspace(1)
end

local function workspace_prev()
  cycle_workspace(-1)
end

hl.bind("CTRL + ALT + right", workspace_next)
hl.bind("CTRL + ALT + left", workspace_prev)
hl.bind("mouse:276", workspace_next)
hl.bind("mouse:275", workspace_prev)

-- Move window float ---------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + left", hl.dsp.window.move({ direction = "left" }))
hl.bind(mod .. " + SHIFT + right", hl.dsp.window.move({ direction = "right" }))
hl.bind(mod .. " + SHIFT + up", hl.dsp.window.move({ direction = "up" }))
hl.bind(mod .. " + SHIFT + down", hl.dsp.window.move({ direction = "down" }))

-- Workspaces 1–9 ------------------------------------------------------------------------------------------------------
for i = 1, 9 do
  hl.bind(mod .. " + " .. i, hl.dsp.focus({ workspace = i }))
  hl.bind(mod .. " + SHIFT + " .. i, hl.dsp.window.move({ workspace = i }))
end

-- Volume --------------------------------------------------------------------------------------------------------------
hl.bind(
  "XF86AudioRaiseVolume",
  hl.dsp.exec_cmd("wpctl set-volume -l 1.5 @DEFAULT_AUDIO_SINK@ 5%+"),
  { locked = true, repeating = true }
)
hl.bind(
  "XF86AudioLowerVolume",
  hl.dsp.exec_cmd("wpctl set-volume @DEFAULT_AUDIO_SINK@ 5%-"),
  { locked = true, repeating = true }
)
hl.bind("XF86AudioMute", hl.dsp.exec_cmd("wpctl set-mute @DEFAULT_AUDIO_SINK@ toggle"), { locked = true })

-- Brightness ----------------------------------------------------------------------------------------------------------
hl.bind("XF86MonBrightnessUp", hl.dsp.exec_cmd("brightnessctl set +5%"), { locked = true, repeating = true })
hl.bind("XF86MonBrightnessDown", hl.dsp.exec_cmd("brightnessctl set 5%-"), { locked = true, repeating = true })

-- Multimidia ----------------------------------------------------------------------------------------------------------
hl.bind("XF86AudioPlay", hl.dsp.exec_cmd("playerctl play-pause"))
hl.bind("XF86AudioPause", hl.dsp.exec_cmd("playerctl pause"))
hl.bind("XF86AudioNext", hl.dsp.exec_cmd("playerctl next"))
hl.bind("XF86AudioPrev", hl.dsp.exec_cmd("playerctl previous"))
hl.bind("XF86AudioStop", hl.dsp.exec_cmd("playerctl stop"))

-- Turn the monitor off/on ---------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + M", hl.dsp.dpms({ action = "toggle" }))

-- Default browser -----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + B", hl.dsp.exec_cmd("xdg-open https://"))

-- Screen recording ----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + G", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/screenshot.sh")) .. " --video-full"))
hl.bind(mod .. " + SHIFT + G", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/screenshot.sh")) .. " --video-full-stop"))

-- Clipboard history ---------------------------------------------------------------------------------------------------
hl.bind(mod .. " + H", hl.dsp.exec_cmd("cliphist list | rofi -config " .. rofi_config .. " -dmenu -i -p Clipboard | cliphist decode | wl-copy"))
hl.bind(mod .. " + SHIFT + H", hl.dsp.exec_cmd('cliphist wipe && notify-send "Clipboard" "History erased!"'))

-- Screenshot / Print --------------------------------------------------------------------------------------------------
hl.bind("Print", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/screenshot.sh")) .. " --image-region"))
hl.bind(mod .. " + Print", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/screenshot.sh")) .. " --image-window"))
hl.bind(mod .. " + SHIFT + Print", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/screenshot.sh")) .. " --image-full"))

-- Mode Resize Window (keyboard) ---------------------------------------------------------------------------------------
local _in_resize = false

hl.bind(mod .. " + R", function()
  local w = hl.get_active_window()
  if w == nil then
    return
  end

  -- Toggle off
  if _in_resize then
    _in_resize = false
    hl.dispatch(hl.dsp.submap("reset"))
    return
  end

  -- Only enter resize if window is floating
  if not w.floating then
    return
  end

  _in_resize = true
  hl.dispatch(hl.dsp.submap("resize"))
end)

hl.define_submap("resize", function()
  hl.bind("right", hl.dsp.window.resize({ x = 20, y = 0, relative = true }), { repeating = true })
  hl.bind("left", hl.dsp.window.resize({ x = -20, y = 0, relative = true }), { repeating = true })
  hl.bind("down", hl.dsp.window.resize({ x = 0, y = 20, relative = true }), { repeating = true })
  hl.bind("up", hl.dsp.window.resize({ x = 0, y = -20, relative = true }), { repeating = true })

  -- Move window
  hl.bind("SHIFT + right", hl.dsp.window.move({ x = 20, y = 0, relative = true }), { repeating = true })
  hl.bind("SHIFT + left", hl.dsp.window.move({ x = -20, y = 0, relative = true }), { repeating = true })
  hl.bind("SHIFT + down", hl.dsp.window.move({ x = 0, y = 20, relative = true }), { repeating = true })
  hl.bind("SHIFT + up", hl.dsp.window.move({ x = 0, y = -20, relative = true }), { repeating = true })

  -- Escape/Return: exits submap
  hl.bind("escape", function()
    _in_resize = false
    hl.dispatch(hl.dsp.submap("reset"))
  end)
  hl.bind("Return", function()
    _in_resize = false
    hl.dispatch(hl.dsp.submap("reset"))
  end)
end)

-- Mode Resize Window (witch mouse) -----------------------------------------------------------------------------------------
hl.bind(mod .. " + mouse:272", hl.dsp.window.drag(), { mouse = true })
hl.bind(mod .. " + mouse:273", hl.dsp.window.resize(), { mouse = true })

-- Emoji picker --------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + period", hl.dsp.exec_cmd("rofimoji --action clipboard --clipboarder wl-copy --typer wtype"))

-- Color Picker --------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + P", hl.dsp.exec_cmd("hyprpicker -a"))

-- Calculator ----------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + C", hl.dsp.exec_cmd("rofi -config " .. rofi_config .. " -show calc -modi calc -no-show-match -no-sort"))

-- Exit Hyprland -------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + escape", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/power-menu.sh"))))

-- Lock session --------------------------------------------------------------------------------------------------------
hl.bind(mod .. " + L", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/power-menu.sh")) .. " --lock"))

-- Reload Hyprland -----------------------------------------------------------------------------------------------------
hl.bind(mod .. " + SHIFT + R", hl.dsp.exec_cmd(_sh(_config_path("hypr/scripts/init.sh")) .. " --reload"))

-- User overrides ------------------------------------------------------------------------------------------------------
_load_user_override("monitors.lua")
_load_user_override("rules.lua")
_load_user_override("bindings.lua")
_load_user_override("user.lua")

-- Autostart -----------------------------------------------------------------------------------------------------------
hl.on("hyprland.start", function()
  hl.exec_cmd(_sh(_config_path("hypr/scripts/init.sh")) .. " --started")
end)
