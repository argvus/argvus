pragma Singleton
import QtQml

QtObject {
    id: root
    property string locale: "en-US"
    property var catalogs: ({})

    function tr(domain, key, args) {
        var values = catalogs[locale + "/" + domain] || {}
        var fallback = catalogs["en-US/" + domain] || {}
        var value = values[key] !== undefined ? values[key] : (fallback[key] !== undefined ? fallback[key] : key)
        if (args) {
            value = value.replace(/\{([^{}]+)\}/g, function (token, name) {
                return Object.prototype.hasOwnProperty.call(args, name) ? String(args[name]) : token
            })
        }
        return value
    }

    function reload() {
        return locale
    }
}
