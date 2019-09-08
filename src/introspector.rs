use crate::clone;
use crate::operation::{OperationFuture, Value};
use libpulse_binding::callbacks::ListResult;
use libpulse_binding::context::introspect;
use libpulse_binding::def::PortAvailable;
use libpulse_binding::proplist::Proplist;
use libpulse_binding::time::MicroSeconds;
use libpulse_binding::volume::{ChannelVolumes, Volume};
use libpulse_binding::{channelmap, def, format, sample};
use std::cell::RefCell;
use std::rc::Rc;

pub struct SinkPortInfo {
  /// Name of the sink.
  pub name: Option<String>,
  /// Description of this sink.
  pub description: Option<String>,
  /// The higher this value is, the more useful this port is as a default.
  pub priority: u32,
  /// A flag indicating availability status of this port.
  pub available: PortAvailable,
}

impl<'a> From<&'a introspect::SinkPortInfo<'a>> for SinkPortInfo {
  fn from(item: &'a introspect::SinkPortInfo<'a>) -> Self {
    SinkPortInfo {
      name: item.name.as_ref().map(|cow| cow.to_string()),
      description: item.description.as_ref().map(|cow| cow.to_string()),
      priority: item.priority,
      available: item.available,
    }
  }
}

impl<'a> From<&'a Box<introspect::SinkPortInfo<'a>>> for SinkPortInfo {
  fn from(item: &'a Box<introspect::SinkPortInfo<'a>>) -> Self {
    SinkPortInfo {
      name: item.name.as_ref().map(|cow| cow.to_string()),
      description: item.description.as_ref().map(|cow| cow.to_string()),
      priority: item.priority,
      available: item.available,
    }
  }
}

pub struct SinkInfo {
  /// Name of the sink.
  pub name: Option<String>,
  /// Index of the sink.
  pub index: u32,
  /// Description of this sink.
  pub description: Option<String>,
  /// Sample spec of this sink.
  pub sample_spec: sample::Spec,
  /// Channel map.
  pub channel_map: channelmap::Map,
  /// Index of the owning module of this sink, or `None` if is invalid.
  pub owner_module: Option<u32>,
  /// Volume of the sink.
  pub volume: ChannelVolumes,
  /// Mute switch of the sink.
  pub mute: bool,
  /// Index of the monitor source connected to this sink.
  pub monitor_source: u32,
  /// The name of the monitor source.
  pub monitor_source_name: Option<String>,
  /// Length of queued audio in the output buffer.
  pub latency: MicroSeconds,
  /// Driver name.
  pub driver: Option<String>,
  /// Flags.
  pub flags: def::SinkFlagSet,
  /// Property list.
  pub proplist: Proplist,
  /// The latency this device has been configured to.
  pub configured_latency: MicroSeconds,
  /// Some kind of “base” volume that refers to unamplified/unattenuated volume in the context of
  /// the output device.
  pub base_volume: Volume,
  /// State.
  pub state: def::SinkState,
  /// Number of volume steps for sinks which do not support arbitrary volumes.
  pub n_volume_steps: u32,
  /// Card index, or `None` if invalid.
  pub card: Option<u32>,
  /// Set of available ports.
  pub ports: Vec<SinkPortInfo>,
  // Pointer to active port in the set, or None.
  pub active_port: Option<SinkPortInfo>,
  /// Set of formats supported by the sink.
  pub formats: Vec<format::Info>,
}

impl<'a> From<&'a introspect::SinkInfo<'a>> for SinkInfo {
  fn from(item: &'a introspect::SinkInfo<'a>) -> Self {
    SinkInfo {
      name: item.name.as_ref().map(|cow| cow.to_string()),
      index: item.index,
      description: item.description.as_ref().map(|cow| cow.to_string()),
      sample_spec: item.sample_spec,
      channel_map: item.channel_map,
      owner_module: item.owner_module,
      volume: item.volume,
      mute: item.mute,
      monitor_source: item.monitor_source,
      monitor_source_name: item.monitor_source_name.as_ref().map(|cow| cow.to_string()),
      latency: item.latency,
      driver: item.driver.as_ref().map(|cow| cow.to_string()),
      flags: item.flags,
      proplist: item.proplist.clone(),
      configured_latency: item.configured_latency,
      base_volume: item.base_volume,
      state: item.state,
      n_volume_steps: item.n_volume_steps,
      card: item.card,
      ports: item.ports.iter().map(From::from).collect(),
      active_port: item.active_port.as_ref().map(From::from),
      formats: item.formats.clone(),
    }
  }
}

pub struct ServerInfo {
  /// User name of the daemon process.
  pub user_name: Option<String>,
  /// Host name the daemon is running on.
  pub host_name: Option<String>,
  /// Version string of the daemon.
  pub server_version: Option<String>,
  /// Server package name (usually “pulseaudio”).
  pub server_name: Option<String>,
  /// Default sample specification.
  pub sample_spec: sample::Spec,
  /// Name of default sink.
  pub default_sink_name: Option<String>,
  /// Name of default source.
  pub default_source_name: Option<String>,
  /// A random cookie for identifying this instance of PulseAudio.
  pub cookie: u32,
  /// Default channel map.
  pub channel_map: channelmap::Map,
}

