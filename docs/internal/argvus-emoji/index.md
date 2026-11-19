# ARGVUS Emoji data

The offline catalog embedded by `argvus-emoji` combines [Unicode Emoji 17.0](https://www.unicode.org/Public/17.0.0/emoji/emoji-test.txt) and [CLDR 48.2](https://github.com/unicode-org/cldr/tree/release-48-2/common). The license ships with the package.

For a data refresh, retain fully qualified rows in `emoji-test.txt` order as tuples of the original sequence, group and descriptive name. Merge CLDR `annotations` and `annotationsDerived` for each ARGVUS locale, applying territory overrides where present. Index only sequences in the Unicode catalog. Remove U+FE0F from annotation lookup keys while retaining it in output sequences. Each annotation value contains the short name and pipe-separated keywords. Inherit missing fields from the English annotations.

The picker uses `XDG_STATE_HOME/argvus/emoji.json` for personal history and favorites, with the usual XDG fallback to `~/.local/state`.

## Insertion

The picker unmaps its Wayland layer before checking the destination window again. Insert uses Shift+Insert and temporarily offers the emoji on both Wayland selections because applications differ in which selection this shortcut reads. If the primary offer is unavailable, Insert uses Ctrl+V with the regular clipboard. The previous MIME offers are restored when the temporary picker offer still owns the selection. Explicit Copy writes to the regular clipboard.

## Grid rendering

The GTK grid binds catalog indices to reusable cells. Search and category changes replace the model items, while GTK creates and binds widgets for the scrollable viewport.
