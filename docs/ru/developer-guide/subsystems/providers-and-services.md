---
title: Поставщики и услуги
description: Пакеты поставщика и их системные интерфейсы.
---
Проекты поставщиков предоставляют специализированные интерфейсы для потребителей пользовательского интерфейса:

- `argvus network` и `argvus bluetooth` интегрируют помощников NetworkManager и Bluetooth;
- `argvus-display` интегрирует состояние монитора с Hyprland;
- `argvus-power` интегрирует действия простоя, блокировки и питания;
- `argvus-firewall` управляет `argvus-firewall.service`;
- помощник `argvus removable-devices` в `argvus` использует UDisks2;
- помощник `argvus control-center` в `argvus` использует API поставщиков и системные интерфейсы D-Bus.

Центр управления является клиентом этих поставщиков, а не их заменой.
