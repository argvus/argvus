---
title: Interfaces D-Bus
description: Contratos D-Bus utilizados por las integraciones de ARGVUS.
---
ARGVUS consume APIs del sistema D-Bus en lugar de exponer un bus único para todo el escritorio. Los consumidores importantes incluyen BlueZ y logind en el Centro de Control y en el saludo inicial, UDisks2 en dispositivos extraíbles, y ratbagd en la configuración de hardware.

Las interfaces exactas pertenecen al proveedor o al servicio upstream. Inspeccione el repositorio propietario y los datos de introspección instalados antes de depender de un método o propiedad.
