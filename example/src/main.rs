use dbus_futures::context::{flags, Context, Proplist};
use futures::executor::block_on;
use libpulse_binding as pulse;

async fn example() {
  let mut proplist = Proplist::new().unwrap();
  proplist
    .set_str(
      pulse::proplist::properties::APPLICATION_NAME,
      "dbus-futures example",
    )
    .unwrap();

  let mut context = Context::new_with_proplist("dbus-futures example context", &proplist);

  context
    .connect(None, flags::NOFLAGS, None)
    .await
    .expect("Failed to connect context");

  let introspect = context.introspect();

  let sinks = introspect.get_sink_info_list().await.unwrap();

  for sink in sinks {
    for port in sink.ports {
      println!(
        "{} - {}",
        port
          .description
          .as_ref()
          .or(port.name.as_ref())
          .unwrap_or(&"???".to_string()),
        sink
          .description
          .as_ref()
          .or(sink.name.as_ref())
          .unwrap_or(&"???".to_string()),
      );
    }
  }
}

fn main() {
  block_on(example());
}
