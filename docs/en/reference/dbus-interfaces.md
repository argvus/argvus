---
title: D-Bus interfaces
description: D-Bus contracts used by ARGVUS integrations.
---

ARGVUS consumes system D-Bus APIs rather than exposing one desktop-wide bus. Important consumers include BlueZ and logind in the Control Center and greeter, UDisks2 in removable devices, and ratbagd in hardware settings.

The exact interfaces belong to the provider or upstream service. Inspect the owning repository and installed introspection data before relying on a method or property.
