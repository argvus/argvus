---
title: Interfaces D-Bus
description: Contrats D-Bus utilisés par les intégrations ARGVUS.
---
ARGVUS utilise les API D-Bus du système plutôt que d'exposer un bus pour tout le bureau. Les consommateurs importants incluent BlueZ et logind dans le Centre de Contrôle et l'écran de bienvenue, UDisks2 dans les périphériques amovibles, et ratbagd dans les paramètres matériels.

Les interfaces exactes appartiennent au fournisseur ou au service en amont. Inspectez le dépôt propriétaire et les données d'introspection installées avant de vous fier à une méthode ou propriété.
