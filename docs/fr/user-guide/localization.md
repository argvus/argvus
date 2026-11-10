---
title: Localisation
description: Utilisez ARGVUS en anglais ou en portugais.
---
`argvus-i18n` fournit les catalogues partagés, la commande `argvus i18n validate` et le module QML `org.argvus.i18n`. ARGVUS distribue actuellement des catalogues en anglais et en portugais.

Les applications natives et les composants shell utilisent les catalogues partagés. Une chaîne traduite peut donc être utilisée par plus d'un dépôt.

Utilisez **Centre de Contrôle → Région & Langue** pour configurer la langue, le paramètre régional, les locales système générées, le fuseau horaire, la date/heure et le clavier. Les catalogues ARGVUS actuels fournissent l'anglais et le portugais ; ceci est séparé de la liste des locales système que le système d'exploitation a générées.

Changer la langue d'affichage enregistre la préférence ARGVUS et redémarre automatiquement `argvus-control-panel.service`, de sorte que le Panneau de configuration reflète le changement immédiatement. D'autres applications et interfaces du shell peuvent nécessiter un rechargement de session ou une nouvelle session. La génération ou la modification des entrées `/etc/locale.gen` nécessite une autorisation et exécute `locale-gen` ; la sélection d'une locale qui n'est pas générée est rejetée. La disposition du clavier, la variante et le mappage des touches de la console sont des paramètres distincts et ne doivent pas être confondus avec la langue de l'interface.

## Lorsque les variables d'environnement remplacent le changement

Les applications ARGVUS détectent d'abord la langue à partir du fichier de langue ARGVUS persistant, puis `LC_ALL`, `LC_MESSAGES` et `LANG`. Les variables d'environnement locales peuvent donc donner l'impression que le Centre de Contrôle ignore un changement de langue, surtout lorsqu'un service utilisateur ou une session graphique conserve un environnement plus ancien.

Inspectez les valeurs effectives avec :

```sh
systemctl --user show-environment | rg '^(LANG|LC_|ARGVUS)='
locale
echo $LANG
localectl status
```
Si `~/.config/locale.conf` existe et définit `LANG`, `LC_ALL` ou `LC_MESSAGES`, renommez-le temporairement afin qu'il ne réintroduise plus ces valeurs. Conservez une sauvegarde au lieu de le supprimer :

```sh
mv ~/.config/locale.conf ~/.config/locale.conf.backup
```
Appliquez à nouveau la langue dans **Centre de Contrôle → Langue et Région**. Le Panneau de configuration est redémarré automatiquement ; si d'autres composants affichent encore la langue précédente, déconnectez-vous et reconnectez-vous afin que la session graphique et `systemd --user` reçoivent le nouvel environnement. Répétez les commandes de diagnostic ensuite. Voir [Variables d'environnement](/docs/reference/environment-variables/) pour les contrats techniques et la priorité.
