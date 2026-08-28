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
    readonly property string cardTitleDisplay:       isPortuguese ? "MONITOR" : "DISPLAY"

    // ── DisplayCard ──
    readonly property string displayScale:       isPortuguese ? "Escala" : "Scale"
    readonly property string displayPowerOn:     isPortuguese ? "Ligar monitor" : "Turn on"
    readonly property string displayPowerOff:    isPortuguese ? "Desligar monitor" : "Turn off"
    readonly property string displayAdvanced:    isPortuguese ? "Ajuste Avançado" : "Advanced Settings"

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

    // ── UserCard ──
    readonly property string userTabAvatar:         isPortuguese ? "Avatar" : "Avatar"
    readonly property string userTabName:           isPortuguese ? "Nome" : "Name"
    readonly property string userTabPassword:       isPortuguese ? "Senha" : "Password"
    readonly property string userAvatarActive:      isPortuguese ? "Avatar definido" : "Avatar set"
    readonly property string userAvatarNone:        isPortuguese ? "Sem avatar" : "No avatar"
    readonly property string userAvatarChange:      isPortuguese ? "Alterar" : "Change"
    readonly property string userAvatarRemove:      isPortuguese ? "Remover" : "Remove"
    readonly property string userAvatarOk:          isPortuguese ? "Avatar atualizado" : "Avatar updated"
    readonly property string userAvatarRemoved:     isPortuguese ? "Avatar removido" : "Avatar removed"
    readonly property string userAvatarError:       isPortuguese ? "Erro ao alterar avatar" : "Failed to change avatar"
    readonly property string user_name_label:       isPortuguese ? "Nome de exibicao:" : "Display name:"
    readonly property string user_name_hint:        isPortuguese
        ? "Nome exibido no login e no sistema."
        : "Name shown on login and across the system."
    readonly property string user_name_save:        isPortuguese ? "Salvar" : "Save"
    readonly property string user_name_ok:          isPortuguese ? "Nome atualizado" : "Name updated"
    readonly property string user_name_error:       isPortuguese ? "Erro ao alterar nome" : "Failed to change name"
    readonly property string userPasswordDesc:      isPortuguese
        ? "Altere sua senha do sistema."
        : "Change your system password."
    readonly property string userPasswordCurrent:   isPortuguese ? "Senha atual:" : "Current password:"
    readonly property string userPasswordNew:       isPortuguese ? "Nova senha:" : "New password:"
    readonly property string userPasswordConfirm:   isPortuguese ? "Confirmar senha:" : "Confirm password:"
    readonly property string userPasswordMismatch:  isPortuguese
        ? "As senhas nao coincidem."
        : "Passwords do not match."
    readonly property string userPasswordChange:    isPortuguese ? "Alterar senha" : "Change password"
    readonly property string userPasswordOk:        isPortuguese ? "Senha alterada" : "Password changed"
    readonly property string userPasswordError:     isPortuguese ? "Erro ao alterar senha" : "Failed to change password"
    readonly property string userPasswordHint:      isPortuguese
        ? "Será solicitada a senha do polkit."
        : "Polkit password will be requested."

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
