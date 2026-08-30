# shellcheck shell=sh

# -- Locale / language detection ----------------------------------------------

locale_detect() {
  case "${LC_MESSAGES:-${LANG:-en}}" in
    pt*) echo "pt" ;;
    *)   echo "en" ;;
  esac
}

locale_is_pt() {
  case "${LC_MESSAGES:-${LANG:-en}}" in
    pt*) return 0 ;;
    *)   return 1 ;;
  esac
}
