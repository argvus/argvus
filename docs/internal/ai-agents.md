# AI Agents selector

The optional `argvus-ai` package owns availability checks, the `ai` translation
catalog and agent launches. Its menu calls `argvus ai-agents --selector [ID…]`,
which opens the GTK 4 helper packaged with `argvus`.

The selector uses Wayland layer-shell with the `argvus-ai-agents` namespace.
It resolves panel borders, colors and backdrop opacity from the shared
appearance configuration, and Hyprland applies the blur and fade rules.
Brand images are packaged under `/usr/share/argvus/ai-agents` and fit into
uniform logical slots while preserving their proportions.

Selecting a card closes the selector and invokes `argvus-ai launch ID`. That
command creates a separate Kitty session for the agent.
