mod model;

pub use model::*;

use napi::{
  threadsafe_function::{ErrorStrategy, ThreadSafeCallContext, ThreadsafeFunction},
  Error, JsFunction, Result,
};
use woodstock::{config::Context, events::read_events, Event};

use crate::config::context::JsBackupContext;

#[napi]
pub fn list_events(
  start_date: String,
  end_date: String,
  context: &JsBackupContext,
  #[napi(ts_arg_type = "(err: null | Error, result: Array<JsEvent>) => void")] callback: JsFunction,
) -> Result<()> {
  let tsfn: ThreadsafeFunction<Vec<Event>, ErrorStrategy::CalleeHandled> = callback
    .create_threadsafe_function(0, |ctx: ThreadSafeCallContext<Vec<Event>>| {
      let events = ctx.value;
      let events: Vec<JsEvent> = events
        .into_iter()
        .map(|e| JsEvent::from_js(e, ctx.env))
        .collect();

      Ok(vec![events])
    })?;

  // Get the path events
  let ctxt: Context = context.into();
  let events = ctxt.config.path.events_path;

  // String (format YYYY-MM-DD) to chrono date
  let start_date = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
    .map_err(|e| Error::from_reason(format!("Can't parse start date {:?}", e).to_string()))?;
  let end_date = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
    .map_err(|e| Error::from_reason(format!("Can't parse end date {:?}", e).to_string()))?;

  let tsfn = tsfn.clone();
  tokio::spawn(async move {
    // Read events from the file
    let events = read_events(events, &start_date, &end_date)
      .await
      .map_err(|e| {
        Error::from_reason(format!("Can't read the events database {:?}", e).to_string())
      });

    tsfn.call(
      events,
      napi::threadsafe_function::ThreadsafeFunctionCallMode::Blocking,
    );
  });

  Ok(())
}