impl<'a> From<&'a introspect::ServerInfo<'a>> for ServerInfo {
  fn from(info: &'a introspect::ServerInfo<'a>) -> Self {
    ServerInfo {
      user_name: info.user_name.as_ref().map(|cow| cow.to_string()),
      host_name: info.host_name.as_ref().map(|cow| cow.to_string()),
      server_version: info.server_version.as_ref().map(|cow| cow.to_string()),
      server_name: info.server_name.as_ref().map(|cow| cow.to_string()),
      sample_spec: info.sample_spec,
      default_sink_name: info.default_sink_name.as_ref().map(|cow| cow.to_string()),
      default_source_name: info.default_source_name.as_ref().map(|cow| cow.to_string()),
      cookie: info.cookie,
      channel_map: info.channel_map,
    }
  }
}

pub struct Introspector {
  pub(crate) introspector: introspect::Introspector,
}

impl Introspector {
  pub fn get_sink_info_list(&self) -> OperationFuture<Vec<SinkInfo>> {
    let result = Rc::new(RefCell::new(Value::new(Some(vec![]))));

    let op = Rc::new(self.introspector.get_sink_info_list(
      clone!(result => move |list| match list {
        ListResult::Item(item) => {
          result
            .borrow_mut()
            .value
            .as_mut()
            .unwrap()
            .push(item.into());
        }
        ListResult::Error => {
          result.borrow_mut().error = true;
        }
        ListResult::End => {}
      }),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }
  pub fn get_sink_info_by_name(&self, name: &str) -> OperationFuture<Option<SinkInfo>> {
    let result = Rc::new(RefCell::new(Value::new(Some(None))));

    let op = Rc::new(self.introspector.get_sink_info_by_name(name,
      clone!(result => move |list| match list {
        ListResult::Item(item) => {
          result
            .borrow_mut()
            .value
            .as_mut()
            .unwrap()
            .replace(item.into());
        }
        ListResult::Error => {
          result.borrow_mut().error = true;
        }
        ListResult::End => {}
      }),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }

  pub fn get_server_info(&self) -> OperationFuture<ServerInfo> {
    let result = Rc::new(RefCell::new(Value::new(None)));

    let op = Rc::new(self.introspector.get_server_info(
      clone!(result => move |info| {
        result
          .borrow_mut()
          .value = Some(info.into());
      })
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }

  /// Sets the volume of a sink device specified by its index.
  ///
  /// Panics on error, i.e. invalid arguments or state.
  pub fn set_sink_volume_by_index(
    &mut self,
    index: u32,
    volume: &ChannelVolumes,
  ) -> OperationFuture<()> {
    let result = Rc::new(RefCell::new(Value::new(Some(()))));

    let op = Rc::new(self.introspector.set_sink_volume_by_index(
      index,
      volume,
      Some(Box::new(clone!(result => move |success| {
        result.borrow_mut().error = !success;
      }))),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }

  /// Sets the volume of a sink device specified by its name.
  ///
  /// Panics on error, i.e. invalid arguments or state.
  pub fn set_sink_volume_by_name(
    &mut self,
    name: &str,
    volume: &ChannelVolumes,
  ) -> OperationFuture<()> {
    let result = Rc::new(RefCell::new(Value::new(Some(()))));

    let op = Rc::new(self.introspector.set_sink_volume_by_name(
      name,
      volume,
      Some(Box::new(clone!(result => move |success| {
        result.borrow_mut().error = !success;
      }))),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }

  /// Sets the mute switch of a sink device specified by its index.
  ///
  /// Panics on error, i.e. invalid arguments or state.
  pub fn set_sink_mute_by_index(&mut self, index: u32, mute: bool) -> OperationFuture<()> {
    let result = Rc::new(RefCell::new(Value::new(Some(()))));

    let op = Rc::new(self.introspector.set_sink_mute_by_index(
      index,
      mute,
      Some(Box::new(clone!(result => move |success| {
        result.borrow_mut().error = !success;
      }))),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }

  /// Sets the mute switch of a sink device specified by its name.
  ///
  /// Panics on error, i.e. invalid arguments or state.
  pub fn set_sink_mute_by_name(&mut self, name: &str, mute: bool) -> OperationFuture<()> {
    let result = Rc::new(RefCell::new(Value::new(Some(()))));

    let op = Rc::new(self.introspector.set_sink_mute_by_name(
      name,
      mute,
      Some(Box::new(clone!(result => move |success| {
        result.borrow_mut().error = !success;
      }))),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }

  /// Changes the profile of a sink.
  ///
  /// Panics on error, i.e. invalid arguments or state.
  pub fn set_sink_port_by_index(&mut self, index: u32, port: &str) -> OperationFuture<()> {
    let result = Rc::new(RefCell::new(Value::new(Some(()))));

    let op = Rc::new(self.introspector.set_sink_port_by_index(
      index,
      port,
      Some(Box::new(clone!(result => move |success| {
        result.borrow_mut().error = !success;
      }))),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }

  /// Changes the profile of a sink.
  ///
  /// Panics on error, i.e. invalid arguments or state.
  pub fn set_sink_port_by_name(&mut self, name: &str, port: &str) -> OperationFuture<()> {
    let result = Rc::new(RefCell::new(Value::new(Some(()))));

    let op = Rc::new(self.introspector.set_sink_port_by_name(
      name,
      port,
      Some(Box::new(clone!(result => move |success| {
        result.borrow_mut().error = !success;
      }))),
    ));

    OperationFuture {
      result: result,
      operation: op,
    }
  }
}
