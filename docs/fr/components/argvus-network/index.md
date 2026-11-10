---
title: Réseautage et Bluetooth
description: Gérer NetworkManager et Bluetooth via ARGVUS.
---
La sous-commande `argvus network` signale et contrôle NetworkManager, et `argvus bluetooth` signale et contrôle l'état et la disponibilité de l'adaptateur Bluetooth. Les deux s'exécutent en processus à l'intérieur de `argvus` ; `networkctl` et `bluetoothctl` restent comme alias. Le Panneau de configuration et Waybar appellent ces commandes.

Utilisez `--help` sur l'une ou l'autre commande pour les sous-commandes installées. L'intégration de l'interface utilisateur réseau et Bluetooth est séparée des gestionnaires avancés optionnels tels que Blueman.

Le Centre de contrôle sépare le statut du réseau des pages de configuration : **Réseau → Statut** résume l'état actuel, tandis que les interfaces, Wi-Fi, Ethernet, VPN, DNS, proxy et pare-feu sont des sections distinctes lorsque leurs fournisseurs sont disponibles. **Bluetooth → État**, **Appareils** et **Appairer** couvrent l'état de l'adaptateur et les actions sur les appareils. Ces pages exécutent des actions système soutenues par des fournisseurs ; la barre des tâches et le Panneau de configuration exposent la surface rapide de statut/action.

Les modifications du réseau et du Bluetooth peuvent nécessiter une autorisation et peuvent s'appliquer immédiatement via NetworkManager ou le service Bluetooth. Elles ne sont pas identiques au changement de l'indicateur de barre des tâches ARGVUS.

Le Centre de contrôle envoie les mots de passe Wi-Fi à nmcli via l'entrée standard avec `--ask` ; les mots de passe sont exclus des arguments du processus et masqués dans les erreurs de commande retournées.

## Gestion du réseau en ligne de commande

Utilisez `argvus networkctl` pour interroger et contrôler le réseau sans l'interface graphique :

```sh
# Show network status
argvus networkctl status

# Toggle networking on/off
argvus networkctl enable
argvus networkctl disable

# Wi-Fi control
argvus networkctl wifi on
argvus networkctl wifi off

# Open the network manager
argvus networkctl manager
```
## Gestion du Bluetooth en ligne de commande

Utilisez `argvus bluetoothctl` pour l'adaptateur Bluetooth :

```sh
# Show the adapter state (key=value lines)
argvus bluetoothctl status

# Exit 0 when an adapter is available, 1 otherwise
argvus bluetoothctl available

# Turn the adapter on or off, or toggle it
argvus bluetoothctl enable
argvus bluetoothctl disable
argvus bluetoothctl toggle

# Open the Bluetooth manager (Blueman)
argvus bluetoothctl manager
```
Lorsque le noyau ne liste aucun contrôleur dans `/sys/class/bluetooth`, `status` répond immédiatement à `status=no-controller`, sans demander à `bluetoothctl` (qui attendrait autrement BlueZ lorsque le service est arrêté). Le module de la barre des tâches et les cartes du Panneau de configuration lisent le même statut.

Utilisez `--help` sur l'une ou l'autre commande pour voir des options supplémentaires et des sous-commandes spécifiques à votre version installée.

Alors que la boîte de dialogue du mot de passe Wi-Fi est ouverte, les mises à jour de balayage conservent le SSID sélectionné et le type de sécurité. Si ce réseau disparaît ou si sa sécurité change, la confirmation s'arrête avec une erreur et nécessite une nouvelle sélection.

Le DNS manuel envoie les serveurs IPv4 et IPv6 à leurs paramètres respectifs de NetworkManager avant de mettre à jour le fichier de résolveur. Les liens symboliques du résolveur gérés par le service sont préservés en refusant le remplacement du fichier. Un retour réussi au DNS automatique restaure et supprime la sauvegarde, de sorte qu'un cycle manuel ultérieur enregistre l'état actuel du résolveur.

La fermeture de l'éditeur de profil réseau attend que son travail en cours soit terminé, de sorte que l'enregistrement ou la suppression d'un profil ne peut pas être interrompu en fermant l'éditeur. Les actualisations en arrière-plan restent annulables.

Les demandes de connexion Wi-Fi partagent le garde-action réseau : une action en cours ou un travail de l'éditeur de profil se termine avant qu'une autre modification puisse commencer. Les messages de progression et d'achèvement suivent la langue sélectionnée.

La confirmation de suppression Bluetooth conserve l'adresse d'origine de l'appareil même lorsque la liste des appareils est actualisée. Les pourcentages de batterie provenant de bluetoothctl sont reconnus, et les requêtes de détails des appareils s'exécutent avec une concurrence limitée et des délais individuels.

Mot de passe Wi-Fi, les champs de recherche DNS manuelle et réseau acceptent le collage entre crochets. Les contrôles collés et les retours à la ligne sont omis. Les raccourcis clavier uniquement Ctrl ou uniquement Alt non supportés n’insèrent pas de lettres. Les éditeurs acceptent les événements AltGr imprimables codés en Ctrl+Alt ; Ctrl+S enregistre seulement sans Alt. L’éditeur de texte partagé conserve la navigation verticale dans les colonnes visuelles et aligne le défilement horizontal pour les caractères larges.
