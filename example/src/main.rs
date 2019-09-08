use futures_util::stream::StreamExt;
use libpulse_binding as pulse;
use libpulse_binding::def::PortAvailable;
use libpulse_futures::context::{flags, Context, Proplist};
use pulse::context::subscribe::subscription_masks;
use glib::MainContext;
use glib::MainLoop;

async fn print_sinks_and_volume(context: &Context) {
  let introspect = context.introspect();

  let sinks = introspect.get_sink_info_list().await.unwrap();

  for sink in sinks {
    for port in sink.ports {
      if port.available == PortAvailable::Yes {
        println!(
          "{} - {}: {}%",
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
          (sink.volume.avg().0 as f32 / sink.n_volume_steps as f32 * 100.0) as u8
        );
      }
    }
  }
}

async fn example(mut c: MainContext) {
  let mut proplist = Proplist::new().unwrap();
  proplist
    .set_str(
      pulse::proplist::properties::APPLICATION_NAME,
      "libpulse-futures example",
    )
    .unwrap();

  let mut context = Context::new_with_maincontext_and_proplist(&mut c, "libpulse-futures example context", &proplist);

  context
    .connect(None, flags::NOFLAGS, None)
    .await
    .expect("Failed to connect context");

  print_sinks_and_volume(&context).await;

  let interest = subscription_masks::SINK;

  let mut subscription = context.subscribe(interest);
  while let Some(_) = subscription.next().await {
    println!("");
    println!("Update:");

    print_sinks_and_volume(&context).await;
  }
}

fn main() {
  let c = MainContext::default();
  c.push_thread_default();
  let l = MainLoop::new(Some(&c), false);
  c.spawn_local(example(c.clone()));
  l.run();
  c.pop_thread_default();
}
