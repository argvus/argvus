import QtQuick
import Quickshell
import Quickshell.Io

ShellRoot {
    // Toggle via: qs -c argvus-control-panel ipc call sidebar toggle
    IpcHandler {
        target: "sidebar"
        function toggle(): void { sidebarWin.sidebarVisible = !sidebarWin.sidebarVisible }
        function open():   void { sidebarWin.sidebarVisible = true }
        function close():  void { sidebarWin.sidebarVisible = false }
    }

    SidebarWindow {
        id: sidebarWin
    }
}
