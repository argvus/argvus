---
title: D-Bus 接口
description: ARGVUS 集成使用的 D-Bus 合约。
---
ARGVUS 使用系统 D-Bus API，而不是暴露一个桌面范围的总线。重要的使用者包括控制中心和问候界面中的 BlueZ 和 logind，可移动设备中的 UDisks2，以及硬件设置中的 ratbagd。

具体接口属于提供者或上游服务。在依赖某个方法或属性之前，请检查所属的代码仓库和已安装的 introspection 数据。
