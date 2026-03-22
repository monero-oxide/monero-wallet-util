#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![doc = include_str!("../README.md")]
#![deny(missing_docs)]

use core::{
  pin::Pin,
  task::{Poll, Context},
  future::Future,
};
extern crate alloc;
use alloc::{vec::Vec, string::String};

use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{js_sys::Uint8Array, Request, RequestInit, Response};

pub use monero_daemon_rpc::prelude;
use monero_daemon_rpc::{prelude::InterfaceError, HttpTransport, MoneroDaemon};

/// An transport to connect to a Monero daemon, using the web browser's HTTP APIs.
#[derive(Clone, Debug)]
pub struct WebSysTransport {
  url: String,
}

impl WebSysTransport {
  /// Create a new RPC connection, using the web browser's HTTP APIs.
  pub fn new(url: String) -> Result<MoneroDaemon<WebSysTransport>, InterfaceError> {
    Ok(MoneroDaemon::new(WebSysTransport { url }))
  }
}

#[doc(hidden)]
pub async fn __unsafe_internal_post(
  url: &str,
  route: &str,
  body: Vec<u8>,
) -> Result<Vec<u8>, InterfaceError> {
  let opts = RequestInit::new();
  opts.set_method("POST");
  opts.set_body(&Uint8Array::from(body.as_slice()).into());
  let request = Request::new_with_str_and_init(&(url.to_string() + "/" + route), &opts)
    .map_err(|e| InterfaceError::InternalError(format!("failed to create request: {e:?}")))?;

  let window = web_sys::window()
    .ok_or_else(|| InterfaceError::InternalError("failed to acquire window".to_string()))?;

  let response = JsFuture::from(window.fetch_with_request(&request)).await.map_err(|e| {
    InterfaceError::InvalidInterface(format!("failed to make request to RPC: {e:?}"))
  })?;
  if !response.is_instance_of::<Response>() {
    Err(InterfaceError::InternalError("fetch result wasn't a response".to_string()))?;
  }
  let response: Response = response.dyn_into().expect("response type was just checked");

  // Check if response is OK
  if !response.ok() {
    return Err(InterfaceError::InvalidInterface(format!(
      "HTTP error: {} {}",
      response.status(),
      response.status_text()
    )));
  }

  // Get response as ArrayBuffer
  let array_buffer = JsFuture::from(
    response
      .array_buffer()
      .map_err(|e| InterfaceError::InternalError(format!("failed to get array buffer: {e:?}")))?,
  )
  .await
  .map_err(|e| InterfaceError::InternalError(format!("failed to await array buffer: {e:?}")))?;

  // Convert ArrayBuffer to Uint8Array and then to Vec<u8>
  let uint8_array = Uint8Array::new(&array_buffer);
  let mut result = vec![0; uint8_array.length() as usize];
  uint8_array.copy_to(&mut result);

  Ok(result)
}

#[repr(transparent)]
struct UnsafeSendFuture<F: Future>(F);
unsafe impl<F: Future> Send for UnsafeSendFuture<F> {}
impl<F: Future> Future for UnsafeSendFuture<F> {
  type Output = F::Output;
  fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    let actual: Pin<&mut F> = unsafe { core::mem::transmute(self) };
    actual.poll(cx)
  }
}

impl HttpTransport for WebSysTransport {
  fn post(
    &self,
    route: &str,
    body: Vec<u8>,
    _response_size_limit: Option<usize>,
  ) -> impl Send + Future<Output = Result<Vec<u8>, InterfaceError>> {
    UnsafeSendFuture(__unsafe_internal_post(&self.url, route, body))
  }
}
