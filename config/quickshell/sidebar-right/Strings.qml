pragma Singleton
import QtQuick

QtObject {
    readonly property bool isPortuguese: Qt.locale().name.startsWith("pt")

    // ── Card Titles ──
    readonly property string cardTitleUser:         isPortuguese ? "USUÁRIO" : "USER"
    readonly property string cardTitleNotifications: isPortuguese ? "NOTIFICACOES" : "NOTIFICATIONS"
    readonly property string cardTitleCalendar:      isPortuguese ? "CALENDÁRIO" : "CALENDAR"
    readonly property string cardTitleWeather:       isPortuguese ? "CLIMA" : "WEATHER"
    readonly property string cardTitleVolume:        "VOLUME"
    readonly property string cardTitleBrightness:     isPortuguese ? "BRILHO" : "BRIGHTNESS"
    readonly property string cardTitleNetwork:       isPortuguese ? "REDE" : "NETWORK"
    readonly property string cardTitleSystem:        isPortuguese ? "SISTEMA" : "SYSTEM"
    readonly property string cardTitleKeyboard:      isPortuguese ? "TECLADO" : "KEYBOARD"
    readonly property string cardTitleAppearance:    isPortuguese ? "APARENCIA" : "APPEARANCE"
    readonly property string cardTitleSpaces:        isPortuguese ? "ESPAÇOS" : "SPACES"
    readonly property string cardTitlePower:         isPortuguese ? "ENERGIA" : "POWER"

    // ── SpacesCard ──
    readonly property string spacesWaybar:           isPortuguese ? "Waybar" : "Waybar"
    readonly property string spacesGapIn:            isPortuguese ? "Gap Interno" : "Inner Gap"
    readonly property string spacesGapOut:           isPortuguese ? "Gap Externo" : "Outer Gap"
    readonly property string btnApply:               isPortuguese ? "Aplicar" : "Apply"

    // ── NotificationCard ──
    readonly property string notifNone:     isPortuguese ? "Nenhuma notificacao" : "No notifications"
    readonly property string notifRecent:   isPortuguese ? "recente(s)" : "recent"
    readonly property string notifClear:    isPortuguese ? "Limpar" : "Clear"
    readonly property string notifAllClear: isPortuguese ? "Tudo limpo" : "All clear"

    // ── NetworkCard ──
    readonly property string netTitle:        "Internet"
    readonly property string netDisabled:     isPortuguese ? "Rede desativada" : "Network disabled"
    readonly property string netConnected:    isPortuguese ? "Conectado" : "Connected"
    readonly property string netNoConnection: isPortuguese ? "Sem conexao" : "No connection"

    // ── WeatherCard ──
    readonly property string weatherLoading:   isPortuguese ? "Buscando dados..." : "Loading..."
    readonly property string weatherError:     isPortuguese ? "Sem conexao com wttr.in" : "No connection to wttr.in"
    readonly property string weatherFeelsLike: isPortuguese ? "Sensacao" : "Feels like"
    readonly property string weatherHumidity:  isPortuguese ? "Umidade" : "Humidity"
    readonly property string weatherWind:      isPortuguese ? "Vento" : "Wind"
    readonly property string weatherConfigure: isPortuguese ? "Configurar" : "Configure"
    readonly property string weatherRefresh:   isPortuguese ? "Atualizar" : "Refresh"

    // ── AppearanceCard ──
    readonly property string btnWallpaper:      isPortuguese ? "Papel de Parede" : "Wallpaper"
    readonly property string btnTheme:          isPortuguese ? "Tema" : "Theme"
    readonly property string btnAccent:         isPortuguese
        ? "Cores (SUPER + SHIFT + A)"
        : "Colors (SUPER + SHIFT + A)"
    readonly property string sysinfoTitle:      isPortuguese ? "PAINEL INFO" : "INFO PANEL"
    readonly property string sysinfoEnabled:    isPortuguese ? "Painel esquerdo ativo" : "Left panel active"
    readonly property string sysinfoDisabled:   isPortuguese ? "Painel esquerdo inativo" : "Left panel inactive"
    readonly property string idleLockTitle:     isPortuguese ? "BLOQUEIO" : "LOCK TIMER"

    // ── PowerCard ──
    readonly property var _ptProfiles: [
        { id: "power-saver",  label: "Economia",    icon: "\uf06c", desc: "Economia de energia" },
        { id: "balanced",     label: "Balanceado",  icon: "\uf24e", desc: "Padrao" },
        { id: "performance",  label: "Performance", icon: "\uf0e7", desc: "Max. desempenho" },
    ]
    readonly property var _enProfiles: [
        { id: "power-saver",  label: "Power Saver",  icon: "\uf06c", desc: "Power saving" },
        { id: "balanced",     label: "Balanced",     icon: "\uf24e", desc: "Default" },
        { id: "performance",  label: "Performance",  icon: "\uf0e7", desc: "Max performance" },
    ]
    readonly property var profiles: isPortuguese ? _ptProfiles : _enProfiles

    readonly property string powerSaverActive:    isPortuguese ? "Economia de bateria ativa" : "Battery saver active"
    readonly property string powerPerfActive:     isPortuguese ? "Maximo desempenho ativo" : "Maximum performance active"
    readonly property string powerBalancedActive: isPortuguese ? "Perfil balanceado ativo" : "Balanced profile active"

    // ── CalendarCard ──
    readonly property var monthNames: isPortuguese
        ? ["Janeiro","Fevereiro","Março","Abril","Maio","Junho","Julho","Agosto","Setembro","Outubro","Novembro","Dezembro"]
        : ["January","February","March","April","May","June","July","August","September","October","November","December"]
    readonly property var dayNames: isPortuguese
        ? ["Dom","Seg","Ter","Qua","Qui","Sex","Sáb"]
        : ["Sun","Mon","Tue","Wed","Thu","Fri","Sat"]
}
