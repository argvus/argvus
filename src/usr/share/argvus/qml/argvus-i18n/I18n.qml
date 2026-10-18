pragma Singleton
import QtQml

QtObject {
    id: root
    property string locale: "en-US"
    property var catalogs: ({})

    function tr(domain, key, args) {
        var values = catalogs[domain] || {}
        var fallback = catalogs["en-US/" + domain] || {}
        var value = values[key] !== undefined ? values[key] : (fallback[key] !== undefined ? fallback[key] : key)
        if (args) {
            Object.keys(args).forEach(function (name) {
                value = value.split("{" + name + "}").join(String(args[name]))
            })
        }
        return value
    }

    function reload() {
        return locale
    }
}
