//! Opening a domain page: switch the route and start loading its data.

use super::*;

impl App {
  #[cfg(feature = "snapshot")]
  pub(super) fn open_snapshot(&mut self) {
    self.snapshot.lang = self.lang;
    self.snapshot.reload();
    self.route = Route::Snapshot;
  }
  #[cfg(feature = "hardware")]
  pub(super) fn open_hardware_page(&mut self, page: HardwarePage) {
    self.hardware.page = page;
    self.route = Route::Hardware;
  }

  #[cfg(feature = "services")]
  pub(super) fn open_services_page(&mut self, page: ServicePage) {
    self.services.page = page;
    self.services.reload();
    self.route = Route::Services;
  }

  #[cfg(feature = "network")]
  pub(super) fn open_network_page(&mut self, page: NetworkPage) {
    self.network.page = page;
    self.network.reload();
    self.route = Route::Network;
  }

  #[cfg(feature = "audio")]
  pub(super) fn open_audio_page(&mut self, page: AudioPage) {
    self.audio.page = page;
    self.audio.reload();
    self.route = Route::Audio;
  }

  #[cfg(feature = "bluetooth")]
  pub(super) fn open_bluetooth_page(&mut self, page: BluetoothPage) {
    self.bluetooth.page = page;
    self.bluetooth.reload();
    self.route = Route::Bluetooth;
  }

  #[cfg(feature = "boot")]
  pub(super) fn open_boot_page(&mut self, page: BootPage) {
    self.boot.page = page;
    self.boot.reload();
    self.route = Route::Boot;
  }

  #[cfg(feature = "packages")]
  pub(super) fn open_packages_page(&mut self, page: PackagesPage) {
    self.packages.open(page);
    self.route = Route::Packages;
  }

  #[cfg(feature = "storage")]
  pub(super) fn open_storage_page(&mut self, page: StoragePage) {
    self.storage.page = page;
    self.storage.reload();
    self.route = Route::Storage;
  }

  #[cfg(feature = "diagnostics")]
  pub(super) fn open_diagnostics_page(&mut self, page: DiagnosticPage) {
    self.diagnostics.page = page;
    self.diagnostics.reload();
    self.route = Route::Diagnostics;
  }

  #[cfg(feature = "session")]
  pub(super) fn open_session_page(&mut self, page: SessionPage) {
    self.session.page = page;
    self.session.reload();
    self.route = Route::Session;
  }

  #[cfg(feature = "displays")]
  pub(super) fn open_displays_page(&mut self, page: DisplayPage) {
    self.displays.page = page;
    self.displays.reload();
    self.route = Route::Displays;
  }

  #[cfg(feature = "appearance")]
  pub(super) fn open_appearance_page(&mut self, page: AppearancePage) {
    self.appearance.open(page);
    self.route = Route::Appearance;
  }
}
